//! Main module of the application
//!
//! This module contains the main function which handles the logging of the application to the
//! stdout and handles the command line arguments provided and launches the `websurfx` server.

use mimalloc::MiMalloc;
use std::net::TcpListener;
use websurfx::{cache::cacher::Cache, config::parser::Config, run};

/// A dhat heap memory profiler
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// The function that launches the main server and registers all the routes of the website.
///
/// # Error
///
/// Returns an error if the port is being used by something else on the system and is not
/// available for being used for other applications.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // A dhat heap profiler initialization.
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Initialize the parsed config file.
    let config = Config::parse(false).unwrap();

    let cache = Cache::build(&config).await;

    log::info!(
        "started server on port {} and IP {}",
        config.port,
        config.binding_ip
    );
    log::info!(
        "Open http://{}:{}/ in your browser",
        config.binding_ip,
        config.port,
    );

    let listener = TcpListener::bind((config.binding_ip.clone(), config.port))?;

    run(listener, config, cache)?.await
}
