//! This module handles the search route of the search engine website.

use crate::{
    aggregator::aggregate,
    handler::{FileType, file_path},
    models::{
        aggregation::SearchResults,
        engine::EngineHandler,
        search_route::{self, SearchParams},
    },
    parser::Config,
    user_agent::random_user_agent,
};

#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
use {crate::cache::SharedCache, tokio::sync::OnceCell};

use actix_web::{HttpRequest, HttpResponse, ResponseError, get, http::header::ContentType, web};
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{borrow::Cow, time::Duration};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// A static constant for holding the cache struct.
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
static SHARED_CACHE: OnceCell<SharedCache> = OnceCell::const_new();

/// Handles the route of search page of the `websurfx` meta search engine website and it takes
/// two search url parameters `q` and `page` where `page` parameter is optional.
/// An optional `format` parameter can be provided to get results as JSON.
///
/// # Example
///
/// ```bash
/// # HTML response (default)
/// curl "http://127.0.0.1:8080/search?q=sweden&page=1"
///
/// # JSON API response
/// curl "http://127.0.0.1:8080/search?q=sweden&format=json=true"
/// ```
///
/// Detect `format=json` from the raw query string before deserialization,
/// so that parse failures can still return a JSON-shaped 400 response.
#[get("/search")]
pub async fn search(
    req: HttpRequest,
    config: web::Data<&'static Config>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let params_result = web::Query::<SearchParams>::from_query(req.query_string());
    let params = if let Err(e) = params_result {
        if req.query_string().contains("&json") {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "code": format!("{}", e.status_code()),
                "error": format!("Invalid query parameters: {}", e)
            })));
        }
        return Err(e.into());
    } else {
        params_result.unwrap()
    };

    let result = fetch_results(req, &config, &params).await?;

    if let Some((current_results, query, page)) = result {
        if let Some(json) = &params.json
            && *json
        {
            return Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .json(&current_results));
        }
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
            crate::templates::views::search::search(
                &config.style.colorscheme,
                &config.style.theme,
                &config.style.animation,
                &query,
                page,
                &current_results,
            )
            .0,
        ))
    } else {
        if let Some(json) = &params.json
            && *json
        {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "code": "400",
                "error": "Empty query provided"
            })));
        }
        Ok(HttpResponse::TemporaryRedirect()
            .insert_header(("location", "/"))
            .finish())
    }
}

/// Fetches search results from cache or upstream engines.
///
/// # Arguments
///
/// * `req` - The HTTP request used to extract cookies and pass to upstream calls.
/// * `config` - A reference to the application configuration.
/// * `params` - The parsed search parameters including query string and page number.
///
/// # Returns
///
/// Returns `Ok(Some((SearchResults, query, page)))` on success with the results,
/// query string, and zero-based page number. Returns `Ok(None)` for empty queries.
/// Returns an error if results could not be fetched from cache or upstream engines.
///
/// # Examples
///
/// ```rust,ignore
/// let result = fetch_results(req, &config, params).await?;
/// ```
async fn fetch_results(
    req: HttpRequest,
    config: &web::Data<&'static Config>,
    params: &SearchParams,
) -> Result<Option<(SearchResults, String, u32)>, Box<dyn std::error::Error>> {
    // Validate the query early, before touching the cache or doing any setup.
    if params.q.as_ref().is_none_or(|q| q.trim().is_empty()) {
        return Ok(None);
    }

    #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
    let cache = SHARED_CACHE
        .get_or_try_init(|| SharedCache::new(config))
        .await?;

    // Safe to unwrap: we validated q is Some and non-empty above.
    let query = params.q.as_deref().unwrap();

    let cookie = req.cookie("appCookie");

    // Get search settings using the user's cookie or from the server's config
    let mut search_settings: search_route::Cookie<'_> = cookie
        .as_ref()
        .and_then(|cookie_value| serde_json::from_str(cookie_value.value()).ok())
        .unwrap_or_else(|| {
            search_route::Cookie::build(
                &config.style,
                config
                    .upstream_search_engines
                    .iter()
                    .filter_map(|(engine, enabled)| {
                        enabled.then_some(Cow::Borrowed(engine.as_str()))
                    })
                    .collect(),
                config.safe_search,
            )
        });

    search_settings.safe_search_level = get_safesearch_level(
        params.safesearch,
        search_settings.safe_search_level,
        config.safe_search,
    );

    // Add a random delay before making the request.
    if config.aggregator.random_delay || config.debug {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
        let delay = nanos % 10 + 1;
        tokio::time::sleep(Duration::from_secs(delay as u64)).await;
    }

    let user_agent: &'static str = random_user_agent(config.threads).await?;

    // .max(1) makes sure that the page >= 0.
    let page = params.page.unwrap_or(1).max(1) - 1;
    let query_owned = query.to_owned();

    let current_results: SearchResults;

    #[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
    {
        let previous_page = page.saturating_sub(1);

        let next_page = page + 1;

        let mut pages = vec![next_page, previous_page, page];
        pages.dedup();

        let urls: Vec<String> = pages
            .iter()
            .map(|page| {
                format!(
                    "http://{}:{}/search?q={}&page={}&safesearch={}&engines={}",
                    config.binding_ip,
                    config.port,
                    query,
                    page,
                    search_settings.safe_search_level,
                    search_settings.engines.join(",")
                )
            })
            .collect();

        let mut cache_keys: Vec<String> =
            tokio::task::spawn_blocking(move || urls.par_iter().cloned().map(hash_url).collect())
                .await?;

        let current_page_cache_key = cache_keys.pop().unwrap();

        // Use match to avoid eagerly evaluating the upstream fetch on cache hits.
        current_results = match cache.cached_results(&current_page_cache_key).await {
            Ok(cached) => cached,
            Err(_) => {
                let fetched_results =
                    results(config, &query_owned, page, &search_settings, user_agent).await?;
                let fetched_results_clone = fetched_results.clone();
                tokio::spawn(async move {
                    cache
                        .cache_results(&[fetched_results], &[current_page_cache_key])
                        .await
                });
                fetched_results_clone
            }
        };

        if let Ok(resolved_results) = cache.cached_results_exists(&cache_keys).await {
            let cache_results_not_exists: (Vec<String>, Vec<u32>) = resolved_results
                .iter()
                .zip(cache_keys.iter())
                .zip(pages.iter())
                .filter(|resolved_result| !*resolved_result.0.0)
                .map(|resolved_result| (resolved_result.0.1.to_string(), *resolved_result.1))
                .unzip();

            // TODO: Move the entire fetch+cache into a background tokio::spawn
            // for non-blocking responses. Currently blocked by `results()`
            // returning `Box<dyn Error>` (not Send); fixing this would require
            // changing the error type to `Box<dyn Error + Send>` across the
            // codebase.
            if !cache_results_not_exists.0.is_empty() {
                let tasks = cache_results_not_exists
                    .1
                    .iter()
                    .map(|page| results(config, &query_owned, *page, &search_settings, user_agent));
                if let Ok(fetched_results) = futures::future::try_join_all(tasks).await {
                    tokio::spawn(async move {
                        let _ = cache
                            .cache_results(&fetched_results, &cache_results_not_exists.0)
                            .await;
                    });
                }
            }
        } else {
            // TODO: Same as above — spawn the entire fetch+cache once results()
            // returns Send-safe errors.
            let tasks = pages
                .iter()
                .map(|page| results(config, &query_owned, *page, &search_settings, user_agent));
            if let Ok(fetched_results) = futures::future::try_join_all(tasks).await {
                tokio::spawn(async move {
                    let _ = cache.cache_results(&fetched_results, &cache_keys).await;
                });
            }
        }
    }

    #[cfg(not(any(feature = "redis-cache", feature = "memory-cache")))]
    {
        current_results = results(config, &query_owned, page, &search_settings, user_agent).await?;
    }

    Ok(Some((current_results, query_owned, page)))
}

/// Fetches the results for a query and page. It First checks the redis cache, if that
/// fails it gets proper results by requesting from the upstream search engines.
///
/// # Arguments
///
/// * `url` - It takes the url of the current page that requested the search results for a
///   particular search query.
/// * `config` - It takes a parsed config struct.
/// * `query` - It takes the page number as u32 value.
/// * `req` - It takes the `HttpRequest` struct as a value.
///
/// # Error
///
/// It returns the `SearchResults` struct if the search results could be successfully fetched from
/// the cache or from the upstream search engines otherwise it returns an appropriate error.
async fn results(
    config: &'static Config,
    query: &str,
    page: u32,
    search_settings: &search_route::Cookie<'_>,
    user_agent: &'static str,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    // eagerly parse cookie value to evaluate safe search level
    let safe_search_level = search_settings.safe_search_level;

    // check if fetched cache results was indeed fetched or it was an error and if so
    // handle the data accordingly.
    if safe_search_level == 4 {
        let mut results: SearchResults = SearchResults::default();

        let flag: bool =
            !is_match_from_filter_list(&file_path(FileType::BlockList).await?, query).await?;
        // Return early when query contains disallowed words,
        if flag {
            results.set_disallowed();
            results.set_safe_search_level(safe_search_level);
            return Ok(results);
        }
    }

    // check if the cookie value is empty or not if it is empty then use the
    // default selected upstream search engines from the config file otherwise
    // parse the non-empty cookie and grab the user selected engines from the
    // UI and use that.
    let mut results: SearchResults = if !search_settings.engines.is_empty() {
        aggregate(
            query,
            page,
            config,
            &search_settings
                .engines
                .iter()
                .filter_map(|engine| EngineHandler::new(engine).ok())
                .collect::<Vec<EngineHandler>>(),
            safe_search_level,
            user_agent,
        )
        .await?
    } else {
        let mut search_results = SearchResults::default();
        search_results.set_no_engines_selected();
        search_results
    };

    let (engine_errors_info, results_empty_check, no_engines_selected) = (
        results.engine_errors_info().is_empty(),
        results.results().is_empty(),
        results.no_engines_selected(),
    );
    results.set_filtered(engine_errors_info & results_empty_check & !no_engines_selected);
    results.set_safe_search_level(safe_search_level);
    Ok(results)
}

/// A helper function which computes the hash of the url and formats and returns it as string.
///
/// # Arguments
///
/// * `url` - It takes an url as string.
#[cfg(any(feature = "redis-cache", feature = "memory-cache"))]
fn hash_url(url: String) -> String {
    blake3::hash(url.as_bytes()).to_string()
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
async fn is_match_from_filter_list(
    file_path: &str,
    query: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let reader = BufReader::new(File::open(file_path).await?);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        let re = Regex::new(&line)?;
        if re.is_match(query) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// A helper function to choose the safe search level value based on the URL parameters,
/// cookie value and config value.
///
/// # Argurments
///
/// * `safe_search_level_from_url` - Safe search level from the URL parameters.
/// * `cookie_safe_search_level` - Safe search level value from the cookie.
/// * `config_safe_search_level` - Safe search level value from the config file.
///
/// # Returns
///
/// Returns an appropriate safe search level value based on the safe search level values
/// from the URL parameters, cookie and the config file.
fn get_safesearch_level(
    safe_search_level_from_url: Option<u8>,
    cookie_safe_search_level: u8,
    config_safe_search_level: u8,
) -> u8 {
    (u8::from(safe_search_level_from_url.is_some())
        * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
            + (u8::from(config_safe_search_level < 3) * safe_search_level_from_url.unwrap_or(0))))
        + (u8::from(safe_search_level_from_url.is_none())
            * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
                + (u8::from(config_safe_search_level < 3) * cookie_safe_search_level)))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A helper function which creates a random mock safe search level value.
    ///
    /// # Returns
    ///
    /// Returns an optional u8 value.
    fn mock_safe_search_level_value() -> Option<u8> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        let delay = (nanos % 10) as i8 - 1;

        if delay == -1 {
            return None;
        }

        Some(if delay > 4 { delay - 4 } else { delay } as u8)
    }

    #[test]
    /// A test function to test whether the output of the branchless and branched code
    /// for the code to choose the appropriate safe search level is same or not.
    fn get_safesearch_level_branched_branchless_code_test() {
        // Get mock values for the safe search level values for URL parameters, cookie
        // and config.
        let safe_search_level_from_url = mock_safe_search_level_value();
        let cookie_safe_search_level = mock_safe_search_level_value().unwrap_or(0);
        let config_safe_search_level = mock_safe_search_level_value().unwrap_or(0);

        // Branched code
        let safe_search_level_value_from_branched_code =
            if let Some(safe_search_level_from_url_parsed) = safe_search_level_from_url {
                if config_safe_search_level >= 3 {
                    config_safe_search_level
                } else {
                    safe_search_level_from_url_parsed
                }
            } else if config_safe_search_level >= 3 {
                config_safe_search_level
            } else {
                cookie_safe_search_level
            };

        // branchless code
        let safe_search_level_value_from_branchless_code =
            (u8::from(safe_search_level_from_url.is_some())
                * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
                    + (u8::from(config_safe_search_level < 3)
                        * safe_search_level_from_url.unwrap_or(0))))
                + (u8::from(safe_search_level_from_url.is_none())
                    * ((u8::from(config_safe_search_level >= 3) * config_safe_search_level)
                        + (u8::from(config_safe_search_level < 3) * cookie_safe_search_level)));

        assert_eq!(
            safe_search_level_value_from_branched_code,
            safe_search_level_value_from_branchless_code
        );
    }
}
