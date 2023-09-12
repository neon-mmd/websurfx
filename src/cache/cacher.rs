//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use error_stack::Report;
#[cfg(feature = "in_memory_cache")]
use mini_moka::sync::Cache as MokaCache;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::{config::parser::Config, results::aggregation_models::SearchResults};

use super::error::PoolError;
#[cfg(feature = "redis")]
use super::redis_cacher::RedisCache;

/// Different implementations for caching, currently it is possible to cache in-memory or in Redis.
#[derive(Clone)]
pub enum Cache {
    /// Caching is disabled
    Disabled,
    #[cfg(feature = "redis")]
    /// Encapsulates the Redis based cache
    Redis(RedisCache),
    #[cfg(feature = "in_memory_cache")]
    /// Contains the in-memory cache.
    InMemory(MokaCache<String, SearchResults>),
}

impl Cache {
    /// Builds the cache from the given configuration.
    pub async fn build(config: &Config) -> Self {
        #[cfg(feature = "redis")]
        if let Some(url) = &config.redis_url {
            log::info!("Using Redis running at {} for caching", &url);
            return Cache::new(
                RedisCache::new(url, 5)
                    .await
                    .expect("Redis cache configured"),
            );
        }
        #[cfg(feature = "in_memory_cache")]
        if config.in_memory_cache {
            log::info!("Using an in-memory cache");
            return Cache::new_in_memory();
        }
        log::info!("Caching is disabled");
        Cache::Disabled
    }

    /// Creates a new cache, which wraps the given RedisCache.
    #[cfg(feature = "redis")]
    pub fn new(redis_cache: RedisCache) -> Self {
        Cache::Redis(redis_cache)
    }

    /// Creates an in-memory cache
    #[cfg(feature = "in_memory_cache")]
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
    pub async fn cached_json(&mut self, url: &str) -> Result<SearchResults, Report<PoolError>> {
        match self {
            Cache::Disabled => Err(Report::new(PoolError::MissingValue)),
            #[cfg(feature = "redis")]
            Cache::Redis(redis_cache) => {
                let json = redis_cache.cached_json(url).await?;
                Ok(serde_json::from_str::<SearchResults>(&json)
                    .map_err(|_| PoolError::SerializationError)?)
            }
            #[cfg(feature = "in_memory_cache")]
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
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<PoolError>> {
        match self {
            Cache::Disabled => Ok(()),
            #[cfg(feature = "redis")]
            Cache::Redis(redis_cache) => {
                let json = serde_json::to_string(search_results)
                    .map_err(|_| PoolError::SerializationError)?;
                redis_cache.cache_results(&json, url).await
            }
            #[cfg(feature = "in_memory_cache")]
            Cache::InMemory(cache) => {
                cache.insert(url.to_string(), search_results.clone());
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

    /// A function which retrieves the cached SearchResulsts from the internal cache.
    pub async fn cached_json(&self, url: &str) -> Result<SearchResults, Report<PoolError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cached_json(url).await
    }

    /// A function which caches the results by using the `url` as the key and
    /// `SearchResults` as the value.
    pub async fn cache_results(
        &self,
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<PoolError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cache_results(search_results, url).await
    }
}
