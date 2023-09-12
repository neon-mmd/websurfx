//! This module provides the modules which provide the functionality to cache the aggregated
//! results fetched and aggregated from the upstream search engines in a json format.

pub mod cacher;
pub mod error;
#[cfg(feature = "redis")]
pub mod redis_cacher;
