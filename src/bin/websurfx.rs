use std::ops::RangeInclusive;

use websurfx::server::routes;

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use clap::{command, Parser};
use handlebars::Handlebars;

#[derive(Parser, Debug, Default)]
#[clap(author = "neon_arch", version, about = "Websurfx server application")]
#[command(propagate_version = true)]
struct CliArgs {
    #[clap(default_value_t = 8080, short, long,value_parser = is_port_in_range)]
    /// provide port number in range [1024-65536] to launch the server on.
    port: u16,
}

const PORT_RANGE: RangeInclusive<usize> = 1024..=65535;

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

// The function that launches the main server and handle routing functionality
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = CliArgs::parse();

    println!("started server on port {}", args.port);

    let mut handlebars: Handlebars = Handlebars::new();

    handlebars
        .register_templates_directory(".html", "./public/templates")
        .unwrap();

    let handlebars_ref: web::Data<Handlebars> = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
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
    // Start server on 127.0.0.1:8080
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}
