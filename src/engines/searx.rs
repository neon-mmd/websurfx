//! The `searx` module handles the scraping of results from the searx search engine instance
//! by querying the upstream searx search engine instance with user provided query and with a page
//! number if provided.

use reqwest::header::{HeaderMap, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT};
use scraper::{Html, Selector};
use std::collections::HashMap;

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
    let url: String = format!("https://searx.work/search?q={query}&pageno={page}");

    // initializing headers and adding appropriate headers.
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT, user_agent.parse()?);
    header_map.insert(REFERER, "https://google.com/".parse()?);
    header_map.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);
    header_map.insert(COOKIE, "categories=general; language=auto; locale=en; autocomplete=duckduckgo; image_proxy=1; method=POST; safesearch=2; theme=simple; results_on_new_tab=1; doi_resolver=oadoi.org; simple_style=auto; center_alignment=1; query_in_title=1; infinite_scroll=0; disabled_engines=; enabled_engines=\"archive is__general\\054yep__general\\054curlie__general\\054currency__general\\054ddg definitions__general\\054wikidata__general\\054duckduckgo__general\\054tineye__general\\054lingva__general\\054startpage__general\\054yahoo__general\\054wiby__general\\054marginalia__general\\054alexandria__general\\054wikibooks__general\\054wikiquote__general\\054wikisource__general\\054wikiversity__general\\054wikivoyage__general\\054dictzone__general\\054seznam__general\\054mojeek__general\\054naver__general\\054wikimini__general\\054brave__general\\054petalsearch__general\\054goo__general\"; disabled_plugins=; enabled_plugins=\"searx.plugins.hostname_replace\\054searx.plugins.oa_doi_rewrite\\054searx.plugins.vim_hotkeys\"; tokens=; maintab=on; enginetab=on".parse()?);

    // fetch the html from upstream searx instance engine
    // TODO: Write better error handling code to handle no results case.
    let results: String = reqwest::Client::new()
        .get(url)
        .headers(header_map) // add spoofed headers to emulate human behaviours.
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
