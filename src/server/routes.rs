//! This module provides the functionality to handle different routes of the `websurfx`
//! meta search engine website and provide appropriate response to each route/page
//! when requested.

use std::fs::read_to_string;

use crate::{
    cache::cacher::RedisCache,
    config::parser::Config,
    engines::engine_models::EngineHandler,
    handler::paths::{file_path, FileType},
    results::{aggregation_models::SearchResults, aggregator::aggregate},
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use serde::Deserialize;
use tokio::join;

/// A named struct which deserializes all the user provided search parameters and stores them.
///
/// # Fields
///
/// * `q` - It stores the search parameter option `q` (or query in simple words)
/// of the search url.
/// * `page` - It stores the search parameter `page` (or pageno in simple words)
/// of the search url.
#[derive(Deserialize)]
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

/// A named struct which is used to deserialize the cookies fetched from the client side.
///
/// # Fields
///
/// * `theme` - It stores the theme name used in the website.
/// * `colorscheme` - It stores the colorscheme name used for the website theme.
/// * `engines` - It stores the user selected upstream search engines selected from the UI.
#[allow(dead_code)]
#[derive(Deserialize)]
struct Cookie<'a> {
    theme: &'a str,
    colorscheme: &'a str,
    engines: Vec<&'a str>,
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
    match &params.q {
        Some(query) => {
            if query.trim().is_empty() {
                return Ok(HttpResponse::Found()
                    .insert_header(("location", "/"))
                    .finish());
            }
            let page = match &params.page {
                Some(page) => *page,
                None => 1,
            };

            let (_, results, _) = join!(
                results(
                    format!(
                        "http://{}:{}/search?q={}&page={}",
                        config.binding_ip,
                        config.port,
                        query,
                        page - 1
                    ),
                    &config,
                    query,
                    page - 1,
                    &req,
                ),
                results(
                    format!(
                        "http://{}:{}/search?q={}&page={}",
                        config.binding_ip, config.port, query, page
                    ),
                    &config,
                    query,
                    page,
                    &req,
                ),
                results(
                    format!(
                        "http://{}:{}/search?q={}&page={}",
                        config.binding_ip,
                        config.port,
                        query,
                        page + 1
                    ),
                    &config,
                    query,
                    page + 1,
                    &req,
                )
            );

            let page_content: String = hbs.render("search", &results?)?;
            Ok(HttpResponse::Ok().body(page_content))
        }
        None => Ok(HttpResponse::Found()
            .insert_header(("location", "/"))
            .finish()),
    }
}

/// Fetches the results for a query and page.
/// First checks the redis cache, if that fails it gets proper results
async fn results(
    url: String,
    config: &Config,
    query: &str,
    page: u32,
    req: &HttpRequest,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    //Initialize redis cache connection struct
    let mut redis_cache = RedisCache::new(&config.redis_url, 5).await?;
    // fetch the cached results json.
    let cached_results_json = redis_cache.cached_json(&url).await;
    // check if fetched cache results was indeed fetched or it was an error and if so
    // handle the data accordingly.
    match cached_results_json {
        Ok(results) => Ok(serde_json::from_str::<SearchResults>(&results)?),
        Err(_) => {
            // check if the cookie value is empty or not if it is empty then use the
            // default selected upstream search engines from the config file otherwise
            // parse the non-empty cookie and grab the user selected engines from the
            // UI and use that.
            let mut results: SearchResults = match req.cookie("appCookie") {
                Some(cookie_value) => {
                    let cookie_value: Cookie = serde_json::from_str(cookie_value.name_value().1)?;

                    let engines: Vec<EngineHandler> = cookie_value
                        .engines
                        .iter()
                        .filter_map(|name| EngineHandler::new(name))
                        .collect();

                    aggregate(
                        query,
                        page,
                        config.aggregator.random_delay,
                        config.debug,
                        &engines,
                        config.request_timeout,
                    )
                    .await?
                }
                None => {
                    aggregate(
                        query,
                        page,
                        config.aggregator.random_delay,
                        config.debug,
                        &config.upstream_search_engines,
                        config.request_timeout,
                    )
                    .await?
                }
            };
            results.add_style(&config.style);
            redis_cache
                .cache_results(&serde_json::to_string(&results)?, &url)
                .await?;
            Ok(results)
        }
    }
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
