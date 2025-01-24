use crate::{debug_end, logging::PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS, tasks::QueryStore, Query};
use anyhow::{anyhow, Ok, Result};
use bevy::{log::error, prelude::Event, utils::HashMap};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Default, Clone, Event, Debug)]
pub struct QueryConsumable {
    pub url: String,
    pub query_key: Option<String>,
    pub force_next_refetch: bool,
    /// Staletime in milliseconds
    pub stale_time: Option<u128>,
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
    cache: &mut HashMap<(String, String), (serde_json::Value, Query, u128)>,
) -> bool {
    consumables.iter().all(|task| {
        let endpoint = task.url.split("?").next().unwrap_or("");
        let query_key = task.query_key.clone().unwrap_or_default();

        if !cache.contains_key(&(endpoint.to_string(), query_key)) {
            return false;
        }
        true
    })
}

/// Returns the latest response for given endpoint and removes it from cache\
pub fn query_extractor<T>(consumable: QueryConsumable, store: &mut QueryStore) -> Result<T>
where
    T: DeserializeOwned,
{
    let start = SystemTime::now();
    let mut extracted_task = None;

    if !consumable.force_next_refetch {
        let extracted_ref = store.cache.get(&(
            consumable.url.clone(),
            consumable.query_key.clone().unwrap_or_default(),
        ));
        if let Some(extr) = extracted_ref {
            extracted_task = Some(extr.clone());
        }
    } else {
        let extracted: Vec<((String, String), (serde_json::Value, Query, u128))> = store
            .cache
            .extract_if(|e, _| {
                e.0.eq(&consumable.url)
                    && e.1
                        .to_string()
                        .eq(&consumable.query_key.clone().unwrap_or_default())
            })
            .collect();

        if !extracted.is_empty() {
            extracted_task = Some(extracted.first().unwrap().1.clone());
        }
    }

    match extracted_task {
        Some(value) => {
            if let Some(stale_duration) = consumable.stale_time {
                if SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_millis()
                    > value.2 + stale_duration
                {
                    store
                        .cache
                        .remove(&(consumable.url.clone(), consumable.query_key.unwrap_or_default()));
                    store.stale_queries.push(value.1);
                    return Err(anyhow!("Task is stale"));
                }
            }

            let api_consumable: Response = serde_json::from_value(value.0.clone())?;

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
        None => Err(anyhow!("No tasks matched {}", consumable.url)),
    }
}
