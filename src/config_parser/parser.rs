//! This module provides the functionality to parse the lua config and convert the config options
//! into rust readable form.

use super::parser_models::Style;
use rlua::Lua;
use std::{format, fs, path::Path};

// ------- Constants --------
static COMMON_DIRECTORY_NAME: &str = "websurfx";
static CONFIG_FILE_NAME: &str = "config.lua";

/// A named struct which stores the parsed config file options.
///
/// # Fields
//
/// * `port` - It stores the parsed port number option on which the server should launch.
/// * `binding_ip_addr` - It stores the parsed ip address option on which the server should launch
/// * `style` - It stores the theming options for the website.
/// * `redis_connection_url` - It stores the redis connection url address on which the redis
/// client should connect.
#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub binding_ip_addr: String,
    pub style: Style,
    pub redis_connection_url: String,
    pub aggregator: AggreatorConfig,
    pub logging: bool,
    pub debug: bool,
}

/// Configuration options for the aggregator.
#[derive(Clone)]
pub struct AggreatorConfig {
    /// Whether to introduce a random delay before sending the request to the search engine.
    pub random_delay: bool,
}

impl Config {
    /// A function which parses the config.lua file and puts all the parsed options in the newly
    /// contructed Config struct and returns it.
    ///
    /// # Error
    ///
    /// Returns a lua parse error if parsing of the config.lua file fails or has a syntax error
    /// or io error if the config.lua file doesn't exists otherwise it returns a newly contructed
    /// Config struct with all the parsed config options from the parsed config file.
    pub fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        Lua::new().context(|context| -> Result<Self, Box<dyn std::error::Error>> {
            let globals = context.globals();

            context
                .load(&fs::read_to_string(
                    Config::handle_different_config_file_path()?,
                )?)
                .exec()?;

            let production_use = globals.get::<_, bool>("production_use")?;
            let aggregator_config = if production_use {
                AggreatorConfig { random_delay: true }
            } else {
                AggreatorConfig {
                    random_delay: false,
                }
            };

            Ok(Config {
                port: globals.get::<_, u16>("port")?,
                binding_ip_addr: globals.get::<_, String>("binding_ip_addr")?,
                style: Style::new(
                    globals.get::<_, String>("theme")?,
                    globals.get::<_, String>("colorscheme")?,
                ),
                redis_connection_url: globals.get::<_, String>("redis_connection_url")?,
                aggregator: aggregator_config,
                logging: globals.get::<_, bool>("logging")?,
                debug: globals.get::<_, bool>("debug")?,
            })
        })
    }
    /// A helper function which returns an appropriate config file path checking if the config
    /// file exists on that path.
    ///
    /// # Error
    ///
    /// Returns a `config file not found!!` error if the config file is not present under following
    /// paths which are:
    /// 1. `~/.config/websurfx/` if it not present here then it fallbacks to the next one (2)
    /// 2. `/etc/xdg/websurfx/config.lua` if it is not present here then it fallbacks to the next
    ///    one (3).
    /// 3. `websurfx/` (under project folder ( or codebase in other words)) if it is not present
    ///    here then it returns an error as mentioned above.
    fn handle_different_config_file_path() -> Result<String, Box<dyn std::error::Error>> {
        if Path::new(
            format!(
                "{}/.config/{}/config.lua",
                std::env::var("HOME").unwrap(),
                COMMON_DIRECTORY_NAME
            )
            .as_str(),
        )
        .exists()
        {
            Ok(format!(
                "{}/.config/{}/{}",
                std::env::var("HOME").unwrap(),
                COMMON_DIRECTORY_NAME,
                CONFIG_FILE_NAME
            ))
        } else if Path::new(
            format!("/etc/xdg/{}/{}", COMMON_DIRECTORY_NAME, CONFIG_FILE_NAME).as_str(),
        )
        .exists()
        {
            Ok("/etc/xdg/websurfx/config.lua".to_string())
        } else if Path::new(format!("./{}/{}", COMMON_DIRECTORY_NAME, CONFIG_FILE_NAME).as_str())
            .exists()
        {
            Ok("./websurfx/config.lua".to_string())
        } else {
            Err(format!("Config file not found!!").into())
        }
    }
}
