#![allow(missing_docs)]

use serde::Deserialize;
use std::num::NonZeroU16;

/// Configuration options for the server.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Server {
    /// Whether to create logs.
    pub logging: bool,
    /// Whether to use debug mode.
    pub debug: bool,
    /// The amount of threads to utilize.
    pub threads: NonZeroU16,
    /// The port websurfx will listen on.
    pub port: NonZeroU16,
    /// The IP address websurfx will listen on.
    pub binding_ip: String,
    pub aggregator: Aggregator,
    /// Timeout for the search requests sent to the upstream search engines (in seconds).
    pub request_timeout: u8,
    pub rate_limiter: RateLimiter,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            logging: true,
            debug: false,
            threads: NonZeroU16::new(10).unwrap(),
            port: NonZeroU16::new(8080).unwrap(),
            binding_ip: "127.0.0.1".to_string(),
            aggregator: Aggregator::default(),
            request_timeout: 30,
            rate_limiter: RateLimiter::default(),
        }
    }
}

/// Configuration options for the aggregator.
#[derive(Clone, Deserialize, Default, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Aggregator {
    /// Whether to use a random_delay for the aggregator.
    /// Enabling this option is recommended for instances with multiple users.
    /// This setting will add a random delay before sending the request to the search engines,
    /// this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests.
    pub random_delay: bool,
}

/// Configuration options for the rate limiter middleware.
#[derive(Clone, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct RateLimiter {
    /// The number of request that are allowed within the provided time limit.
    pub number_of_requests: u8,
    /// The time limit in which the quantity of requests that should be accepted.
    pub time_limit: u8,
}

impl Default for RateLimiter {
    fn default() -> Self {
        RateLimiter {
            number_of_requests: 20,
            time_limit: 3,
        }
    }
}
