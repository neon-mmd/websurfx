//! This module provides the functionality to handle different routes of the `websurfx`
//! meta search engine website and provide appropriate response to each route/page
//! when requested.

use crate::{
    handler::{FileType, file_path},
    parser::Config,
};
use actix_web::{HttpRequest, HttpResponse, get, http::header::ContentType, web};
use tokio::fs::read_to_string;

pub mod export_import;
pub mod search;

/// Handles the route of index page or main page of the `websurfx` meta search engine website.
#[get("/")]
pub async fn index(
    config: web::Data<&'static Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        crate::templates::views::index::index(
            &config.style.colorscheme,
            &config.style.theme,
            &config.style.animation,
        )
        .0,
    ))
}

/// Handles the route of any other accessed route/page which is not provided by the
/// website essentially the 404 error page.
pub async fn not_found(
    config: web::Data<&'static Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        crate::templates::views::not_found::not_found(
            &config.style.colorscheme,
            &config.style.theme,
            &config.style.animation,
        )
        .0,
    ))
}

/// Handles the route of robots.txt page of the `websurfx` meta search engine website.
#[get("/robots.txt")]
pub async fn robots_data(_req: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String =
        read_to_string(format!("{}/robots.txt", file_path(FileType::Theme).await?)).await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(page_content))
}

/// Handles the `/websurfx.xml` route that serves the OpenSearch description
/// document for the `websurfx` meta search engine, allowing browsers to
/// auto-discover and register it as a search provider.
#[get("/websurfx.xml")]
pub async fn opensearch_description(
    _req: HttpRequest,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = read_to_string(format!(
        "{}/websurfx.xml",
        file_path(FileType::Theme).await?
    ))
    .await?;
    let content_type = ContentType("application/opensearchdescription+xml".parse()?);
    Ok(HttpResponse::Ok()
        .insert_header(content_type)
        .body(page_content))
}

/// Handles the route of about page of the `websurfx` meta search engine website.
#[get("/about")]
pub async fn about(
    config: web::Data<&'static Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        crate::templates::views::about::about(
            &config.style.colorscheme,
            &config.style.theme,
            &config.style.animation,
        )
        .0,
    ))
}

/// Handles the route of settings page of the `websurfx` meta search engine website.
#[get("/settings")]
pub async fn settings(
    config: web::Data<&'static Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        crate::templates::views::settings::settings(
            config.safe_search,
            &config.style.colorscheme,
            &config.style.theme,
            &config.style.animation,
            &config.upstream_search_engines,
        )
        .await?
        .0,
    ))
}
