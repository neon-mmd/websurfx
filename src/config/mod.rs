#![allow(missing_docs, clippy::missing_docs_in_private_items)]
//! This module provides the modules which handles the functionality to parse the lua/json config
//! and convert the config options into rust readable form.
use crate::config::{caching::Caching, search::Search, server::Server, style::Style};
use log::LevelFilter;
use serde::Deserialize;

pub mod caching;
pub mod search;
pub mod server;
pub mod style;

/// A named struct which stores the parsed config file options.
#[derive(Clone, Deserialize, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub server: Server,
    pub style: Style,
    pub caching: Caching,
    pub search: Search,
}

impl Config {
    /// a helper function that sets the proper logging level
    fn set_logging_level(&self) {
        if let Ok(pkg_env_var) = std::env::var("PKG_ENV") {
            if pkg_env_var.to_lowercase() == "dev" {
                env_logger::Builder::new()
                    .filter(None, LevelFilter::Trace)
                    .init();
                return;
            }
        }

        // Initializing logging middleware with level set to default or info.
        let log_level = match (self.server.debug, self.server.logging) {
            (true, true) => LevelFilter::Debug,
            (true, false) => LevelFilter::Debug,
            (false, true) => LevelFilter::Info,
            (false, false) => LevelFilter::Error,
        };

        env_logger::Builder::new().filter(None, log_level).init();
    }
}

fn process_settings(mut conf: Config) -> Result<Config, Box<dyn std::error::Error>> {
    conf.search.safe_search = match conf.search.safe_search {
        0..=4 => conf.search.safe_search,
        _ => {
            log::error!("Config Error: The value of `safe_search` option should be a non zero positive integer from 0 to 4.");
            log::error!("Falling back to using the value `1` for the option");
            1
        }
    };

    conf.caching.cache_expiry_time = match conf.caching.cache_expiry_time {
        0..=59 => {
            log::error!("Config Error: The value of `cache_expiry_time` must be greater than 60");
            log::error!("Falling back to using the value `60` for the option");
            60
        }
        _ => conf.caching.cache_expiry_time,
    };

    Ok(conf)
}

#[cfg(feature = "json-config")]
pub mod json;
#[cfg(feature = "lua-config")]
pub mod lua;
