//! This module provides the error enum to handle different errors associated while requesting data from
//! the redis server using an async connection pool.
use std::fmt;

use redis::RedisError;

/// A custom error type used for handling redis async pool associated errors.
#[derive(Debug)]
pub enum PoolError {
    /// This variant handles all errors related to `RedisError`,
    RedisError(RedisError),
    /// This variant handles the errors which occurs when all the connections
    /// in the connection pool return a connection dropped redis error.
    PoolExhaustionWithConnectionDropError,
    SerializationError,
    MissingValue,
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolError::RedisError(redis_error) => {
                if let Some(detail) = redis_error.detail() {
                    write!(f, "{}", detail)
                } else {
                    write!(f, "")
                }
            }
            PoolError::PoolExhaustionWithConnectionDropError => {
                write!(
                    f,
                    "Error all connections from the pool dropped with connection error"
                )
            }
            PoolError::MissingValue => {
                write!(f, "The value is missing from the cache")
            }
            PoolError::SerializationError => {
                write!(f, "Unable to serialize, deserialize from the cache")
            }
        }
    }
}

impl error_stack::Context for PoolError {}
