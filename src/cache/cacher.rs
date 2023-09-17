//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use error_stack::Report;
#[cfg(feature = "memory-cache")]
use mini_moka::sync::Cache as MokaCache;
#[cfg(feature = "memory-cache")]
use std::time::Duration;
use tokio::sync::Mutex;

use crate::{config::parser::Config, models::aggregation_models::SearchResults};

use super::error::CacheError;
#[cfg(feature = "redis-cache")]
use super::redis_cacher::RedisCache;

/// Different implementations for caching, currently it is possible to cache in-memory or in Redis.
#[derive(Clone)]
pub enum Cache {
    /// Caching is disabled
    Disabled,
    #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
    /// Encapsulates the Redis based cache
    Redis(RedisCache),
    #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
    /// Contains the in-memory cache.
    InMemory(MokaCache<String, SearchResults>),
    #[cfg(all(feature = "redis-cache", feature = "memory-cache"))]
    /// Contains both the in-memory cache and Redis based cache
    Hybrid(RedisCache, MokaCache<String, SearchResults>),
}

impl Cache {
    /// A function that builds the cache from the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - It takes the config struct as an argument.
    ///
    /// # Returns
    ///
    /// It returns a newly initialized variant based on the feature enabled by the user.
    pub async fn build(_config: &Config) -> Self {
        #[cfg(all(feature = "redis-cache", feature = "memory-cache"))]
        {
            log::info!("Using a hybrid cache");
            Cache::new_hybrid(
                RedisCache::new(&_config.redis_url, 5)
                    .await
                    .expect("Redis cache configured"),
            )
        }
        #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
        {
            log::info!("Listening redis server on {}", &_config.redis_url);
            Cache::new(
                RedisCache::new(&_config.redis_url, 5)
                    .await
                    .expect("Redis cache configured"),
            )
        }
        #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
        {
            log::info!("Using an in-memory cache");
            Cache::new_in_memory()
        }
        #[cfg(not(any(feature = "memory-cache", feature = "redis-cache")))]
        {
            log::info!("Caching is disabled");
            Cache::Disabled
        }
    }

    /// A function that initializes a new connection pool struct.
    ///
    /// # Arguments
    ///
    /// * `redis_cache` - It takes the newly initialized connection pool struct as an argument.
    ///
    /// # Returns
    ///
    /// It returns a `Redis` variant with the newly initialized connection pool struct.
    #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
    pub fn new(redis_cache: RedisCache) -> Self {
        Cache::Redis(redis_cache)
    }

    /// A function that initializes the `in memory` cache which is used to cache the results in
    /// memory with the search engine thus improving performance by making retrieval and caching of
    /// results faster.
    ///
    /// # Returns
    ///
    /// It returns a `InMemory` variant with the newly initialized in memory cache type.
    #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
    pub fn new_in_memory() -> Self {
        let cache = MokaCache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(60))
            .build();
        Cache::InMemory(cache)
    }

    /// A function that initializes both in memory cache and redis client connection for being used
    /// for managing hybrid cache which increases resiliancy of the search engine by allowing the
    /// cache to switch to `in memory` caching if the `redis` cache server is temporarily
    /// unavailable.
    ///
    /// # Arguments
    ///
    /// * `redis_cache` - It takes `redis` client connection struct as an argument.
    ///
    /// # Returns
    ///
    /// It returns a tuple variant `Hybrid` storing both the in-memory cache type and the `redis`
    /// client connection struct.
    #[cfg(all(feature = "redis-cache", feature = "memory-cache"))]
    pub fn new_hybrid(redis_cache: RedisCache) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(60))
            .build();
        Cache::Hybrid(redis_cache, cache)
    }

    /// A function which fetches the cached json results as json string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as a string.
    ///
    /// # Error
    ///
    /// Returns the `SearchResults` from the cache if the program executes normally otherwise
    /// returns a `CacheError` if the results cannot be retrieved from the cache.
    pub async fn cached_json(&mut self, _url: &str) -> Result<SearchResults, Report<CacheError>> {
        match self {
            Cache::Disabled => Err(Report::new(CacheError::MissingValue)),
            #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
            Cache::Redis(redis_cache) => {
                let json = redis_cache.cached_json(_url).await?;
                Ok(serde_json::from_str::<SearchResults>(&json)
                    .map_err(|_| CacheError::SerializationError)?)
            }
            #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
            Cache::InMemory(in_memory) => match in_memory.get(&_url.to_string()) {
                Some(res) => Ok(res),
                None => Err(Report::new(CacheError::MissingValue)),
            },
            #[cfg(all(feature = "redis-cache", feature = "memory-cache"))]
            Cache::Hybrid(redis_cache, in_memory) => match redis_cache.cached_json(_url).await {
                Ok(res) => Ok(serde_json::from_str::<SearchResults>(&res)
                    .map_err(|_| CacheError::SerializationError)?),
                Err(_) => match in_memory.get(&_url.to_string()) {
                    Some(res) => Ok(res),
                    None => Err(Report::new(CacheError::MissingValue)),
                },
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
    ///
    /// # Error
    ///
    /// Returns a unit type if the program caches the given search results without a failure
    /// otherwise it returns a `CacheError` if the search results cannot be cached due to a
    /// failure.
    pub async fn cache_results(
        &mut self,
        _search_results: &SearchResults,
        _url: &str,
    ) -> Result<(), Report<CacheError>> {
        match self {
            Cache::Disabled => Ok(()),
            #[cfg(all(feature = "redis-cache", not(feature = "memory-cache")))]
            Cache::Redis(redis_cache) => {
                let json = serde_json::to_string(_search_results)
                    .map_err(|_| CacheError::SerializationError)?;
                redis_cache.cache_results(&json, _url).await
            }
            #[cfg(all(feature = "memory-cache", not(feature = "redis-cache")))]
            Cache::InMemory(cache) => {
                cache.insert(_url.to_string(), _search_results.clone());
                Ok(())
            }
            #[cfg(all(feature = "memory-cache", feature = "redis-cache"))]
            Cache::Hybrid(redis_cache, cache) => {
                let json = serde_json::to_string(_search_results)
                    .map_err(|_| CacheError::SerializationError)?;
                match redis_cache.cache_results(&json, _url).await {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        cache.insert(_url.to_string(), _search_results.clone());
                        Ok(())
                    }
                }
            }
        }
    }
}

/// A structure to efficiently share the cache between threads - as it is protected by a Mutex.
pub struct SharedCache {
    /// The internal cache protected from concurrent access by a mutex
    cache: Mutex<Cache>,
}

impl SharedCache {
    /// A function that creates a new `SharedCache` from a Cache implementation.
    ///
    /// # Arguments
    ///
    /// * `cache` - It takes the `Cache` enum variant as an argument with the prefered cache type.
    ///
    /// Returns a newly constructed `SharedCache` struct.
    pub fn new(cache: Cache) -> Self {
        Self {
            cache: Mutex::new(cache),
        }
    }

    /// A getter function which retrieves the cached SearchResulsts from the internal cache.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the search url as an argument which will be used as the key to fetch the
    /// cached results from the cache.
    ///
    /// # Error
    ///
    /// Returns a `SearchResults` struct containing the search results from the cache if nothing
    /// goes wrong otherwise returns a `CacheError`.
    pub async fn cached_json(&self, url: &str) -> Result<SearchResults, Report<CacheError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cached_json(url).await
    }

    /// A setter function which caches the results by using the `url` as the key and
    /// `SearchResults` as the value.
    ///
    /// # Arguments
    ///
    /// * `search_results` - It takes the `SearchResults` as an argument which are results that
    /// needs to be cached.
    /// * `url` - It takes the search url as an argument which will be used as the key for storing
    /// results in the cache.
    ///
    /// # Error
    ///
    /// Returns an unit type if the results are cached succesfully otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cache_results(
        &self,
        search_results: &SearchResults,
        url: &str,
    ) -> Result<(), Report<CacheError>> {
        let mut mut_cache = self.cache.lock().await;
        mut_cache.cache_results(search_results, url).await
    }
}
