//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use super::{
    Cacher,
    error::{CacheError, CacheResult},
};
use crate::models::aggregation::SearchResults;
use crate::parser::Config;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use redis::{AsyncCommands, Client, ExistenceCheck, SetExpiry, SetOptions, aio::ConnectionManager};
use tokio::task::JoinSet;

/// A constant holding the redis pipeline size.
const REDIS_PIPELINE_SIZE: usize = 3;

/// A named struct which stores the redis Connection url address to which the client will
/// connect to.
#[derive(Clone)]
pub struct RedisCache {
    /// It stores a pool of connections ready to be used.
    connection_pool: Box<[ConnectionManager]>,
    /// It stores the max TTL for keys.
    cache_ttl: u16,
    /// It stores the redis pipeline struct of size 3.
    pipeline: redis::Pipeline,
}

impl RedisCache {
    /// A function which fetches the cached json results as json string.
    ///
    /// # Arguments
    ///
    /// * `redis_connection_url` - It takes the redis Connection url address.
    /// * `pool_size` - It takes the size of the connection pool (in other words the number of
    ///   connections that should be stored in the pool).
    /// * `cache_ttl` - It takes the the time to live for cached results to live in the redis
    ///   server.
    ///
    /// # Error
    ///
    /// Returns a newly constructed `RedisCache` struct on success otherwise returns a standard
    /// error type.
    pub async fn new(
        redis_connection_url: &str,
        pool_size: u8,
        cache_ttl: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_connection_url)?;
        let mut tasks: JoinSet<_> = JoinSet::new();

        for _ in 0..pool_size {
            let client_partially_cloned = client.clone();
            tasks.spawn(async move { client_partially_cloned.get_connection_manager().await });
        }

        let mut outputs = Vec::with_capacity(tasks.len());
        while let Some(task) = tasks.join_next().await {
            outputs.push(task??);
        }

        let redis_cache = RedisCache {
            connection_pool: outputs.into_boxed_slice(),
            cache_ttl,
            pipeline: redis::Pipeline::with_capacity(REDIS_PIPELINE_SIZE),
        };

        Ok(redis_cache)
    }

    /// A helper function which checks each connection in the pool to verify if the connections are
    /// still usable (basically have they not been dropped or disconnected from the server) or not
    /// and if they are usuable then it returns that connection from the pool.
    ///
    /// # Error
    ///
    /// It returns the connection it is usable otherwise it returns an error if all connections in
    /// the pool are not usuable.
    async fn connection(&self) -> CacheResult<ConnectionManager> {
        for mut conn in self.connection_pool.clone() {
            if conn.ping::<String>().await.is_ok() {
                return Ok(conn);
            }
        }

        Err(CacheError::PoolExhaustionWithConnectionDropError)
    }

    /// A function which checks whether the cached value exists or not.
    ///
    /// # Arguments
    ///  
    /// * `key` - It takes a string as key.
    ///
    /// # Error
    ///
    /// Returns the json as a String from the cache on success otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cached_json_exists(&mut self, keys: &[String]) -> CacheResult<Vec<bool>> {
        for key in keys {
            self.pipeline.exists(key);
        }

        self.pipeline
            .query_async(&mut self.connection().await?)
            .await
            .map_err(CacheError::RedisError)
    }

    /// A function which fetches the cached json as json string from the redis server.
    ///
    /// # Arguments
    ///
    /// * `key` - It takes a string as key.
    ///
    /// # Error
    ///
    /// Returns the json as a String from the cache on success otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cached_json(&mut self, key: &str) -> CacheResult<String> {
        self.connection()
            .await?
            .get(key)
            .await
            .map_err(CacheError::RedisError)
    }

    /// A function which caches the json by using the key and
    /// `json results` as the value and stores it in redis server with ttl(time to live)
    /// set to 60 seconds.
    ///
    /// # Arguments
    ///
    /// * `json_results` - It takes the json results string as an argument.
    /// * `key` - It takes the key as a String.
    ///
    /// # Error
    ///
    /// Returns an unit type if the results are cached succesfully otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cache_json(
        &mut self,
        json_results: impl Iterator<Item = String>,
        keys: impl Iterator<Item = String>,
    ) -> CacheResult<()> {
        for (key, json_result) in keys.zip(json_results) {
            self.pipeline.set_options(
                key,
                json_result,
                SetOptions::default()
                    .conditional_set(ExistenceCheck::NX)
                    .get(true)
                    .with_expiration(SetExpiry::EX(self.cache_ttl.into())),
            );
        }

        self.pipeline
            .query_async(&mut self.connection().await?)
            .await
            .map_err(CacheError::RedisError)
    }
}

#[async_trait::async_trait]
impl Cacher for RedisCache {
    async fn build(config: &Config) -> Self {
        log::info!(
            "Initialising redis cache. Listening to {}",
            &config.redis_url
        );
        RedisCache::new(&config.redis_url, 5, config.cache_expiry_time)
            .await
            .expect("Redis cache configured")
    }

    async fn cached_results_exists(&mut self, urls: &[String]) -> CacheResult<Vec<bool>> {
        Ok(self.cached_json_exists(urls).await?)
    }

    async fn cached_results(&mut self, url: &str) -> CacheResult<SearchResults> {
        use base64::Engine;
        let base64_string = self.cached_json(url).await?;

        let bytes = tokio::task::spawn_blocking(|| {
            base64::engine::general_purpose::STANDARD_NO_PAD.decode(base64_string)
        })
        .await
        .map_err(|_| CacheError::Base64DecodingOrEncodingError)?
        .map_err(|_| CacheError::Base64DecodingOrEncodingError)?;

        self.post_process_search_results(bytes).await
    }

    async fn cache_results(
        &mut self,
        search_results: &[SearchResults],
        urls: &[String],
    ) -> CacheResult<()> {
        use base64::Engine;

        // size of search_results is expected to be equal to size of urls -> key/value pairs  for cache;
        let search_results_len = search_results.len();

        let mut bytes = Vec::with_capacity(search_results_len);

        for result in search_results {
            let processed = self.pre_process_search_results(result).await?;
            bytes.push(processed);
        }

        let base64_strings = tokio::task::spawn_blocking(move || {
            bytes
                .par_iter()
                .map(|bytes_vec| base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes_vec))
                .collect::<Box<[String]>>()
        })
        .await
        .map_err(|_| CacheError::Base64DecodingOrEncodingError)?;

        self.cache_json(base64_strings.iter().cloned(), urls.iter().cloned())
            .await
    }
}
