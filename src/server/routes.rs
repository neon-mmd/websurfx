//! This module provides the functionality to handle different routes of the `websurfx`
//! meta search engine website and provide approriate response to each route/page
//! when requested.

use std::fs::read_to_string;

use crate::{
    cache::cacher::RedisCache,
    config_parser::parser::Config,
    search_results_handler::{aggregation_models::SearchResults, aggregator::aggregate},
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use serde::Deserialize;

/// A named struct which deserializes all the user provided search parameters and stores them.
///
/// # Fields
///
/// * `q` - It stores the search parameter option `q` (or query in simple words)
/// of the search url.
/// * `page` - It stores the search parameter `page` (or pageno in simple words)
/// of the search url.
#[derive(Debug, Deserialize)]
struct SearchParams {
    q: Option<String>,
    page: Option<u32>,
}

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

/// Handles the route of search page of the `websurfx` meta search engine website and it takes
/// two search url parameters `q` and `page` where `page` parameter is optional.
///
/// # Example
///
/// ```bash
/// curl "http://127.0.0.1:8080/search?q=sweden&page=1"
/// ```
///
/// Or
///
/// ```bash
/// curl "http://127.0.0.1:8080/search?q=sweden"
/// ```
#[get("/search")]
pub async fn search(
    hbs: web::Data<Handlebars<'_>>,
    req: HttpRequest,
    config: web::Data<Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let params = web::Query::<SearchParams>::from_query(req.query_string())?;

    //Initialize redis cache connection struct
    let mut redis_cache = RedisCache::new(config.redis_connection_url.clone())?;
    match &params.q {
        Some(query) => {
            if query.trim().is_empty() {
                Ok(HttpResponse::Found()
                    .insert_header(("location", "/"))
                    .finish())
            } else {
                let page_url: String;  // Declare the page_url variable without initializing it

                // ...
                
                let page = match params.page {
                    Some(page_number) => {
                        if page_number <= 1 {
                            page_url = format!(
                                "http://{}:{}/search?q={}&page={}",
                                config.binding_ip_addr, config.port, query, 1
                            );
                            1
                        } else {
                            page_url = format!(
                                "http://{}:{}/search?q={}&page={}",
                                config.binding_ip_addr, config.port, query, page_number
                            );
                
                            page_number
                        }
                    }
                    None => {
                        page_url = format!(
                            "http://{}:{}{}&page={}",
                            config.binding_ip_addr,
                            config.port,
                            req.uri(),
                            1
                        );
                
                        1
                    }
                };
                                              
                // fetch the cached results json.
                let cached_results_json = redis_cache.cached_results_json(&page_url);
                // check if fetched results was indeed fetched or it was an error and if so
                // handle the data accordingly.
                match cached_results_json {
                    Ok(results_json) => {
                        let new_results_json: SearchResults = serde_json::from_str(&results_json)?;
                        let page_content: String = hbs.render("search", &new_results_json)?;
                        Ok(HttpResponse::Ok().body(page_content))
                    }
                    Err(_) => {
                        let mut results_json: crate::search_results_handler::aggregation_models::SearchResults =
                            aggregate(query, page).await?;
                        results_json.add_style(config.style.clone());
                        redis_cache
                            .cache_results(serde_json::to_string(&results_json)?, &page_url)?;
                        let page_content: String = hbs.render("search", &results_json)?;
                        Ok(HttpResponse::Ok().body(page_content))
                    }
                }
            }
        }
        None => Ok(HttpResponse::Found()
            .insert_header(("location", "/"))
            .finish()),
    }
}

/// Handles the route of robots.txt page of the `websurfx` meta search engine website.
#[get("/robots.txt")]
pub async fn robots_data(_req: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = read_to_string("./public/robots.txt")?;
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
