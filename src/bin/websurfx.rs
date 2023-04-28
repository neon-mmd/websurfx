//! Main module of the application
//!
//! This module contains the main function which handles the logging of the application to the
//! stdout and handles the command line arguments provided and launches the `websurfx` server.

use std::{ops::RangeInclusive, net::TcpListener};

use clap::{command, Parser};
use env_logger::Env;
use websurfx::run;

/// A commandline arguments struct.
#[derive(Parser, Debug, Default)]
#[clap(author = "neon_arch", version, about = "Websurfx server application")]
#[command(propagate_version = true)]
struct CliArgs {
    #[clap(default_value_t = 8080, short, long,value_parser = is_port_in_range)]
    /// provide port number in range [1024-65536] to launch the server on.
    port: u16,
}

const PORT_RANGE: RangeInclusive<usize> = 1024..=65535;

/// A function to check whether port is valid u32 number or is in range
/// between [1024-65536] otherwise display an appropriate error message.
///
/// # Arguments
///
/// * `s` - Takes a commandline argument port as a string.
///
/// # Error
///
/// Check whether the provided argument to `--port` commandline option is a valid
/// u16 argument and returns it as a u16 value otherwise returns an error with an
/// appropriate error message.
fn is_port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` is not a valid port number"))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not found in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

/// The function that launches the main server and registers all the routes of the website.
///
/// # Error
///
/// Returns an error if the port is being used by something else on the system and is not
/// available for being used for other applications.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = CliArgs::parse();

    // Initializing logging middleware with level set to default or info.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    log::info!("started server on port {}", args.port);

    let listener = TcpListener::bind(("127.0.0.1", args.port))?;

    run(listener)?.await
}
