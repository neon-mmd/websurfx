//! This main library module provides the functionality to provide and handle the Tcp server
//! and register all the routes for the `websurfx` meta search engine website.

mod aggregator;
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
mod cache;
mod engines;
mod handler;
mod models;
pub mod parser;
mod routes;
pub mod templates;
mod user_agent;

use actix_cors::Cors;
use actix_files as fs;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
    App, HttpServer,
    dev::Server,
    http::header::{self, CacheControl, CacheDirective},
    middleware::{Compress, DefaultHeaders, Logger},
    web,
};
use handler::{FileType, file_path};
use parser::Config;
use tokio::{net::TcpListener, time::Duration};

#[cfg(feature = "embedded")]
use std::path::Path;

/// Sets the application data root when Websurfx is used as an embedded library.
///
/// Enable the `embedded` Cargo feature and call this before [`Config::parse`] or [`run`]. The
/// directory must contain `public/` and `websurfx/` subdirectories; the latter holds
/// `config.lua`, `allowlist.txt`, and `blocklist.txt`. Websurfx continues to use its standard
/// system paths when a requested path is absent from this root. The selected `public/` directory
/// is used as a whole, so it should contain the complete public asset tree.
///
/// The data root can only be set once and must be set before any operation initializes the
/// resource paths.
#[cfg(feature = "embedded")]
pub fn set_data_root(path: impl AsRef<Path>) -> tokio::io::Result<()> {
    handler::set_data_root(path)
}

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
/// use tokio::{sync::OnceCell, net::TcpListener};
/// use websurfx::{parser::Config, run};
///
/// /// A static constant for holding the parsed config.
/// static CONFIG: OnceCell<Config> = OnceCell::const_new();
///
/// #[tokio::main]
/// async fn main(){
///     // Initialize the parsed config globally.
///     let config = CONFIG
///        .get_or_try_init(|| async move {
///            Config::parse(false)
///                .await
///                .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::Other, e.to_string()))
///        })
///        .await
///        .unwrap();
///     let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind address");
///     let server = run(listener,&config).await.expect("Failed to start server");
/// }
/// ```
pub async fn run(listener: TcpListener, config: &'static Config) -> tokio::io::Result<Server> {
    let public_folder_path = file_path(FileType::Theme).await?;

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
            .wrap(cors)
            .wrap(Governor::new(
                &GovernorConfigBuilder::default()
                    .seconds_per_request(config.rate_limiter.time_limit as u64)
                    .burst_size(config.rate_limiter.number_of_requests as u32)
                    .finish()
                    .unwrap(),
            ))
            .wrap(
                DefaultHeaders::new().add(CacheControl(vec![CacheDirective::MaxAge(
                    config.http_cache_expiry_time as u32,
                )])),
            )
            // Serve images and static files (css and js files).
            .service(
                fs::Files::new("/static", public_folder_path.join("static")).show_files_listing(),
            )
            .service(
                fs::Files::new("/favicon", public_folder_path.join("favicon")).show_files_listing(),
            )
            .service(
                fs::Files::new("/images", public_folder_path.join("images")).show_files_listing(),
            )
            .service(routes::robots_data) // robots.txt
            .service(routes::index) // index page
            .service(routes::search::search) // search page
            .service(routes::about) // about page
            .service(routes::settings) // settings page
            .service(routes::export_import::set_settings) // upload settings
            .service(routes::export_import::download) // download page
            .service(routes::opensearch_description) // opensearch description file
            .default_service(web::route().to(routes::not_found)) // error page
    })
    .workers(config.threads as usize)
    // Set the keep-alive timer for client connections
    .keep_alive(Duration::from_secs(
        config.client_connection_keep_alive as u64,
    ))
    // Start server on 127.0.0.1 with the user provided port number. for example 127.0.0.1:8080.
    .listen(listener.into_std()?)?
    .run();
    Ok(server)
}
