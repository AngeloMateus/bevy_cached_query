use crate::{
    _tests_::util::init_test_app,
    tasks::{Method, QueryBuilder, QueryStore},
};
use ntest::{assert_true, timeout};

#[timeout(1000)]
#[test]
fn loading() {
    let url = "http://127.0.0.1:8080/";
    let mut app = init_test_app();

    assert_true!(app.world().contains_resource::<QueryStore>());

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url(url.to_string())
            .build()
            .unwrap(),
    );
    app.update();
    let store = app.world().get_resource::<QueryStore>().unwrap();
    assert_true!(store.loading_requests.iter().len() == 1);
}
