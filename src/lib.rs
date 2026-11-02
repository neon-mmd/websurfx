//! This main library module provides the functionality to provide and handle the Tcp server
//! and register all the routes for the `websurfx` meta search engine website.

mod aggregator;
mod assets;
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
mod cache;
mod engines;
mod handler;
mod json;
mod models;
pub mod parser;
mod routes;
mod server;
pub mod templates;
mod user_agent;

use handler::{FileType, file_path};
use parser::Config;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use xitca_web::{
    App,
    handler::handler_service,
    middleware::{compress::Compress, eraser::TypeEraser, tower_http_compat::TowerHttpCompat},
    route::get,
};

/// A catch-all path used to render the 404 page for anything that is not a known route.
const CATCH_ALL_PATH: &str = "/{*path}";

/// Runs the web server on the provided TCP listener and registers all the routes.
///
/// The config is installed as xitca's typed application state, so every handler reads the
/// same one back out of the request context.
///
/// # Arguments
///
/// * `listener` - It takes the `TcpListener` to serve requests on.
/// * `config` - It takes the parsed application config.
///
/// # Errors
///
/// Returns an error if the public folder cannot be located, or if the listener cannot be
/// taken over by the server.
///
/// # Example
///
/// ```no_run
/// use websurfx::{parser::Config, serve};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let config = Config::parse(false)
///         .await
///         .map_err(|error| std::io::Error::other(error.to_string()))?;
///     let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
///     serve(listener, config).await
/// }
/// ```
pub async fn serve(listener: TcpListener, config: Config) -> std::io::Result<()> {
    let public_folder_path = file_path(FileType::Theme)
        .await
        .map_err(std::io::Error::other)?;

    // The route tree is built inline rather than handed back to the caller: xitca boxes the
    // state builder inside `App`, and that box only infers when the app is consumed directly
    // here. Both values are therefore leaked, which also gives the background prefetch tasks
    // the `'static` config they need. A server holds one config for its whole life anyway, so
    // neither allocation is ever freed.
    let config: &'static Config = Box::leak(Box::new(config));
    server::set_cache_expiry(config.http_cache_expiry_time);
    crate::models::engine::set_instances(config.searx_url.clone(), config.librex_url.clone());
    server::set_rate_limit(
        config.rate_limiter.time_limit,
        config.rate_limiter.number_of_requests,
    );

    let public_folder_path: &'static str = Box::leak(public_folder_path.into_boxed_str());

    routes::set_assets(Arc::new(assets::PublicFolder::new(Path::new(
        public_folder_path,
    ))));

    App::new()
        .with_state(config)
        .at("/robots.txt", get(handler_service(routes::robots_data)))
        .at(
            "/websurfx.xml",
            get(handler_service(routes::opensearch_description)),
        )
        .at("/", get(handler_service(routes::index)))
        .at("/search", get(handler_service(routes::search::search)))
        .at("/about", get(handler_service(routes::about)))
        .at(
            "/settings",
            get(handler_service(routes::settings))
                // xitca's router rejects a second registration for a path that already has
                // one, even for a different method, so the POST is chained onto the GET.
                .post(handler_service(routes::export_import::set_settings)),
        )
        .at(
            "/download",
            get(handler_service(routes::export_import::download)),
        )
        .at(
            "/static/{*path}",
            get(handler_service(routes::static_files)).head(handler_service(routes::static_files)),
        )
        .at(
            "/favicon/{*path}",
            get(handler_service(routes::favicon)).head(handler_service(routes::favicon)),
        )
        .at(
            "/images/{*path}",
            get(handler_service(routes::images)).head(handler_service(routes::images)),
        )
        // Everything that is not a known route renders the 404 page.
        .at(
            CATCH_ALL_PATH,
            get(handler_service(routes::not_found))
                .post(handler_service(routes::not_found))
                .put(handler_service(routes::not_found))
                .delete(handler_service(routes::not_found))
                .patch(handler_service(routes::not_found))
                .head(handler_service(routes::not_found))
                .options(handler_service(routes::not_found)),
        )
        .enclosed_fn(server::rate_limit)
        // `enclosed` wraps from the inside out, so the last call is the outermost. The
        // logging and cache-control middlewares therefore come first, before `Compress`
        // changes the response body type.
        .enclosed_fn(server::with_cache_control)
        .enclosed_fn(server::access_log)
        .enclosed(Compress)
        // `TowerHttpCompat` rewrites the response body type, so `TypeEraser` restores it
        // before the outermost middleware sees the response.
        .enclosed(TowerHttpCompat::new(server::cors_layer()))
        .enclosed(TypeEraser::response_body())
        .serve()
        .server_threads(config.threads as usize)
        .keep_alive_timeout(Duration::from_secs(
            config.client_connection_keep_alive as u64,
        ))
        .listen(listener.into_std()?)?
        .run()
        .await?;

    Ok(())
}
