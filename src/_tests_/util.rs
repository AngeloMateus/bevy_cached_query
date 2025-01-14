use bevy::app::{App, Update};

use crate::tasks::{api_task_poll, api_task_sequence, spawn_api_task, QueryStore};

pub fn init_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, api_task_poll);
    app.init_resource::<QueryStore>();
    app.observe(spawn_api_task);
    app.observe(api_task_sequence);

    app
}
