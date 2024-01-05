//! This module provides the modules which provide the functionality to cache the aggregated
//! results fetched and aggregated from the upstream search engines in a json format.
pub mod cacher;
#[cfg(any(feature = "encrypt-cache-results", feature = "cec-cache-results"))]
pub mod encryption;
pub mod error;

#[cfg(feature = "redis-cache")]
pub mod redis_cacher;
