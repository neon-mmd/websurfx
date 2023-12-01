//! This module provides public models for handling, storing and serializing parsed config file
//! options from config.lua by grouping them together.

use std::collections::HashMap;

/// A named struct which stores,deserializes, serializes and groups the parsed config file options
/// of theme and colorscheme names into the Style struct which derives the `Clone`, `Serialize`
/// and Deserialize traits where the `Clone` trait is derived for allowing the struct to be
/// cloned and passed to the server as a shared data between all routes except `/robots.txt` and
/// the `Serialize` trait has been derived for allowing the object to be serialized so that it
/// can be passed to handlebars template files and the `Deserialize` trait has been derived in
/// order to allow the deserializing the json back to struct in aggregate function in
/// aggregator.rs and create a new struct out of it and then serialize it back to json and pass
/// it to the template files.
#[derive(Clone, Default)]
pub struct Style {
    /// It stores the parsed theme option used to set a theme for the website.
    pub theme: String,
    /// It stores the parsed colorscheme option used to set a colorscheme for the
    /// theme being used.
    pub colorscheme: String,
}

impl Style {
    /// Constructs a new `Style` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `theme` - It takes the parsed theme option used to set a theme for the website.
    /// * `colorscheme` - It takes the parsed colorscheme option used to set a colorscheme
    /// for the theme being used.
    pub fn new(theme: String, colorscheme: String) -> Self {
        Style { theme, colorscheme }
    }
}

/// Configuration options for the aggregator.
#[derive(Clone)]
pub struct AggregatorConfig {
    /// It stores the option to whether enable or disable random delays between
    /// requests.
    pub random_delay: bool,
}

/// Configuration options for the rate limiter middleware.
#[derive(Clone)]
pub struct RateLimiter {
    /// The number of request that are allowed within a provided time limit.
    pub number_of_requests: u8,
    /// The time limit in which the quantity of requests that should be accepted.
    pub time_limit: u8,
}

#[derive(Clone)]
pub struct RequestClientConfig {
    pub proxy_url: Option<String>,
    pub is_tor_proxy: Option<bool>,
    pub use_http2: bool,
    pub https_only: bool,
    pub timeout: u8,
    pub max_retries: u8,
    pub max_redirects: u8,
}

impl TryFrom<HashMap<String, String>> for RequestClientConfig {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let bool_parse = |s| match s {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(()),
        };

        let is_tor_proxy = value.get("is_tor_proxy").map(|s| {
            bool_parse(&s).unwrap_or_else(|_| {
                log::error!("Config Error: The value of `is_tor_proxy` option should be a boolean");
                log::error!("Falling back to using the value `false` for the option");
                false
            })
        });

        let https_only = match value.get("https_only") {
            Some(v) => bool_parse(v).unwrap_or_else(|_| {
                log::error!("Config Error: The value of https_only` option should be a boolean.");
                log::error!("Falling back to using the value `true` for the option");
                true
            }),
            None => {
                log::error!("Config Error: The value of https_only` option is not set");
                log::error!("Falling back to using the value `true` for the option");
                true
            }
        };

        Ok(RequestClientConfig {
                    proxy_url: value.get("proxy_url").and_then(|s| Some(s.clone())),
                    is_tor_proxy,
                    https_only,
                    use_http2: value["use_http2"].parse()?,
                    timeout: value["timeout"].parse()?,
                    max_retries: value["max_retries"].parse()?,
                    max_redirects: value["max_redirects"].parse()?,
                })
    }

}
