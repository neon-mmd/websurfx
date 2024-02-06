//! This module provides the functionality to parse the lua config and convert the config options
//! into rust readable form.
use std::fs;

use mlua::{Lua, LuaSerdeExt};

use crate::config::{process_settings, Config};
use crate::handler::{file_path, FileType};

impl Config {
    /// A function which deserializes the config.lua into a Config struct.
    ///
    /// # Arguments
    ///
    /// * `logging_initialized` - It takes a boolean which ensures that the logging doesn't get
    /// initialized twice. Pass false if the logger has not yet been initialized.
    ///
    pub fn parse(logging_initialized: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let config_file = match file_path(FileType::Config) {
            Ok(f) => f,
            Err(_) => {
                log::error!("Config Error: No config file found, falling back to defaults");
                let conf = Self::default();
                if !logging_initialized {
                    conf.set_logging_level();
                }
                return Ok(conf);
            }
        };
        let val = lua.load(fs::read_to_string(config_file)?).eval()?;

        let mut conf: Config = lua.from_value(val)?;

        if !logging_initialized {
            conf.set_logging_level();
        }

        conf = process_settings(conf)?;

        Ok(conf)
    }
}
