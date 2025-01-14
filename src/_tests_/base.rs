use std::{sync::Mutex, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use ntest::{assert_true, timeout};
use serde::Deserialize;
use tiny_http::{Response, Server};

use crate::{
    extractor::{query_extractor, QueryConsumable},
    tasks::{
        api_task_poll, api_task_sequence, loading_requests_is_empty, spawn_api_task, Method, Query,
        QueryBuilder, QueryStore,
    },
};

fn init_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, api_task_poll);
    app.init_resource::<QueryStore>();
    app.observe(spawn_api_task);
    app.observe(api_task_sequence);

    app
}

#[timeout(50)]
#[test]
fn query_cache() {
    let mut app = init_test_app();
    assert_true!(app.world().contains_resource::<QueryStore>());

    let server = Server::http("127.0.0.1:8080").unwrap();

    app.world_mut().commands().trigger(
        QueryBuilder::default()
            .method(Method::Get)
            .url("http://127.0.0.1:8080".to_string())
            .build()
            .unwrap(),
    );

    #[derive(Deserialize)]
    struct GetResponse {
        msg: String,
    }

    loop {
        server.unblock();
        for request in server.incoming_requests() {
            let response = Response::from_string("{\"msg\": \"hello world\"}");
            request.respond(response).expect("Responded");
        }
        let mut store = app.world_mut().get_resource_mut::<QueryStore>().unwrap();
        let result = query_extractor::<GetResponse>(
            QueryConsumable {
                url: "http://127.0.0.1:8080".to_string(),
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
