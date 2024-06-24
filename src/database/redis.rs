use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::errors::ApiError;
use crate::report_error;
use redis::{Commands};
use tonic::{Response, Status};
use tonic::metadata::MetadataValue;

pub fn connect_redis(redis_url: String) -> Result<redis::Client, redis::RedisError> {
    redis::Client::open(redis_url)
}

#[derive(Clone)]
pub struct CacheClient {
    client: Arc<redis::Client>,
    cache_ttl: u64,
}

impl CacheClient {
    pub fn new(client: redis::Client, cache_ttl: u64) -> Self {
        CacheClient {
            client: Arc::new(client),
            cache_ttl,
        }
    }

    fn generate_cache_key(&self, method_name: &str, request: &impl Serialize) -> String {
        format!("{}:{}", method_name, serde_json::to_string(request).unwrap())
    }

    async fn get_cache<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        let mut conn = self.client.get_connection().map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        let data: Option<Vec<u8>> = conn.get(key).map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        if let Some(data) = data {
            let result = serde_json::from_slice(&data).map_err(|e| {
                report_error(&e);
                ApiError::CacheError
            })?;

            log::debug!("Cache hit for key: {}", key);

            Ok(Some(result))
        } else {

            log::debug!("Cache miss for key: {}", key);

            Ok(None)
        }
    }

    async fn set_cache<T: Serialize>(&self, key: &str, value: &T) -> Result<(), ApiError> {
        let mut conn = self.client.get_connection().map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        let data = serde_json::to_vec(value).map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        conn.set_ex(key, data, self.cache_ttl).map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        log::debug!("Cache set for key: {}", key);

        Ok(())
    }

    pub(crate) async fn invalid_cache(&self, method_name: &str, request: &impl Serialize) -> Result<(), ApiError> {
        let cache_key = self.generate_cache_key(method_name, request);
        let mut conn = self.client.get_connection().map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        conn.del(cache_key).map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        Ok(())
    }

    pub(crate) async fn invalidate_related_cache_keys(&self, organizer_key: String) -> Result<(), ApiError> {
        let mut conn = self.client.get_connection().map_err(|e| {
            report_error(&e);
            ApiError::CacheError
        })?;

        let keys_to_invalidate = vec![
            "list_*:{\"filters\":{*\"organizerKey\":\"".to_string() + &organizer_key + "\"*}*",
        ];

        log::debug!("Invalidating cache keys: {:?}", keys_to_invalidate);

        for key_pattern in keys_to_invalidate {
            let keys: Vec<String> = conn.keys(key_pattern).map_err(|e| {
                report_error(&e);
                ApiError::CacheError
            })?;
            for key in keys {
                conn.del(&key).map_err(|e| {
                    report_error(&e);
                    ApiError::CacheError
                })?;
            }
        }

        Ok(())
    }

    pub(crate) async fn handle_cache<T, F, Fut>(
        &self,
        method_name: &str,
        request: &impl Serialize,
        call: F,
    ) -> Result<Response<T>, Status>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<Response<T>, Status>> + Send,
    {
        static CACHE_STATUS: &str = "x-cache";
        static CACHE_CONTROL: &str = "cache-control";

        let cache_key = self.generate_cache_key(method_name, request);

        // Cache hit
        if let Some(cached_response) = self.get_cache::<T>(&cache_key).await? {
            let mut response = Response::new(cached_response);
            response.metadata_mut().insert(
                CACHE_STATUS,
                MetadataValue::from_static("HIT"),
            );

            let control_content = format!("max-age={}, must-revalidate", self.cache_ttl);
            response.metadata_mut().insert(
                CACHE_CONTROL,
                MetadataValue::from_str(&control_content).unwrap(),
            );

            return Ok(response);
        }

        // Cache miss
        let mut response = call().await?;

        // Add response to cache
        self.set_cache(&cache_key, response.get_ref()).await?;

        response.metadata_mut().insert(
            CACHE_STATUS,
            MetadataValue::from_static("MISS"),
        );

        let control_content = format!("max-age={}, must-revalidate", self.cache_ttl);
        response.metadata_mut().insert(
            CACHE_CONTROL,
            MetadataValue::from_str(&control_content).unwrap(),
        );

        Ok(response)
    }
}
