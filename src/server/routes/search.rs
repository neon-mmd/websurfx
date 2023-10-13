//! This module handles the search route of the search engine website.

use crate::{
    cache::cacher::SharedCache,
    config::parser::Config,
    handler::paths::{file_path, FileType},
    models::{
        aggregation_models::SearchResults,
        engine_models::EngineHandler,
        server_models::{Cookie, SearchParams},
    },
    results::aggregator::aggregate,
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};
use tokio::join;

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
    cache: web::Data<SharedCache>,
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
                        "http://{}:{}/search?q={}&page={}&safesearch=",
                        config.binding_ip,
                        config.port,
                        query,
                        page - 1,
                    ),
                    &config,
                    &cache,
                    query,
                    page - 1,
                    req.clone(),
                    &params.safesearch
                ),
                results(
                    format!(
                        "http://{}:{}/search?q={}&page={}&safesearch=",
                        config.binding_ip, config.port, query, page
                    ),
                    &config,
                    &cache,
                    query,
                    page,
                    req.clone(),
                    &params.safesearch
                ),
                results(
                    format!(
                        "http://{}:{}/search?q={}&page={}&safesearch=",
                        config.binding_ip,
                        config.port,
                        query,
                        page + 1,
                    ),
                    &config,
                    &cache,
                    query,
                    page + 1,
                    req.clone(),
                    &params.safesearch
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
    cache: &web::Data<SharedCache>,
    query: &str,
    page: u32,
    req: HttpRequest,
    safe_search: &Option<u8>,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    // fetch the cached results json.
    let cached_results = cache.cached_json(&url).await;
    // check if fetched cache results was indeed fetched or it was an error and if so
    // handle the data accordingly.
    match cached_results {
        Ok(results) => Ok(results),
        Err(_) => {
            let mut safe_search_level: u8 = match config.safe_search {
                3..=4 => config.safe_search,
                _ => match safe_search {
                    Some(safesearch) => match safesearch {
                        0..=2 => *safesearch,
                        _ => config.safe_search,
                    },
                    None => config.safe_search,
                },
            };

            if safe_search_level == 4 {
                let mut results: SearchResults = SearchResults::default();
                let mut _flag: bool =
                    is_match_from_filter_list(file_path(FileType::BlockList)?, query)?;
                _flag = !is_match_from_filter_list(file_path(FileType::AllowList)?, query)?;

                if _flag {
                    results.set_disallowed();
                    results.add_style(&config.style);
                    results.set_page_query(query);
                    cache.cache_results(&results, &url).await?;
                    results.set_safe_search_level(safe_search_level);
                    return Ok(results);
                }
            }

            // check if the cookie value is empty or not if it is empty then use the
            // default selected upstream search engines from the config file otherwise
            // parse the non-empty cookie and grab the user selected engines from the
            // UI and use that.
            let mut results: SearchResults = match req.cookie("appCookie") {
                Some(cookie_value) => {
                    let cookie_value: Cookie<'_> =
                        serde_json::from_str(cookie_value.name_value().1)?;

                    let engines: Vec<EngineHandler> = cookie_value
                        .engines
                        .iter()
                        .filter_map(|name| EngineHandler::new(name).ok())
                        .collect();

                    safe_search_level = match config.safe_search {
                        3..=4 => config.safe_search,
                        _ => match safe_search {
                            Some(safesearch) => match safesearch {
                                0..=2 => *safesearch,
                                _ => config.safe_search,
                            },
                            None => cookie_value.safe_search_level,
                        },
                    };

                    match engines.is_empty() {
                        false => {
                            aggregate(
                                query,
                                page,
                                config.aggregator.random_delay,
                                config.debug,
                                &engines,
                                config.request_timeout,
                                safe_search_level,
                            )
                            .await?
                        }
                        true => {
                            let mut search_results = SearchResults::default();
                            search_results.set_no_engines_selected();
                            search_results.set_page_query(query);
                            search_results
                        }
                    }
                }
                None => {
                    aggregate(
                        query,
                        page,
                        config.aggregator.random_delay,
                        config.debug,
                        &config.upstream_search_engines,
                        config.request_timeout,
                        safe_search_level,
                    )
                    .await?
                }
            };
            if results.engine_errors_info().is_empty()
                && results.results().is_empty()
                && !results.no_engines_selected()
            {
                results.set_filtered();
            }
            results.add_style(&config.style);
            cache
                .cache_results(&results, &(format!("{url}{safe_search_level}")))
                .await?;
            results.set_safe_search_level(safe_search_level);
            Ok(results)
        }
    }
}

/// A helper function which checks whether the search query contains any keywords which should be
/// disallowed/allowed based on the regex based rules present in the blocklist and allowlist files.
///
/// # Arguments
///
/// * `file_path` - It takes the file path of the list as the argument.
/// * `query` - It takes the search query to be checked against the list as an argument.
///
/// # Error
///
/// Returns a bool indicating whether the results were found in the list or not on success
/// otherwise returns a standard error type on a failure.
fn is_match_from_filter_list(
    file_path: &str,
    query: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut flag = false;
    let mut reader = BufReader::new(File::open(file_path)?);
    for line in reader.by_ref().lines() {
        let re = Regex::new(&line?)?;
        if re.is_match(query) {
            flag = true;
            break;
        }
    }
    Ok(flag)
}
