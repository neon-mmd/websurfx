//! This module provides the error enum to handle different errors associated while requesting data from
//! the redis server using an async connection pool.
use std::fmt;

use redis::RedisError;

/// A custom error type used for handling redis async pool associated errors.
///
/// This enum provides variants three different categories of errors:
/// * `RedisError` - This variant handles all errors related to `RedisError`,
/// * `PoolExhaustionWithConnectionDropError` - This variant handles the error
/// which occurs when all the connections in the connection pool return a connection
/// dropped redis error.
#[derive(Debug)]
pub enum PoolError {
    RedisError(RedisError),
    PoolExhaustionWithConnectionDropError,
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
        }
    }
}

impl error_stack::Context for PoolError {}
