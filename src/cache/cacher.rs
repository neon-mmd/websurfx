//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use error_stack::Report;
use mini_moka::sync::Cache as MokaCache;
use std::time::Duration;
use tokio::sync::Mutex;

use super::{error::PoolError, redis_cacher::RedisCache};

/// Different implementations for caching, currently it is possible to cache in-memory or in Redis.
#[derive(Clone)]
pub enum Cache {
    /// Encapsulates the Redis based cache
    Redis(RedisCache),
    /// Contains the in-memory cache.
    InMemory(MokaCache<String, String>),
}

impl Cache {
    /// Creates a new cache, which wraps the given RedisCache.
    pub fn new(redis_cache: RedisCache) -> Self {
        Cache::Redis(redis_cache)
    }

    /// Creates an in-memory cache
    pub fn new_in_memory() -> Self {
        let cache = MokaCache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(60))
            .build();
        Cache::InMemory(cache)
    }

    /// A function which fetches the cached json results as json string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as a string.
    pub async fn cached_json(&mut self, url: &str) -> Result<String, Report<PoolError>> {
        match self {
            Cache::Redis(redis_cache) => redis_cache.cached_json(url).await,
            Cache::InMemory(in_memory) => match in_memory.get(&url.to_string()) {
                Some(res) => Ok(res),
                None => Err(Report::new(PoolError::MissingValue)),
            },
        }
    }

    /// A function which caches the results by using the `url` as the key and
    /// `json results` as the value and stores it in the cache
    ///
    /// # Arguments
    ///
    /// * `json_results` - It takes the json results string as an argument.
    /// * `url` - It takes the url as a String.
    pub async fn cache_results(
        &mut self,
        json_results: String,
        url: &str,
    ) -> Result<(), Report<PoolError>> {
        match self {
            Cache::Redis(redis_cache) => redis_cache.cache_results(&json_results, url).await,
            Cache::InMemory(cache) => {
                cache.insert(url.to_string(), json_results);
                Ok(())
            }
        }
    }
}

/// A structure to efficiently share the cache between threads - as it is protected by a Mutex.
pub struct SharedCache {
    cache: Mutex<Cache>,
}

impl SharedCache {
    /// Creates a new SharedCache from a Cache implementation
    pub fn new(cache: Cache) -> Self {
        Self {
            cache: Mutex::new(cache),
        }
    }

    /// A function which fetches the cached json results as json string.
    pub async fn cached_json(&self, url: &str) -> Result<String, Report<PoolError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cached_json(url).await
    }

    /// A function which caches the results by using the `url` as the key and
    /// `json results` as the value and stores it in the cache
    pub async fn cache_results(
        &self,
        json_results: String,
        url: &str,
    ) -> Result<(), Report<PoolError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cache_results(json_results, url).await
    }
}
