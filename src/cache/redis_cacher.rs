//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use error_stack::Report;
use futures::future::try_join_all;
use md5::compute;
use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};

use super::error::CacheError;

/// A named struct which stores the redis Connection url address to which the client will
/// connect to.
#[derive(Clone)]
pub struct RedisCache {
    /// It stores a pool of connections ready to be used.
    connection_pool: Vec<ConnectionManager>,
    /// It stores the size of the connection pool (in other words the number of
    /// connections that should be stored in the pool).
    pool_size: u8,
    /// It stores the index of which connection is being used at the moment.
    current_connection: u8,
}

impl RedisCache {
    /// A function which fetches the cached json results as json string.
    ///
    /// # Arguments
    ///
    /// * `redis_connection_url` - It takes the redis Connection url address.
    /// * `pool_size` - It takes the size of the connection pool (in other words the number of
    /// connections that should be stored in the pool).
    ///
    /// # Error
    ///
    /// Returns a newly constructed `RedisCache` struct on success otherwise returns a standard
    /// error type.
    pub async fn new(
        redis_connection_url: &str,
        pool_size: u8,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_connection_url)?;
        let mut tasks: Vec<_> = Vec::new();

        for _ in 0..pool_size {
            tasks.push(client.get_tokio_connection_manager());
        }

        let redis_cache = RedisCache {
            connection_pool: try_join_all(tasks).await?,
            pool_size,
            current_connection: Default::default(),
        };
        Ok(redis_cache)
    }

    /// A helper function which computes the hash of the url and formats and returns it as string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as string.
    fn hash_url(&self, url: &str) -> String {
        format!("{:?}", compute(url))
    }

    /// A function which fetches the cached json results as json string from the redis server.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as a string.
    ///
    /// # Error
    ///
    /// Returns the results as a String from the cache on success otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cached_json(&mut self, url: &str) -> Result<String, Report<CacheError>> {
        self.current_connection = Default::default();
        let hashed_url_string: &str = &self.hash_url(url);

        let mut result: Result<String, RedisError> = self.connection_pool
            [self.current_connection as usize]
            .get(hashed_url_string)
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
                            .get(hashed_url_string)
                            .await;
                        continue;
                    }
                    false => return Err(Report::new(CacheError::RedisError(error))),
                },
                Ok(res) => return Ok(res),
            }
        }
    }

    /// A function which caches the results by using the hashed `url` as the key and
    /// `json results` as the value and stores it in redis server with ttl(time to live)
    /// set to 60 seconds.
    ///
    /// # Arguments
    ///
    /// * `json_results` - It takes the json results string as an argument.
    /// * `url` - It takes the url as a String.
    ///
    /// # Error
    ///
    /// Returns an unit type if the results are cached succesfully otherwise returns a `CacheError`
    /// on a failure.
    pub async fn cache_results(
        &mut self,
        json_results: &str,
        url: &str,
    ) -> Result<(), Report<CacheError>> {
        self.current_connection = Default::default();
        let hashed_url_string: &str = &self.hash_url(url);

        let mut result: Result<(), RedisError> = self.connection_pool
            [self.current_connection as usize]
            .set_ex(hashed_url_string, json_results, 60)
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
                            .set_ex(hashed_url_string, json_results, 60)
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
