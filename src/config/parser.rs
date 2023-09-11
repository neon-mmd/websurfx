//! This module provides the functionality to parse the lua config and convert the config options
//! into rust readable form.

use crate::handler::paths::{file_path, FileType};

use super::parser_models::{AggregatorConfig, RateLimiter, Style};
use log::LevelFilter;
use mlua::Lua;
use std::{collections::HashMap, fs, thread::available_parallelism};

/// A named struct which stores the parsed config file options.
///
/// # Fields
//
/// * `port` - It stores the parsed port number option on which the server should launch.
/// * `binding_ip` - It stores the parsed ip address option on which the server should launch
/// * `style` - It stores the theming options for the website.
/// * `redis_url` - It stores the redis connection url address on which the redis
/// client should connect.
/// * `aggregator` -  It stores the option to whether enable or disable production use.
/// * `logging` - It stores the option to whether enable or disable logs.
/// * `debug` - It stores the option to whether enable or disable debug mode.
/// * `upstream_search_engines` - It stores all the engine names that were enabled by the user.
/// * `request_timeout` - It stores the time (secs) which controls the server request timeout.
/// * `threads` - It stores the number of threads which controls the app will use to run.
#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub binding_ip: String,
    pub style: Style,
    pub redis_url: String,
    pub aggregator: AggregatorConfig,
    pub logging: bool,
    pub debug: bool,
    pub upstream_search_engines: Vec<crate::engines::engine_models::EngineHandler>,
    pub request_timeout: u8,
    pub threads: u8,
    pub rate_limiter: RateLimiter,
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

            let rate_limiter = globals.get::<_,HashMap<String, u8>>("rate_limiter")?;

            Ok(Config {
                port: globals.get::<_, u16>("port")?,
                binding_ip: globals.get::<_, String>("binding_ip")?,
                style: Style::new(
                    globals.get::<_, String>("theme")?,
                    globals.get::<_, String>("colorscheme")?,
                ),
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
                    .filter_map(|engine| crate::engines::engine_models::EngineHandler::new(&engine))
                    .collect(),
                request_timeout: globals.get::<_, u8>("request_timeout")?,
                threads,
                rate_limiter: RateLimiter {
                    number_of_requests: rate_limiter["number_of_requests"], 
                    time_limit: rate_limiter["time_limit"], 
            }
        })
    }
}

/// a helper function that sets the proper logging level
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
