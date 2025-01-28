use crate::tasks::{api_task_poll, api_task_sequence, spawn_api_task, watch_cache, QueryStore};
use bevy::app::{App, Update};
use serde::Deserialize;

pub fn init_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, api_task_poll);
    app.add_systems(Update, watch_cache);
    app.init_resource::<QueryStore>();
    app.observe(spawn_api_task);
    app.observe(api_task_sequence);

    app
}

#[derive(Deserialize)]
pub struct GetResponse {
    pub msg: String,
}
