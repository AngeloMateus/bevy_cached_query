use crate::{
    _tests_::util::{init_test_app, GetResponse},
    extractor::{query_extractor, QueryConsumable},
    tasks::{Method, QueryBuilder, QueryStore},
};
use bevy::prelude::*;
use ntest::{assert_true, timeout};
use std::{thread::sleep, time::Duration};

#[timeout(5000)]
#[test]
fn is_staletime() {
    let url = "http://127.0.0.1:8080/is_stale";
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
        sleep(Duration::from_millis(250));

        let result = query_extractor::<GetResponse>(
            QueryConsumable {
                url: url.to_string(),
                stale_time: Some(200),
                ..default()
            },
            &mut store,
        );

        if let Ok(response) = result {
            assert_ne!(response.msg, "Should not be consumed");
            break;
        }
        if let Err(e) = result {
            if e.to_string().contains("Task is stale") {
                break;
            }
        }

        app.update();
    }
}

#[timeout(1000)]
#[test]
fn refetch() {
    let url = "http://127.0.0.1:8080/refetch";
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

    let mut index = 0;
    loop {
        let mut store = app.world_mut().get_resource_mut::<QueryStore>().unwrap();

        sleep(Duration::from_millis(match index {
            0 => 400,
            _ => 50,
        }));

        let result = query_extractor::<GetResponse>(
            QueryConsumable {
                url: url.to_string(),
                stale_time: Some(200),
                ..default()
            },
            &mut store,
        );
        if let Ok(response) = result {
            assert_eq!(store.cache.iter().count(), 1);
            assert_eq!(response.msg, "Should refetch");
            break;
        }

        if let Err(e) = result {
            if e.to_string().contains("Task is stale") {
                assert_eq!(store.stale_queries.iter().len(), 1);
                continue;
            }
        }

        index += 1;
        app.update();
    }
    assert_true!(
        app.world_mut()
            .get_resource_mut::<QueryStore>()
            .unwrap()
            .cache
            .iter()
            .len()
            == 1
    );
}
