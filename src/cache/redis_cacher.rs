//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use super::error::CacheError;
use error_stack::Report;
use futures::stream::FuturesUnordered;
use redis::{
    aio::ConnectionManager, AsyncCommands, Client, ExistenceCheck, RedisError, SetExpiry,
    SetOptions,
};

/// A constant holding the redis pipeline size.
const REDIS_PIPELINE_SIZE: usize = 3;

/// A named struct which stores the redis Connection url address to which the client will
/// connect to.
pub struct RedisCache {
    /// It stores a pool of connections ready to be used.
    connection_pool: Box<[ConnectionManager]>,
    /// It stores the size of the connection pool (in other words the number of
    /// connections that should be stored in the pool).
    pool_size: u8,
    /// It stores the index of which connection is being used at the moment.
    current_connection: u8,
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
    /// connections that should be stored in the pool).
    /// * `cache_ttl` - It takes the the time to live for cached results to live in the redis
    /// server.
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
        let tasks: FuturesUnordered<_> = FuturesUnordered::new();

        for _ in 0..pool_size {
            let client_partially_cloned = client.clone();
            tasks.push(tokio::spawn(async move {
                client_partially_cloned.get_connection_manager().await
            }));
        }

        let mut outputs = Vec::with_capacity(tasks.len());
        for task in tasks {
            outputs.push(task.await??);
        }

        let redis_cache = RedisCache {
            connection_pool: outputs.into_boxed_slice(),
            pool_size,
            current_connection: Default::default(),
            cache_ttl,
            pipeline: redis::Pipeline::with_capacity(REDIS_PIPELINE_SIZE),
        };

        Ok(redis_cache)
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
    pub async fn cached_json(&mut self, key: &str) -> Result<String, Report<CacheError>> {
        self.current_connection = Default::default();

        let mut result: Result<String, RedisError> = self.connection_pool
            [self.current_connection as usize]
            .get(key)
            .await;

        // Code to check whether the current connection being used is dropped with connection error
        // or not. if it drops with the connection error then the current connection is replaced
        // with a new connection from the pool which is then used to run the redis command then
        // that connection is also checked whether it is dropped or not if it is not then the
        // result is passed as a `Result` or else the same process repeats again and if all of the
        // connections in the pool result in connection drop error then a custom pool error is
        // returned.
        loop {
            match result {
                Err(error) => match error.is_connection_dropped() {
                    true => {
                        self.current_connection += 1;
                        if self.current_connection == self.pool_size {
                            return Err(Report::new(
                                CacheError::PoolExhaustionWithConnectionDropError,
                            ));
                        }
                        result = self.connection_pool[self.current_connection as usize]
                            .get(key)
                            .await;
                        continue;
                    }
                    false => return Err(Report::new(CacheError::RedisError(error))),
                },
                Ok(res) => return Ok(res),
            }
        }
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
    ) -> Result<(), Report<CacheError>> {
        self.current_connection = Default::default();

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

        let mut result: Result<(), RedisError> = self
            .pipeline
            .query_async(&mut self.connection_pool[self.current_connection as usize])
            .await;

        // Code to check whether the current connection being used is dropped with connection error
        // or not. if it drops with the connection error then the current connection is replaced
        // with a new connection from the pool which is then used to run the redis command then
        // that connection is also checked whether it is dropped or not if it is not then the
        // result is passed as a `Result` or else the same process repeats again and if all of the
        // connections in the pool result in connection drop error then a custom pool error is
        // returned.
        loop {
            match result {
                Err(error) => match error.is_connection_dropped() {
                    true => {
                        self.current_connection += 1;
                        if self.current_connection == self.pool_size {
                            return Err(Report::new(
                                CacheError::PoolExhaustionWithConnectionDropError,
                            ));
                        }
                        result = self
                            .pipeline
                            .query_async(
                                &mut self.connection_pool[self.current_connection as usize],
                            )
                            .await;
                        continue;
                    }
                    false => return Err(Report::new(CacheError::RedisError(error))),
                },
                Ok(_) => return Ok(()),
            }
        }
    }
}
