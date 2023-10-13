//! This module provides the functionality to parse the lua config and convert the config options
//! into rust readable form.

use crate::handler::paths::{file_path, FileType};

use crate::models::engine_models::{EngineError, EngineHandler};
use crate::models::parser_models::{AggregatorConfig, RateLimiter, Style};
use log::LevelFilter;
use mlua::Lua;
use std::{collections::HashMap, fs, thread::available_parallelism};

/// A named struct which stores the parsed config file options.
#[derive(Clone)]
pub struct Config {
    /// It stores the parsed port number option on which the server should launch.
    pub port: u16,
    /// It stores the parsed ip address option on which the server should launch
    pub binding_ip: String,
    /// It stores the theming options for the website.
    pub style: Style,
    #[cfg(feature = "redis-cache")]
    /// It stores the redis connection url address on which the redis
    /// client should connect.
    pub redis_url: String,
    /// It stores the option to whether enable or disable production use.
    pub aggregator: AggregatorConfig,
    /// It stores the option to whether enable or disable logs.
    pub logging: bool,
    /// It stores the option to whether enable or disable debug mode.
    pub debug: bool,
    /// It stores all the engine names that were enabled by the user.
    pub upstream_search_engines: Vec<EngineHandler>,
    /// It stores the time (secs) which controls the server request timeout.
    pub request_timeout: u8,
    /// It stores the number of threads which controls the app will use to run.
    pub threads: u8,
    /// It stores configuration options for the ratelimiting middleware.
    pub rate_limiter: RateLimiter,
    /// It stores the level of safe search to be used for restricting content in the
    /// search results.
    pub safe_search: u8,
}

impl Config {
    /// A function which parses the config.lua file and puts all the parsed options in the newly
    /// constructed Config struct and returns it.
    ///
    /// # Arguments
    ///
    /// * `logging_initialized` - It takes a boolean which ensures that the logging doesn't get
    /// initialized twice. Pass false if the logger has not yet been initialized.
    ///
    /// # Error
    ///
    /// Returns a lua parse error if parsing of the config.lua file fails or has a syntax error
    /// or io error if the config.lua file doesn't exists otherwise it returns a newly constructed
    /// Config struct with all the parsed config options from the parsed config file.
    pub fn parse(logging_initialized: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let globals = lua.globals();

        lua.load(&fs::read_to_string(file_path(FileType::Config)?)?)
            .exec()?;

        let parsed_threads: u8 = globals.get::<_, u8>("threads")?;

        let debug: bool = globals.get::<_, bool>("debug")?;
        let logging: bool = globals.get::<_, bool>("logging")?;

        if !logging_initialized {
            set_logging_level(debug, logging);
        }

        let threads: u8 = if parsed_threads == 0 {
            let total_num_of_threads: usize = available_parallelism()?.get() / 2;
            log::error!(
                "Config Error: The value of `threads` option should be a non zero positive integer"
            );
            log::error!("Falling back to using {} threads", total_num_of_threads);
            total_num_of_threads as u8
        } else {
            parsed_threads
        };

        let rate_limiter = globals.get::<_, HashMap<String, u8>>("rate_limiter")?;

        let parsed_safe_search: u8 = globals.get::<_, u8>("safe_search")?;
        let safe_search: u8 = match parsed_safe_search {
            0..=4 => parsed_safe_search,
            _ => {
                log::error!("Config Error: The value of `safe_search` option should be a non zero positive integer from 0 to 4.");
                log::error!("Falling back to using the value `1` for the option");
                1
            }
        };

        Ok(Config {
            port: globals.get::<_, u16>("port")?,
            binding_ip: globals.get::<_, String>("binding_ip")?,
            style: Style::new(
                globals.get::<_, String>("theme")?,
                globals.get::<_, String>("colorscheme")?,
            ),
            #[cfg(feature = "redis-cache")]
            redis_url: globals.get::<_, String>("redis_url")?,
            aggregator: AggregatorConfig {
                random_delay: globals.get::<_, bool>("production_use")?,
            },
            logging,
            debug,
            upstream_search_engines: globals
                .get::<_, HashMap<String, bool>>("upstream_search_engines")?
                .into_iter()
                .filter_map(|(key, value)| value.then_some(key))
                .map(|engine| EngineHandler::new(&engine))
                .collect::<Result<Vec<EngineHandler>, error_stack::Report<EngineError>>>()?,
            request_timeout: globals.get::<_, u8>("request_timeout")?,
            threads,
            rate_limiter: RateLimiter {
                number_of_requests: rate_limiter["number_of_requests"],
                time_limit: rate_limiter["time_limit"],
            },
            safe_search,
        })
    }
}

/// a helper function that sets the proper logging level
///
/// # Arguments
///
/// * `debug` - It takes the option to whether enable or disable debug mode.
/// * `logging` - It takes the option to whether enable or disable logs.
fn set_logging_level(debug: bool, logging: bool) {
    if let Ok(pkg_env_var) = std::env::var("PKG_ENV") {
        if pkg_env_var.to_lowercase() == "dev" {
            env_logger::Builder::new()
                .filter(None, LevelFilter::Trace)
                .init();
            return;
        }
    }

    // Initializing logging middleware with level set to default or info.
    let log_level = match (debug, logging) {
        (true, true) => LevelFilter::Debug,
        (true, false) => LevelFilter::Debug,
        (false, true) => LevelFilter::Info,
        (false, false) => LevelFilter::Error,
    };

    env_logger::Builder::new().filter(None, log_level).init();
}
