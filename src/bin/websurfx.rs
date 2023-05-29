//! Main module of the application
//!
//! This module contains the main function which handles the logging of the application to the
//! stdout and handles the command line arguments provided and launches the `websurfx` server.

use std::net::TcpListener;

use websurfx::{config_parser::parser::Config, run};

/// The function that launches the main server and registers all the routes of the website.
///
/// # Error
///
/// Returns an error if the port is being used by something else on the system and is not
/// available for being used for other applications.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the parsed config file.
    let config = Config::parse().unwrap();

    // Initializing logging middleware with level set to default or info.
    if config.logging || config.debug {
        use env_logger::Env;
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    }

    log::info!("started server on port {}", config.port);

    let listener = TcpListener::bind((config.binding_ip_addr.clone(), config.port))?;

    run(listener, config)?.await
}
