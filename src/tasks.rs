use crate::{
    debug_end, extractor::QueryConsumable, logging::PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS, proto,
};
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task, TaskPool},
    utils::HashMap,
};
use derive_builder::Builder;
use serde_json::json;
use std::{
    collections::VecDeque,
    time::{Duration, SystemTime},
};
use ureq::Response;

#[allow(clippy::type_complexity)]
#[derive(Resource, Default, Debug)]
/// Stores all API tasks
pub struct QueryStore {
    /// Hashmap: (url, query_key, task_sequence) -> (response, query, called at)
    pub loading_requests:
        HashMap<(String, String, Option<String>), Task<(Result<Response, ureq::Error>, Query, u128)>>,
    /// Hashmap: (url, query_key) -> json (value, called at)\
    pub cache: HashMap<(String, String), (serde_json::Value, Query, u128)>,
    pub sequences: HashMap<String, VecDeque<Query>>,
    pub stale_queries: Vec<Query>,
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Method {
    #[default]
    Get,
    Post,
    Delete,
}

#[derive(Event)]
pub struct ErrorTriggerEvent {
    pub url: String,
    pub error: u16,
}

#[derive(Event, Default, Debug, Eq, PartialEq, Hash, Clone, Builder)]
#[builder(setter(strip_option, into), default)]
pub struct Query {
    pub method: Method,
    pub url: String,
    pub params: Option<Vec<(String, String)>>,
    pub body: serde_json::Value,
    pub headers: Option<Vec<(String, String)>>,
    pub timeout: Option<Duration>,
    /// Querys with the same query_key will be cached, if no query key is provided, the url will be used as the key instead
    pub query_key: Option<String>,
    pub skip_cache_check: Option<bool>,
    sequence_key: Option<String>,
}

/// Sequence of tasks to execute in order
#[derive(Event, Default, Debug, Clone)]
pub struct QuerySequence {
    pub key: String,
    pub tasks: VecDeque<Query>,
}

/// Tas sequence consumeable
#[derive(Event, Default, Debug, Clone)]
pub struct QuerySequenceConsumeable {
    pub key: String,
    pub tasks: VecDeque<QueryConsumable>,
}

/// API request handler
/// Spawn a new task if the url is not already in api_tasks.loading_requests
pub fn spawn_api_task(trigger: Trigger<Query>, mut query_store: ResMut<QueryStore>) {
    let url = trigger.event().url.clone();
    let query_key = trigger.event().query_key.clone().unwrap_or_default();

    if query_store.cache.contains_key(&(url.clone(), query_key.clone())) {
        return;
    }
    let method = trigger.event().method;
    let params = trigger.event().params.clone();
    let body = trigger.event().body.clone();
    let timeout = trigger.event().timeout;
    let sequence_key = trigger.event().sequence_key.clone();

    let thread_pool = AsyncComputeTaskPool::get_or_init(TaskPool::new);
    let headers = trigger.event().headers.clone();
    let key_exists = query_store.loading_requests.contains_key(&(
        url.clone(),
        query_key.clone(),
        sequence_key.clone(),
    ));
    let new_url = url.clone();
    let query = trigger.event().clone();
    let skip_cache_check = trigger.event().skip_cache_check.unwrap_or_default();
    let call_get = method == Method::Get;

    if call_get && !skip_cache_check && !key_exists {
        println!("calling get");
        let task = thread_pool.spawn(async move {
            let url = new_url.clone();
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            (
                get(url.clone(), params.clone(), headers, timeout).await,
                query.clone(),
                now,
            )
        });

        query_store
            .loading_requests
            .insert((url.clone(), query_key, sequence_key), task);
    } else if !call_get {
        let task = thread_pool.spawn(async move {
            let url = new_url.clone();
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            (
                match method {
                    Method::Post => {
                        post(url.clone(), params.clone(), body.clone(), headers, timeout).await
                    }
                    _ => delete(url.clone(), params.clone(), headers, timeout).await,
                },
                query.clone(),
                now,
            )
        });

        query_store
            .loading_requests
            .insert((url.clone(), query_key, sequence_key), task);
    }
}

/// Polls the status of all API tasks
///
/// Appends completed requests to the store HashMap
///
/// Should remove matching older requests
///
/// @TODO: add stale time for query_key to remove from store
pub fn api_task_poll(mut query_store: ResMut<QueryStore>, mut commands: Commands) {
    let start = SystemTime::now();
    let mut completed_requests = vec![];
    let current_sequence_tasks = query_store.bypass_change_detection().sequences.clone();
    query_store
        .bypass_change_detection()
        .loading_requests
        .retain(|(url, query_key, sequence), task| {
            // keep the entry in our HashMap only if the task is not done yet
            let mut retain = true;


            // check task
            let poll_status = block_on(future::poll_once(task));

            // if this task is done, handle return data
            if let Some(st) = poll_status {
                retain = false;

                match st.0 {
                    Ok(res) => match res.into_json::<serde_json::Value>() {
                        Ok(json) => {
                            if let Some(sequence) = sequence {
                                if let Some(sequence_tasks) =
                                    current_sequence_tasks.clone().get_mut(sequence)
                                {
                                    if sequence_tasks.pop_front().is_some() {
                                        commands.trigger(QuerySequence {
                                            key: sequence.to_string(),
                                            tasks: sequence_tasks.clone(),
                                        });
                                    };
                                };
                            }
                            completed_requests.push((
                                (url.to_string(), query_key.clone()),
                                (json!({"status": 200, "body": json}), st.1, st.2),
                            ));
                        }
                        Err(err) => {
                            proto!("Failed to deserialize response {:#?}", err);
                            completed_requests.push((
                                (url.to_string(), query_key.clone()),
                                (json!({"status":500}), st.1, st.2),
                            ));
                        }
                    },
                    Err(err) => {
                        proto!("{:#?}", err);
                        if let Some(err_res) = err.into_response() {
                            commands.trigger(ErrorTriggerEvent {
                                error: err_res.status(),
                                url: url.to_string(),
                            });
                            completed_requests.push((
                                (url.to_string(), query_key.clone()),
                                (json!({"status":err_res.status(),"msg": err_res.into_string().unwrap()}),st.1, st.2),
                            ));
                        } else {
                            completed_requests
                                .push(((url.to_string(), query_key.clone()), (json!({"status":500}), st.1, st.2)));
                        }
                    }
                }
            }

            retain
        });

    query_store.cache.extend(completed_requests);
    debug_end!(start, PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS);
}

//@TODO this does not wait for request to complete
pub fn api_task_sequence(
    trigger: Trigger<QuerySequence>,
    mut api_tasks: ResMut<QueryStore>,
    mut commands: Commands,
) {
    api_tasks
        .sequences
        .insert(trigger.event().key.clone(), trigger.event().tasks.clone());

    let tasks = api_tasks.sequences.get(&trigger.event().key).unwrap();
    if !tasks.is_empty() {
        let next_task = Query {
            sequence_key: Some(trigger.event().key.clone()),
            ..tasks[0].clone()
        };
        commands.trigger(next_task);
    }
}

pub fn watch_cache(mut query_store: ResMut<QueryStore>, mut commands: Commands) {
    if !query_store.stale_queries.is_empty() {
        let removed = query_store.stale_queries.remove(0);
        commands.trigger(removed);
    }
}

async fn get(
    url: String,
    params: Option<Vec<(String, String)>>,
    headers: Option<Vec<(String, String)>>,
    timeout: Option<Duration>,
) -> Result<ureq::Response, ureq::Error> {
    let agent = ureq::builder().timeout_connect(Duration::from_secs(5)).build();

    let mut request = agent
        .get(url.as_str())
        .query_pairs(
            params
                .unwrap_or_default()
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .timeout(timeout.unwrap_or(Duration::from_secs(5)));
    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.set(&key, &value);
        }
    }
    let response = request.call()?;

    Ok(response)
}

async fn post(
    url: String,
    params: Option<Vec<(String, String)>>,
    body: serde_json::Value,
    headers: Option<Vec<(String, String)>>,
    timeout: Option<Duration>,
) -> Result<ureq::Response, ureq::Error> {
    let agent = ureq::builder().timeout_connect(Duration::from_secs(5)).build();
    let mut request = agent
        .post(url.as_str())
        .query_pairs(
            params
                .unwrap_or_default()
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .timeout(timeout.unwrap_or(Duration::from_secs(5)));

    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.set(&key, &value);
        }
    }

    let response = request.send_json(body)?;
    Ok(response)
}

async fn delete(
    url: String,
    params: Option<Vec<(String, String)>>,
    headers: Option<Vec<(String, String)>>,
    timeout: Option<Duration>,
) -> Result<ureq::Response, ureq::Error> {
    let agent = ureq::builder().timeout_connect(Duration::from_secs(5)).build();
    let mut request = agent
        .delete(url.as_str())
        .query_pairs(
            params
                .unwrap_or_default()
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .timeout(timeout.unwrap_or(Duration::from_secs(5)));

    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.set(&key, &value);
        }
    }

    let response = request.call()?;
    Ok(response)
}

pub fn query_store_is_empty(store: Res<QueryStore>) -> bool {
    store.cache.is_empty()
}

pub fn loading_requests_is_empty(store: Res<QueryStore>) -> bool {
    store.loading_requests.is_empty()
}
