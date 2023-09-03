//! This module handles the search route of the search engine website.

use crate::{
    cache::cacher::RedisCache,
    config::parser::Config,
    models::{
        aggregation_models::SearchResults,
        engine_models::EngineHandler,
        server_models::{Cookie, SearchParams},
    },
    results::aggregator::aggregate,
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use tokio::join;

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
                    query.to_string(),
                    page - 1,
                    req.clone(),
                ),
                results(
                    format!(
                        "http://{}:{}/search?q={}&page={}",
                        config.binding_ip, config.port, query, page
                    ),
                    &config,
                    query.to_string(),
                    page,
                    req.clone(),
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
                    query.to_string(),
                    page + 1,
                    req.clone(),
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

/// Fetches the results for a query and page. It First checks the redis cache, if that
/// fails it gets proper results by requesting from the upstream search engines.
///
/// # Arguments
///
/// * `url` - It takes the url of the current page that requested the search results for a
/// particular search query.
/// * `config` - It takes a parsed config struct.
/// * `query` - It takes the page number as u32 value.
/// * `req` - It takes the `HttpRequest` struct as a value.
///
/// # Error
///
/// It returns the `SearchResults` struct if the search results could be successfully fetched from
/// the cache or from the upstream search engines otherwise it returns an appropriate error.
async fn results(
    url: String,
    config: &Config,
    query: String,
    page: u32,
    req: HttpRequest,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    // Initialize redis cache connection struct
    let mut redis_cache = RedisCache::new(config.redis_url.clone())?;
    // fetch the cached results json.
    let cached_results_json = redis_cache.cached_json(&url);
    // check if fetched cache results was indeed fetched or it was an error and if so
    // handle the data accordingly.
    match cached_results_json {
        Ok(results) => Ok(serde_json::from_str::<SearchResults>(&results).unwrap()),
        Err(_) => {
            // check if the cookie value is empty or not if it is empty then use the
            // default selected upstream search engines from the config file otherwise
            // parse the non-empty cookie and grab the user selected engines from the
            // UI and use that.
            let mut results: SearchResults = match req.cookie("appCookie") {
                Some(cookie_value) => {
                    let cookie_value: Cookie = serde_json::from_str(cookie_value.name_value().1)?;

                    let engines = cookie_value
                        .engines
                        .iter()
                        .filter_map(|name| EngineHandler::new(name))
                        .collect();

                    aggregate(
                        query,
                        page,
                        config.aggregator.random_delay,
                        config.debug,
                        engines,
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
                        config.upstream_search_engines.clone(),
                        config.request_timeout,
                    )
                    .await?
                }
            };
            results.add_style(config.style.clone());
            redis_cache.cache_results(serde_json::to_string(&results)?, &url)?;
            Ok(results)
        }
    }
}
