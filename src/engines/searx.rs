use std::collections::HashMap;

use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};

use crate::search_results_handler::aggregation_models::RawSearchResult;

// This function scrapes results from the upstream engine searx instance and puts all the scraped
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
            visiting_url: result
                .select(&result_url)
                .next()
                .unwrap()
                .value()
                .attr("href")
                .unwrap()
                .to_string(),
            description: result
                .select(&result_desc)
                .next()
                .unwrap()
                .inner_html()
                .trim()
                .to_string(),
            engine: vec!["searx".to_string()],
        };
        search_results.insert(
            result
                .select(&result_url)
                .next()
                .unwrap()
                .value()
                .attr("href")
                .unwrap()
                .to_string(),
            search_result,
        );
    }

    Ok(search_results)
}
