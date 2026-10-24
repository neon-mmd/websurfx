//! Main application server and routing configuration for Websurfx.

mod aggregator;
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
mod cache;
pub mod engines;
mod handler;
pub mod models;
pub mod parser;
mod routes;
pub mod templates;
pub mod user_agent;

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

    let seconds_per_request = (config.rate_limiter.time_limit as u64).max(1);
    let burst_size = (config.rate_limiter.number_of_requests as u32).max(1);
    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(seconds_per_request)
        .burst_size(burst_size)
        .finish()
        .expect("Valid rate limiter configuration");

    let server = HttpServer::new(move || {
        let cors: Cors = Cors::default()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::ORIGIN,
                header::CONTENT_TYPE,
                header::REFERER,
                header::COOKIE,
            ]);

        let default_headers = DefaultHeaders::new()
            .add(CacheControl(vec![CacheDirective::MaxAge(
                config.http_cache_expiry_time as u32,
            )]))
            .add((
                header::CONTENT_SECURITY_POLICY,
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none';",
            ))
            .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
            .add((header::X_FRAME_OPTIONS, "DENY"))
            .add((header::REFERRER_POLICY, "strict-origin-when-cross-origin"));

        App::new()
            .wrap(Compress::default())
            // Mask query strings in access logs to preserve user privacy
            .wrap(Logger::new(r#"%a "%U" %s %b "%{User-Agent}i" %T"#))
            .app_data(web::Data::new(config))
            .wrap(cors)
            .wrap(Governor::new(&governor_conf))
            .wrap(default_headers)
            // Serve images and static files without directory browsing
            .service(fs::Files::new("/static", format!("{}/static", public_folder_path)))
            .service(fs::Files::new("/favicon", format!("{}/favicon", public_folder_path)))
            .service(fs::Files::new("/images", format!("{}/images", public_folder_path)))
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
