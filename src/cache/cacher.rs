//! This module provides the functionality to cache the aggregated results fetched and aggregated
//! from the upstream search engines in a json format.

use md5::compute;
use redis::{Client, Commands, Connection};

/// A named struct which stores the redis Connection url address to which the client will
/// connect to.
///
/// # Fields
///
/// * `redis_connection_url` - It stores the redis Connection url address.
#[derive(Clone)]
pub struct RedisCache {
    redis_connection_url: String,
}

impl RedisCache {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `redis_connection_url` - It stores the redis Connection url address.
    pub fn new(redis_connection_url: String) -> Self {
        RedisCache {
            redis_connection_url,
        }
    }

    /// A helper function which computes the hash of the url and formats and returns it as string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as string.
    fn compute_url_hash(self, url: &str) -> String {
        format!("{:?}", compute(url))
    }

    /// A function which fetches the cached json results as json string from the redis server.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as a string.
    pub fn cached_results_json(self, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let hashed_url_string = self.clone().compute_url_hash(&url);
        let mut redis_connection: Connection =
            Client::open(self.redis_connection_url)?.get_connection()?;
        Ok(redis_connection.get(hashed_url_string)?)
    }

    /// A function which caches the results by using the hashed `url` as the key and
    /// `json results` as the value and stores it in redis server with ttl(time to live)
    /// set to 60 seconds.
    ///
    /// # Arguments
    ///
    /// * `json_results` - It takes the json results string as an argument.
    /// * `url` - It takes the url as a String.
    pub fn cache_results(
        self,
        json_results: String,
        url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hashed_url_string = self.clone().compute_url_hash(&url);
        let mut redis_connection: Connection =
            Client::open(self.redis_connection_url)?.get_connection()?;

        // put results_json into cache
        redis_connection.set(hashed_url_string.clone(), json_results)?;

        // Set the TTL for the key to 60 seconds
        redis_connection
            .expire::<String, u32>(hashed_url_string.clone(), 60)
            .unwrap();

        Ok(())
    }
}
