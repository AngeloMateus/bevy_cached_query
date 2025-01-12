use crate::{debug_end, logging::PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS, proto};
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
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
/**
Stores all API tasks
*/
pub struct QueryStores {
    /**
    Hashmap: (url, timestamp, task_sequence) -> response
    */
    pub loading_requests:
        HashMap<(String, String, Option<String>), Task<Result<Response, ureq::Error>>>,
    /**
    Hashmap: (url, query_key) -> json value

    TODO: add stale time for to remove from completed_tasks
    */
    pub store: HashMap<(String, String), serde_json::Value>,
    pub sequences: HashMap<String, VecDeque<Query>>,
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TApiMethod {
    #[default]
    Get,
    Post,
}

#[derive(Event, Default, Debug, Eq, PartialEq, Hash, Clone, Builder)]
#[builder(setter(strip_option, into), default)]
pub struct Query {
    pub method: TApiMethod,
    pub url: String,
    pub params: Option<Vec<(String, String)>>,
    pub body: serde_json::Value,
    pub headers: Option<Vec<(String, String)>>,
    pub timeout: Option<Duration>,
    pub query_key: Option<String>,
    sequence_key: Option<String>,
}

/**
Sequence of tasks to execute in order
*/
#[derive(Event, Default, Debug, Eq, PartialEq, Hash, Clone)]
pub struct TApiTaskSequence {
    pub key: String,
    pub tasks: VecDeque<Query>,
}

/**
API request handler
Spawn a new task if the url is not already in api_tasks.loading_requests
*/
pub fn spawn_api_task(trigger: Trigger<Query>, mut api_tasks: ResMut<QueryStores>) {
    let url = trigger.event().url.clone();
    let method = trigger.event().method;
    let params = trigger.event().params.clone();
    let body = trigger.event().body.clone();
    let timeout = trigger.event().timeout;
    let query_key = trigger.event().query_key.clone().unwrap_or_default();
    let sequence = trigger.event().sequence_key.clone();

    //@TODO Add query_key to store and consumer
    let thread_pool = AsyncComputeTaskPool::get();
    let headers = trigger.event().headers.clone();

    if !api_tasks
        .loading_requests
        .contains_key(&(url.clone(), query_key.clone(), sequence.clone()))
    {
        let new_url = url.clone();
        let task = thread_pool.spawn(async move {
            let url = new_url.clone();
            match method {
                TApiMethod::Get => get(url.clone(), params, headers, timeout).await,

                TApiMethod::Post => post(url.clone(), params, body, headers, timeout).await,
            }
        });

        api_tasks
            .loading_requests
            .insert((url.clone(), query_key, sequence), task);
    }
}

/**
Polls the status of all API tasks

Appends completed requests to the store HashMap

Should remove older requests of the same type

@TODO: add stale time for query_key to remove from store
*/
pub fn api_task_poll(
    mut api_tasks: ResMut<QueryStores>,
    // mut app_res: ResMut<ApplicationResource>,
    mut commands: Commands,
) {
    let start = SystemTime::now();
    let mut completed_requests = vec![];
    let current_sequence_tasks = api_tasks.sequences.clone();
    api_tasks
        .loading_requests
        .retain(|(url, query_key, sequence), task| {
            // keep the entry in our HashMap only if the task is not done yet
            let mut retain = true;

            // check task
            let poll_status = block_on(future::poll_once(task));

            // if this task is done, handle return data
            if let Some(response) = poll_status {
                retain = false;

                match response {
                    Ok(res) => {
                        proto!("res from response {:?}", res);
                        match res.into_json::<serde_json::Value>() {
                            Ok(json) => {
                                if let Some(sequence) = sequence {
                                    if let Some(sequence_tasks) =
                                        current_sequence_tasks.clone().get_mut(sequence)
                                    {
                                        if sequence_tasks.pop_front().is_some() {
                                            commands.trigger(TApiTaskSequence {
                                                key: sequence.to_string(),
                                                tasks: sequence_tasks.clone(),
                                            });
                                        };
                                    };
                                }
                                completed_requests.push((
                                    (url.to_string(), query_key.clone()),
                                    json!({"status": 200, "body": json}),
                                ));
                            }
                            Err(err) => {
                                proto!("Failed to deserialize response {:#?}", err);
                                completed_requests.push((
                                    (url.to_string(), query_key.clone()),
                                    json!({"status":500}),
                                ));
                            }
                        }
                    }
                    Err(err) => {
                        proto!("{:#?}", err);
                        if let Some(err_res) = err.into_response() {
                            if err_res.status() == 401 {
                                //@TODO
                                proto!("\n\nSHOULD RE-AUTHENTICATE\nre-add reauthentication \n\n");
                                // app_res.require_authentication = true;
                                // commands.trigger(AuthenticateUser);
                            } else {
                                completed_requests.push((
                                (url.to_string(), query_key.clone()),
                                json!({"status":err_res.status(),"msg": err_res.into_string().unwrap()}),
                            ));
                            }
                        } else {
                            completed_requests.push((
                                (url.to_string(), query_key.clone()),
                                json!({"status":500}),
                            ));
                        }
                    }
                }
            }

            retain
        });

    api_tasks.store.extend(completed_requests);
    debug_end!(start, PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS);
}

pub fn api_task_sequence(
    trigger: Trigger<TApiTaskSequence>,
    mut api_tasks: ResMut<QueryStores>,
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

async fn get(
    url: String,
    params: Option<Vec<(String, String)>>,
    headers: Option<Vec<(String, String)>>,
    timeout: Option<Duration>,
) -> Result<ureq::Response, ureq::Error> {
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(5))
        .build();

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
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(5))
        .build();
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

pub fn task_store_is_empty(api_tasks: Res<QueryStores>) -> bool {
    api_tasks.store.is_empty()
}

pub fn loading_tasks_is_empty(api_tasks: Res<QueryStores>) -> bool {
    api_tasks.loading_requests.is_empty()
}
