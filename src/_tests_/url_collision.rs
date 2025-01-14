use ntest::{assert_true, timeout};

use crate::{
    _tests_::util::init_test_app,
    tasks::{Method, QueryBuilder, QueryStore},
};

#[timeout(1000)]
#[test]
fn url_collision_cache_fetch() {
    let url = "http://127.0.0.1:8080/url_collision_cache_fetch";
    let mut app = init_test_app();
    let mut commands = app.world_mut().commands();

    commands.trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .build()
            .unwrap(),
    );

    commands.trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .params(vec![(String::from("param1"), "value1".to_string())])
            .build()
            .unwrap(),
    );

    let store = app.world().get_resource::<QueryStore>().unwrap();
    let size = store.cache.len();

    app.update();

    assert_true!(size == 1);
}
