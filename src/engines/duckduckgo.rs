//! The `duckduckgo` module handles the scraping of results from the duckduckgo search engine
//! by querying the upstream duckduckgo search engine with user provided query and with a page
//! number if provided.

use std::collections::HashMap;

use reqwest::header::{HeaderMap, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT};
use scraper::{Html, Selector};

use crate::search_results_handler::aggregation_models::RawSearchResult;

/// This function scrapes results from the upstream engine duckduckgo and puts all the scraped
/// results like title, visiting_url (href in html),engine (from which engine it was fetched from)
/// and description in a RawSearchResult and then adds that to HashMap whose keys are url and
/// values are RawSearchResult struct and then returns it within a Result enum.
///
/// # Arguments
///
/// * `query` - Takes the user provided query to query to the upstream search engine with.
/// * `page` - Takes an u32 as an argument.
/// * `user_agent` - Takes a random user agent string as an argument.
///
/// # Errors
///
/// Returns a reqwest error if the user is not connected to the internet or if their is failure to
/// reach the above `upstream search engine` page and also returns error if the scraping
/// selector fails to initialize"
pub async fn results(
    query: &str,
    page: u32,
    user_agent: &str,
) -> Result<HashMap<String, RawSearchResult>, Box<dyn std::error::Error>> {
    // Page number can be missing or empty string and so appropriate handling is required
    // so that upstream server recieves valid page number.
    let url: String = match page {
        1 => {
            format!("https://html.duckduckgo.com/html/?q={query}&s=&dc=&v=1&o=json&api=/d.js")
        }
        _ => {
            format!(
                "https://duckduckgo.com/html/?q={}&s={}&dc={}&v=1&o=json&api=/d.js",
                query,
                (page / 2 + (page % 2)) * 30,
                (page / 2 + (page % 2)) * 30 + 1
            )
        }
    };

    // initializing HeaderMap and adding appropriate headers.
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT, user_agent.parse()?);
    header_map.insert(REFERER, "https://google.com/".parse()?);
    header_map.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);
    header_map.insert(COOKIE, "kl=wt-wt".parse()?);

    // fetch the html from upstream duckduckgo engine
    // TODO: Write better error handling code to handle no results case.
    let results: String = reqwest::Client::new()
        .get(url)
        .headers(header_map) // add spoofed headers to emulate human behaviour
        .send()
        .await?
        .text()
        .await?;

    let document: Html = Html::parse_document(&results);
    let results: Selector = Selector::parse(".result")?;
    let result_title: Selector = Selector::parse(".result__a")?;
    let result_url: Selector = Selector::parse(".result__url")?;
    let result_desc: Selector = Selector::parse(".result__snippet")?;

    // scrape all the results from the html
    Ok(document
        .select(&results)
        .map(|result| {
            RawSearchResult::new(
                result
                    .select(&result_title)
                    .next()
                    .unwrap()
                    .inner_html()
                    .trim()
                    .to_string(),
                format!(
                    "https://{}",
                    result
                        .select(&result_url)
                        .next()
                        .unwrap()
                        .inner_html()
                        .trim()
                ),
                result
                    .select(&result_desc)
                    .next()
                    .unwrap()
                    .inner_html()
                    .trim()
                    .to_string(),
                vec!["duckduckgo".to_string()],
            )
        })
        .map(|search_result| (search_result.visiting_url.clone(), search_result))
        .collect())
}
