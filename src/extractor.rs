use crate::{debug_end, logging::PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS};
use anyhow::{anyhow, Ok, Result};
use bevy::{log::error, prelude::Event, utils::HashMap};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Default, Clone, Event, Debug)]
pub struct QueryConsumable {
    pub url: String,
    pub query_key: Option<String>,
    pub force_next_refetch: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    msg: Option<String>,
    status: u16,
    body: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct TMessageResponse {
    pub msg: String,
}

/// Checks if a vector of tasks has finished loading\
/// Useful for sequences of tasks
pub fn check_completed_queries(
    consumables: Vec<QueryConsumable>,
    store: &mut HashMap<(String, String), serde_json::Value>,
) -> bool {
    consumables.iter().all(|task| {
        let endpoint = task.url.split("?").next().unwrap_or("");
        let query_key = task.query_key.clone().unwrap_or_default();

        if !store.contains_key(&(endpoint.to_string(), query_key)) {
            return false;
        }
        true
    })
}

/// Returns the latest response for given endpoint and removes it from cache\
/// @TODO if there is a more recent request in the loading_requests, return that instead and clear all older requests
pub fn query_extractor<T>(
    consumable: QueryConsumable,
    store: &mut HashMap<(String, String), serde_json::Value>,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let start = SystemTime::now();
    let url = consumable.url;
    let query_key = consumable.query_key.clone();
    let force_next_refetch = consumable.force_next_refetch;
    let mut extracted_task = None;
    if !force_next_refetch {
        let extracted_ref = store.get(&(url.clone(), query_key.unwrap_or_default()));
        if let Some(extr) = extracted_ref {
            extracted_task = Some(extr.clone());
        }
    } else {
        let extracted: Vec<((String, String), serde_json::Value)> = store
            .extract_if(|e, _| {
                e.0.eq(&url) && e.1.to_string().eq(&query_key.clone().unwrap_or_default())
            })
            .collect();

        if !extracted.is_empty() {
            extracted_task = Some(extracted.first().unwrap().1.clone());
        }
    }

    match extracted_task {
        Some(value) => {
            let api_consumable: Response = serde_json::from_value(value.clone())?;

            if api_consumable.status != 200 {
                error!("API error {:?}", api_consumable.msg);
                return Err(anyhow!(api_consumable.status));
            }

            if api_consumable.body.is_none() {
                return Err(anyhow!("Failed to extract body"));
            }

            let body: T = serde_json::from_value(api_consumable.body.unwrap())?;

            debug_end!(start, PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS);
            Ok(body)
        }
        None => Err(anyhow!("No tasks matched {}", url)),
    }
}
