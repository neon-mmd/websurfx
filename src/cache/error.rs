//! This module provides the error enum to handle different errors associated while requesting data from
//! the redis server using an async connection pool.
use std::fmt;

#[cfg(feature = "redis-cache")]
use redis::RedisError;

/// A custom error type used for handling redis async pool associated errors.
#[derive(Debug)]
pub enum CacheError {
    /// This variant handles all errors related to `RedisError`,
    #[cfg(feature = "redis-cache")]
    RedisError(RedisError),
    /// This variant handles the errors which occurs when all the connections
    /// in the connection pool return a connection dropped redis error.
    PoolExhaustionWithConnectionDropError,
    /// Whenever serialization or deserialization fails during communication with the cache.
    SerializationError,
    /// Returned when the value is missing.
    MissingValue,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "redis-cache")]
            CacheError::RedisError(redis_error) => {
                if let Some(detail) = redis_error.detail() {
                    write!(f, "{}", detail)
                } else {
                    write!(f, "")
                }
            }
            CacheError::PoolExhaustionWithConnectionDropError => {
                write!(
                    f,
                    "Error all connections from the pool dropped with connection error"
                )
            }
            CacheError::MissingValue => {
                write!(f, "The value is missing from the cache")
            }
            CacheError::SerializationError => {
                write!(f, "Unable to serialize, deserialize from the cache")
            }
        }
    }
}

impl error_stack::Context for CacheError {}
