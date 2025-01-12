use bevy::{prelude::*, time::common_conditions::on_timer};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use task_queue::{
    api_task_poll, api_task_sequence, loading_tasks_is_empty, spawn_api_task, QueryStores,
};

pub mod extractor;
mod logging;
pub mod task_queue;

pub struct QueryTasksPlugin;
pub type Query = task_queue::Query;

#[derive(Serialize, Deserialize)]
pub struct TApiErrorResponse {
    pub msg: String,
    pub status: String,
}

impl Plugin for QueryTasksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            api_task_poll
                .run_if(on_timer(Duration::from_millis(100)))
                .run_if(not(loading_tasks_is_empty)),
        );
        app.init_resource::<QueryStores>();
        app.observe(spawn_api_task);
        app.observe(api_task_sequence);
    }
}
