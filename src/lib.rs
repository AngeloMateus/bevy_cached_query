use bevy::{prelude::*, time::common_conditions::on_timer};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tasks::{
    api_task_poll, api_task_sequence, loading_requests_is_empty, spawn_api_task, watch_cache, QueryStore,
};

mod _tests_;
pub mod extractor;
mod logging;
pub mod tasks;

pub struct QueryTasksPlugin;
pub type Query = tasks::Query;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub msg: String,
    pub status: String,
}

impl Plugin for QueryTasksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            api_task_poll
                .run_if(on_timer(Duration::from_millis(100)))
                .run_if(not(loading_requests_is_empty)),
        )
        .add_systems(FixedUpdate, watch_cache)
        .init_resource::<QueryStore>()
        .observe(spawn_api_task)
        .observe(api_task_sequence);
    }
}
