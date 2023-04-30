//! This module provides the functionality to parse the lua config and convert the config options
//! into rust readable form.

use super::parser_models::Style;
use rlua::Lua;
use std::fs;

/// A named struct which stores the parsed config file options.
///
/// # Fields
//
/// * `port` - It stores the parsed port number option on which the server should launch.
/// * `binding_ip_addr` - It stores the parsed ip address option on which the server should launch
#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub binding_ip_addr: String,
    pub style: Style,
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
        let lua = Lua::new();

        lua.context(|context| {
            let globals = context.globals();

            context
                .load(&fs::read_to_string("./websurfx/config.lua")?)
                .exec()?;

            Ok(Config {
                port: globals.get::<_, u16>("port")?,
                binding_ip_addr: globals.get::<_, String>("binding_ip_addr")?,
                style: Style::new(
                    globals.get::<_, String>("theme")?,
                    globals.get::<_, String>("colorscheme")?,
                ),
            })
        })
    }
}
