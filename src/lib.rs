//! This main library module provides the functionality to provide and handle the Tcp server
//! and register all the routes for the `websurfx` meta search engine website.

#![forbid(unsafe_code, clippy::panic)]
#![deny(missing_docs, clippy::missing_docs_in_private_items, clippy::perf)]
#![warn(clippy::cognitive_complexity, rust_2018_idioms)]

pub mod cache;
pub mod config;
pub mod engines;
pub mod handler;
pub mod models;
pub mod results;
pub mod server;
pub mod templates;

use std::{net::TcpListener, sync::OnceLock, time::Duration};

use crate::server::router;

use actix_cors::Cors;
use actix_files as fs;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
    dev::Server,
    http::header,
    middleware::{Compress, Logger},
    web, App, HttpServer,
};
use cache::cacher::{Cacher, SharedCache};
use config::parser::Config;
use handler::{file_path, FileType};

/// A static constant for holding the cache struct.
static SHARED_CACHE: OnceLock<SharedCache> = OnceLock::new();

/// Runs the web server on the provided TCP listener and returns a `Server` instance.
///
/// # Arguments
///
/// * `listener` - A `TcpListener` instance representing the address and port to listen on.
///
/// # Returns
///
/// Returns a `Result` containing a `Server` instance on success, or an `std::io::Error` on failure.
///
/// # Example
///
/// ```rust
/// use std::{net::TcpListener, sync::OnceLock};
/// use websurfx::{config::parser::Config, run, cache::cacher::create_cache};
///
/// /// A static constant for holding the parsed config.
/// static CONFIG: OnceLock<Config> = OnceLock::new();
///
/// #[tokio::main]
/// async fn main(){
///     // Initialize the parsed config globally.
///     let config = CONFIG.get_or_init(|| Config::parse(true).unwrap());
///     let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind address");
///     let cache = create_cache(config).await;
///     let server = run(listener,&config,cache).expect("Failed to start server");
/// }
/// ```
pub fn run(
    listener: TcpListener,
    config: &'static Config,
    cache: impl Cacher + 'static,
) -> std::io::Result<Server> {
    let public_folder_path: &str = file_path(FileType::Theme)?;

    let cache = SHARED_CACHE.get_or_init(|| SharedCache::new(cache));

    let server = HttpServer::new(move || {
        let cors: Cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![
                header::ORIGIN,
                header::CONTENT_TYPE,
                header::REFERER,
                header::COOKIE,
            ]);

        App::new()
            // Compress the responses provided by the server for the client requests.
            .wrap(Compress::default())
            .wrap(Logger::default()) // added logging middleware for logging.
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(cache))
            .wrap(cors)
            .wrap(Governor::new(
                &GovernorConfigBuilder::default()
                    .seconds_per_request(config.rate_limiter.time_limit as u64)
                    .burst_size(config.rate_limiter.number_of_requests as u32)
                    .finish()
                    .unwrap(),
            ))
            // Serve images and static files (css and js files).
            .service(
                fs::Files::new("/static", format!("{}/static", public_folder_path))
                    .show_files_listing(),
            )
            .service(
                fs::Files::new("/images", format!("{}/images", public_folder_path))
                    .show_files_listing(),
            )
            .service(router::robots_data) // robots.txt
            .service(router::index) // index page
            .service(server::routes::search::search) // search page
            .service(router::about) // about page
            .service(router::settings) // settings page
            .default_service(web::route().to(router::not_found)) // error page
    })
    .workers(config.threads as usize)
    // Set the keep-alive timer for client connections
    .keep_alive(Duration::from_secs(
        config.client_connection_keep_alive as u64,
    ))
    // Start server on 127.0.0.1 with the user provided port number. for example 127.0.0.1:8080.
    .listen(listener)?
    .run();
    Ok(server)
}
