use crate::models::engine_models::EngineError;
use error_stack::{Report, Result, ResultExt};
use scraper::{Html, Selector};

pub struct SearchResultParser {
    pub no_result: Selector,
    pub results: Selector,
    pub result_title: Selector,
    pub result_url: Selector,
    pub result_desc: Selector,
}

impl SearchResultParser {
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
}

fn new_selector(selector: &str) -> Result<Selector, EngineError> {
    Selector::parse(selector).map_err(|err| {
        Report::new(EngineError::UnexpectedError).attach_printable(format!(
            "invalid CSS selector: {}, err: {:?}",
            selector, err
        ))
    })
}
