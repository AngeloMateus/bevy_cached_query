use crate::{
    _tests_::util::init_test_app,
    extractor::{check_completed_queries, QueryConsumable},
    tasks::{Method, QueryBuilder, QuerySequence, QueryStore},
};
use bevy::prelude::*;
use ntest::{assert_true, timeout};

#[timeout(4)]
#[test]
fn sequence() {
    let url1 = "http://127.0.0.1:8080/seq1";
    let url2 = "http://127.0.0.1:8080/seq2";
    let mut app = init_test_app();

    assert_true!(app.world().contains_resource::<QueryStore>());

    app.world_mut().commands().trigger(QuerySequence {
        key: "sequence".to_string(),
        tasks: vec![
            QueryBuilder::default()
                .method(Method::Post)
                .url(url1)
                .build()
                .unwrap(),
            QueryBuilder::default()
                .method(Method::Get)
                .url(url2)
                .build()
                .unwrap(),
        ]
        .into(),
    });

    loop {
        let tasks = vec![
            QueryConsumable {
                url: url1.to_string(),
                ..default()
            },
            QueryConsumable {
                url: url2.to_string(),
                ..default()
            },
        ];
        let mut store = app.world_mut().get_resource_mut::<QueryStore>().unwrap();
        println!(
            "cache {:#?} \nsequences {:#?}\n-------------------------------------------\n",
            store
                .cache
                .iter()
                .map(|a| (a.1 .1.url.clone(), a.1 .2))
                .collect::<Vec<(String, u128)>>(),
            store.sequences
        );
        if check_completed_queries(tasks.clone(), &mut store.cache) {
            let requests = store
                .cache
                .iter()
                .map(|a| (a.1 .1.url.clone(), a.1 .2))
                .collect::<Vec<(String, u128)>>();
            let url1_time = requests
                .iter()
                .find(|&(url, _)| url == url1)
                .map(|&(_, time)| time)
                .unwrap();
            let url2_time = requests
                .iter()
                .find(|&(url, _)| url == url2)
                .map(|&(_, time)| time)
                .unwrap();
            assert!(url1_time <= url2_time, "url1 was not called before url2");
            break;
        }

        app.update();
    }
}
