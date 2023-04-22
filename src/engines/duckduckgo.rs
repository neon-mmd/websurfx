use std::collections::HashMap;

use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};

use crate::search_results_handler::aggregation_models::RawSearchResult;

// This function scrapes results from the upstream engine duckduckgo and puts all the scraped
// results like title, visiting_url (href in html),engine (from which engine it was fetched from)
// and description in a RawSearchResult and then adds that to HashMap whose keys are url and
// values are RawSearchResult struct and then returns it within a Result enum.
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
                format!("https://html.duckduckgo.com/html/?q={query}&s=&dc=&v=1&o=json&api=/d.js")
            } else {
                format!(
                    "https://duckduckgo.com/html/?q={}&s={}&dc={}&v=1&o=json&api=/d.js",
                    query,
                    page_number / 2 * 30,
                    page_number / 2 * 30 + 1
                )
            }
        }
        None => format!("https://html.duckduckgo.com/html/?q={query}&s=&dc=&v=1&o=json&api=/d.js"),
    };

    // fetch the html from upstream duckduckgo engine
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
    let result_title: Selector = Selector::parse(".result__a")?;
    let result_url: Selector = Selector::parse(".result__url")?;
    let result_desc: Selector = Selector::parse(".result__snippet")?;

    let mut search_results: HashMap<String, RawSearchResult> = HashMap::new();

    // scrape all the results from the html
    for result in document.select(&results) {
        let search_result: RawSearchResult = RawSearchResult {
            title: result
                .select(&result_title)
                .next()
                .unwrap()
                .inner_html()
                .trim()
                .to_string(),
            visiting_url: format!(
                "https://{}",
                result
                    .select(&result_url)
                    .next()
                    .unwrap()
                    .inner_html()
                    .trim()
            ),
            description: result
                .select(&result_desc)
                .next()
                .unwrap()
                .inner_html()
                .trim()
                .to_string(),
            engine: vec!["duckduckgo".to_string()],
        };
        search_results.insert(
            format!(
                "https://{}",
                result
                    .select(&result_url)
                    .next()
                    .unwrap()
                    .inner_html()
                    .trim()
            ),
            search_result,
        );
    }

    Ok(search_results)
}
