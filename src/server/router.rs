//! This module provides the functionality to handle different routes of the `websurfx`
//! meta search engine website and provide appropriate response to each route/page
//! when requested.

use crate::{
    config::parser::Config,
    handler::paths::{file_path, FileType},
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use std::fs::read_to_string;

/// Handles the route of index page or main page of the `websurfx` meta search engine website.
#[get("/")]
pub async fn index(
    hbs: web::Data<Handlebars<'_>>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("index", &config.style).unwrap();
    Ok(HttpResponse::Ok().body(page_content))
}

/// Handles the route of any other accessed route/page which is not provided by the
/// website essentially the 404 error page.
pub async fn not_found(
    hbs: web::Data<Handlebars<'_>>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("404", &config.style)?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page_content))
}

/// Handles the route of robots.txt page of the `websurfx` meta search engine website.
#[get("/robots.txt")]
pub async fn robots_data(_req: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String =
        read_to_string(format!("{}/robots.txt", file_path(FileType::Theme)?))?;
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=ascii")
        .body(page_content))
}

/// Handles the route of about page of the `websurfx` meta search engine website.
#[get("/about")]
pub async fn about(
    hbs: web::Data<Handlebars<'_>>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("about", &config.style)?;
    Ok(HttpResponse::Ok().body(page_content))
}

/// Handles the route of settings page of the `websurfx` meta search engine website.
#[get("/settings")]
pub async fn settings(
    hbs: web::Data<Handlebars<'_>>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("settings", &config.style)?;
    Ok(HttpResponse::Ok().body(page_content))
}
