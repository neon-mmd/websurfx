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
/// * `binding_ip` - It stores the parsed ip address option on which the server should launch
/// * `style` - It stores the theming options for the website.
/// * `redis_url` - It stores the redis connection url address on which the redis
/// client should connect.
#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub binding_ip: String,
    pub style: Style,
    pub redis_url: String,
    pub aggregator: AggregatorConfig,
    pub logging: bool,
    pub debug: bool,
}

/// Configuration options for the aggregator.
#[derive(Clone)]
pub struct AggregatorConfig {
    /// Whether to introduce a random delay before sending the request to the search engine.
    pub random_delay: bool,
}

impl Config {
    /// A function which parses the config.lua file and puts all the parsed options in the newly
    /// constructed Config struct and returns it.
    ///
    /// # Error
    ///
    /// Returns a lua parse error if parsing of the config.lua file fails or has a syntax error
    /// or io error if the config.lua file doesn't exists otherwise it returns a newly constructed
    /// Config struct with all the parsed config options from the parsed config file.
    pub fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        Lua::new().context(|context| -> Result<Self, Box<dyn std::error::Error>> {
            let globals = context.globals();

            context
                .load(&fs::read_to_string(Config::get_config_path()?)?)
                .exec()?;

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
    fn get_config_path() -> Result<String, Box<dyn std::error::Error>> {
        // check user config

        let path = format!(
            "{}/.config/{}/config.lua",
            std::env::var("HOME").unwrap(),
            COMMON_DIRECTORY_NAME
        );
        if Path::new(path.as_str()).exists() {
            return Ok(format!(
                "{}/.config/{}/{}",
                std::env::var("HOME").unwrap(),
                COMMON_DIRECTORY_NAME,
                CONFIG_FILE_NAME
            ));
        }

        // look for config in /etc/xdg
        if Path::new(format!("/etc/xdg/{}/{}", COMMON_DIRECTORY_NAME, CONFIG_FILE_NAME).as_str())
            .exists()
        {
            return Ok("/etc/xdg/websurfx/config.lua".to_string());
        }

        // use dev config
        if Path::new(format!("./{}/{}", COMMON_DIRECTORY_NAME, CONFIG_FILE_NAME).as_str()).exists()
        {
            return Ok("./websurfx/config.lua".to_string());
        }

        // if no of the configs above exist, return error
        Err("Config file not found!!".to_string().into())
    }
}
