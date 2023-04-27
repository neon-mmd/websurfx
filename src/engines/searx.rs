//! The `searx` module handles the scraping of results from the searx search engine instance
//! by querying the upstream searx search engine instance with user provided query and with a page
//! number if provided.

use std::collections::HashMap;

use reqwest::header::USER_AGENT;
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
/// * `page` - Takes an Option<u32> as argument which can be either None or a valid page number.
/// * `user_agent` - Takes a random user agent string as an argument.
///
/// # Errors
///
/// Returns a reqwest error if the user is not connected to the internet or if their is failure to
/// reach the above **upstream search engine** page and also returns error if the scraping
/// selector fails to initialize"
pub async fn results(
    query: &str,
    page: Option<u32>,
    user_agent: &str,
) -> Result<HashMap<String, RawSearchResult>, Box<dyn std::error::Error>> {
    // Page number can be missing or empty string and so appropriate handling is required
    // so that upstream server recieves valid page number.
    let url: String = match page {
        Some(page_number) => {
            if page_number <= 1 {
                format!("https://searx.work/search?q={query}")
            } else {
                format!("https://searx.work/search?q={query}&pageno={page_number}",)
            }
        }
        None => format!("https://searx.work/search?q={query}"),
    };

    // fetch the html from upstream searx instance engine
    // TODO: Write better error handling code to handle no results case.
    let results: String = reqwest::Client::new()
        .get(url)
        .header(USER_AGENT, user_agent)
        .send()
        .await?
        .text()
        .await?;

    let document: Html = Html::parse_document(&results);
    let results: Selector = Selector::parse(".result")?;
    let result_title: Selector = Selector::parse("h3>a")?;
    let result_url: Selector = Selector::parse("h3>a")?;
    let result_desc: Selector = Selector::parse(".content")?;

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
                result
                    .select(&result_url)
                    .next()
                    .unwrap()
                    .value()
                    .attr("href")
                    .unwrap()
                    .to_string(),
                result
                    .select(&result_desc)
                    .next()
                    .unwrap()
                    .inner_html()
                    .trim()
                    .to_string(),
                vec!["searx".to_string()],
            )
        })
        .map(|search_result| (search_result.visiting_url.clone(), search_result))
        .collect())
}
