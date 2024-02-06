#![allow(missing_docs)]

use serde::Deserialize;

/// Stores configurations related to caching.
#[derive(Clone, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Caching {
    /// The expiry time of the search results from the cache (in seconds).
    #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
    pub cache_expiry_time: u16,
    /// The URI to the redis server to use for caching.
    #[cfg(feature = "redis-cache")]
    pub redis_url: String,
}

impl Default for Caching {
    fn default() -> Self {
        Caching {
            #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
            cache_expiry_time: 600,
            #[cfg(feature = "redis-cache")]
            redis_url: "redis://127.0.0.1:8082",
        }
    }
}
