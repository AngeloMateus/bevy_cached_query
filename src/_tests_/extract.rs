use crate::{
    _tests_::util::{init_test_app, GetResponse},
    extractor::{query_extractor, QueryConsumable},
    tasks::{Method, QueryBuilder, QueryStore},
};
use bevy::prelude::*;
use ntest::{assert_true, timeout};

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
            .build()
            .unwrap(),
    );

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

#[timeout(1000)]
#[test]
fn force_next_refetch() {
    let url = "http://127.0.0.1:8080/force_next_refetch";
    let mut app = init_test_app();

    assert_true!(app.world().contains_resource::<QueryStore>());

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url.to_string())
            .build()
            .unwrap(),
    );

    for index in 0..2 {
        let mut store = app.world_mut().get_resource_mut::<QueryStore>().unwrap();
        let result = query_extractor::<GetResponse>(
            QueryConsumable {
                url: url.to_string(),
                force_next_refetch: true,
                ..default()
            },
            &mut store.cache,
        );
        if index > 0 {
            assert!(result.is_err());
        } else if let Ok(response) = result {
            assert_eq!(response.msg, "Should be consumed once");
            break;
        }

        app.update();
    }
}
