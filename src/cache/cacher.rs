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
pub struct RedisCache {
    connection: Connection,
}

impl RedisCache {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `redis_connection_url` - It stores the redis Connection url address.
    pub fn new(redis_connection_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_connection_url)?;
        let connection = client.get_connection()?;
        let redis_cache = RedisCache { connection };
        Ok(redis_cache)
    }

    /// A helper function which computes the hash of the url and formats and returns it as string.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as string.
    fn compute_url_hash(url: &str) -> String {
        format!("{:?}", compute(url))
    }

    /// A function which fetches the cached json results as json string from the redis server.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes an url as a string.
    pub fn cached_results_json(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let hashed_url_string = Self::compute_url_hash(url);
        Ok(self.connection.get(hashed_url_string)?)
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
        &mut self,
        json_results: String,
        url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let hashed_url_string = Self::compute_url_hash(url);

        // put results_json into cache
        self.connection.set(&hashed_url_string, json_results)?;

        // Set the TTL for the key to 60 seconds
        self.connection
            .expire::<String, u32>(hashed_url_string, 60)
            .unwrap();

        Ok(())
    }
}
