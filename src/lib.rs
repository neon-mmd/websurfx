//! This main library module provides the functionality to provide and handle the Tcp server
//! and register all the routes for the `websurfx` meta search engine website.

pub mod cache;
pub mod config_parser;
pub mod engines;
pub mod search_results_handler;
pub mod server;

use std::net::TcpListener;

use crate::server::routes;

use actix_files as fs;
use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use config_parser::parser::Config;
use handlebars::Handlebars;

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
/// use std::net::TcpListener;
/// use websurfx::{config_parser::parser::Config, run};
///
/// let config = Config::parse().unwrap();
/// let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind address");
/// let server = run(listener,config).expect("Failed to start server");
/// ```
pub fn run(listener: TcpListener, config: Config) -> std::io::Result<Server> {
    let mut handlebars: Handlebars = Handlebars::new();

    handlebars
        .register_templates_directory(".html", "./public/templates")
        .unwrap();

    let handlebars_ref: web::Data<Handlebars> = web::Data::new(handlebars);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default()) // added logging middleware for logging.
            // Serve images and static files (css and js files).
            .service(fs::Files::new("/static", "./public/static").show_files_listing())
            .service(fs::Files::new("/images", "./public/images").show_files_listing())
            .service(routes::robots_data) // robots.txt
            .service(routes::index) // index page
            .service(routes::search) // search page
            .service(routes::about) // about page
            .service(routes::settings) // settings page
            .default_service(web::route().to(routes::not_found)) // error page
    })
    // Start server on 127.0.0.1 with the user provided port number. for example 127.0.0.1:8080.
    .listen(listener)?
    .run();
    Ok(server)
}
