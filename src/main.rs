//! Main module of the application
//!
//! This module contains the main function which handles the logging of the application to the
//! stdout and handles the command line arguments provided and launches the `websurfx` server.
#[cfg(not(feature = "dhat-heap"))]
use mimalloc::MiMalloc;

use tokio::net::TcpListener;
use websurfx::{parser::Config, serve};

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
#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // A dhat heap profiler initialization.
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let config = Config::parse(false)
        .await
        .map_err(|e| tokio::io::Error::other(e.to_string()))?;

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

    let listener = TcpListener::bind((config.binding_ip.as_str(), config.port)).await?;

    serve(listener, config).await
}
