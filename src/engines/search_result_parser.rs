//! This modules provides helper functionalities for parsing a html document into internal SearchResult.
use std::collections::HashMap;

use crate::models::{aggregation_models::SearchResult, engine_models::EngineError};
use error_stack::{Report, Result};
use scraper::{html::Select, ElementRef, Html, Selector};

/// A html search result parser, based on a predefined CSS selectors.
pub struct SearchResultParser {
    /// selector to locate the element which is displayed, if there were nothing found.
    no_result: Selector,
    /// selector to locate the element which contains one item from the search result.
    results: Selector,
    /// selector to locate the title relative to the search result item.
    result_title: Selector,
    /// selector to locate the url relative to the search result item.
    result_url: Selector,
    /// selector to locate the description relative to the search result item.
    result_desc: Selector,
}

impl SearchResultParser {
    /// Creates a new parser, if all the selectors are valid, otherwise it returns an EngineError
    pub fn new(
        no_result_selector: &str,
        results_selector: &str,
        result_title_selector: &str,
        result_url_selector: &str,
        result_desc_selector: &str,
    ) -> Result<SearchResultParser, EngineError> {
        Ok(SearchResultParser {
            no_result: new_selector(no_result_selector)?,
            results: new_selector(results_selector)?,
            result_title: new_selector(result_title_selector)?,
            result_url: new_selector(result_url_selector)?,
            result_desc: new_selector(result_desc_selector)?,
        })
    }

    /// Parse the html and returns element representing the 'no result found' response.
    pub fn parse_for_no_results<'a>(&'a self, document: &'a Html) -> Select<'a, 'a> {
        document.select(&self.no_result)
    }

    /// Parse the html, and convert the results to SearchResult with the help of the builder function
    pub fn parse_for_results(
        &self,
        document: &Html,
        builder: impl Fn(&ElementRef<'_>, &ElementRef<'_>, &ElementRef<'_>) -> Option<SearchResult>,
    ) -> Result<HashMap<String, SearchResult>, EngineError> {
        let res = document
            .select(&self.results)
            .filter_map(|result| {
                let title = result.select(&self.result_title).next();
                let url = result.select(&self.result_url).next();
                let desc = result.select(&self.result_desc).next();
                match (title, url, desc) {
                    (Some(ref t), Some(ref u), Some(ref d)) => builder(t, u, d),
                    _ => None,
                }
            })
            .map(|search_result| (search_result.url.clone(), search_result))
            .collect();
        Ok(res)
    }
}

/// Create a Selector struct, if the given parameter is a valid css expression, otherwise convert it into an EngineError.
fn new_selector(selector: &str) -> Result<Selector, EngineError> {
    Selector::parse(selector).map_err(|err| {
        Report::new(EngineError::UnexpectedError).attach_printable(format!(
            "invalid CSS selector: {}, err: {:?}",
            selector, err
        ))
    })
}
