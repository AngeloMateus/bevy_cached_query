use bevy::prelude::*;
use ntest::{assert_true, timeout};
use serde::Deserialize;

use crate::{
    _tests_::util::init_test_app,
    extractor::{query_extractor, QueryConsumable},
    tasks::{Method, QueryBuilder, QueryStore},
};

#[timeout(1000)]
#[test]
fn extractor() {
    let url = "http://127.0.0.1:8080/extractor";
    let mut app = init_test_app();

    assert_true!(app.world().contains_resource::<QueryStore>());

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url.to_string())
            .headers(vec![("Connection".to_string(), "close".to_string())])
            .build()
            .unwrap(),
    );

    #[derive(Deserialize)]
    struct GetResponse {
        msg: String,
    }

    loop {
        let mut store = app.world_mut().get_resource_mut::<QueryStore>().unwrap();
        let result = query_extractor::<GetResponse>(
            QueryConsumable {
                url: url.to_string(),
                ..default()
            },
            &mut store.cache,
        );
        if let Ok(response) = result {
            assert_eq!(response.msg, "hello world");
            break;
        }

        app.update();
    }
}
