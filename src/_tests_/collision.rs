use crate::{
    _tests_::util::{init_test_app, GetResponse},
    extractor::{query_extractor, QueryConsumable},
    tasks::{Method, QueryBuilder, QueryStore},
};
use ntest::assert_true;

#[test]
fn same_query_key() {
    let url = "http://127.0.0.1:8080/same_query_key";
    let mut app = init_test_app();

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .query_key("collision_test")
            .build()
            .unwrap(),
    );

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .params(vec![(String::from("param1"), "value1".to_string())])
            .query_key("collision_test")
            .build()
            .unwrap(),
    );

    app.update();

    let store = app.world().get_resource::<QueryStore>().unwrap();
    assert_true!(store.loading_requests.len() == 1);
}

#[test]
fn same_url() {
    let url = "http://127.0.0.1:8080/same_url";
    let mut app = init_test_app();

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .build()
            .unwrap(),
    );

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .params(vec![(String::from("second_request"), "is_discarded".to_string())])
            .build()
            .unwrap(),
    );

    loop {
        let mut store = app.world_mut().get_resource_mut::<QueryStore>().unwrap();
        let result = query_extractor::<GetResponse>(
            QueryConsumable {
                url: url.to_string(),
                ..Default::default()
            },
            &mut store,
        );

        if let Ok(response) = result {
            println!("test response {:#?}", response.msg);
            assert_eq!(response.msg, "success");
            break;
        }

        app.update();
    }
    assert_true!(
        app.world_mut()
            .get_resource_mut::<QueryStore>()
            .unwrap()
            .cache
            .len()
            == 1
    );
}

#[test]
fn same_url_different_query_key() {
    let url = "http://127.0.0.1:8080/same_url_different_query_key";
    let mut app = init_test_app();

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .query_key("key1")
            .build()
            .unwrap(),
    );

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url)
            .query_key("key2")
            .build()
            .unwrap(),
    );

    app.update();
    let store = app.world().get_resource::<QueryStore>().unwrap();
    assert_true!(store.loading_requests.len() == 2);
}
