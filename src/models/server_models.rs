//! This module provides the models to parse cookies and search parameters from the search
//! engine website.
use serde::Deserialize;

use super::parser_models::Style;

/// A named struct which deserializes all the user provided search parameters and stores them.
#[derive(Deserialize)]
pub struct SearchParams {
    /// It stores the search parameter option `q` (or query in simple words)
    /// of the search url.
    pub q: Option<String>,
    /// It stores the search parameter `page` (or pageno in simple words)
    /// of the search url.
    pub page: Option<u32>,
    /// It stores the search parameter `safesearch` (or safe search level in simple words) of the
    /// search url.
    pub safesearch: Option<u8>,
}

/// A named struct which is used to deserialize the cookies fetched from the client side.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Cookie {
    /// It stores the theme name used in the website.
    pub theme: String,
    /// It stores the colorscheme name used for the website theme.
    pub colorscheme: String,
    /// It stores the user selected upstream search engines selected from the UI.
    pub engines: Vec<String>,
    /// It stores the user selected safe search level from the UI.
    pub safe_search_level: u8,
}

impl Cookie {
    /// server_models::Cookie contructor function
    pub fn build(style: &Style, mut engines: Vec<String>, safe_search_level: u8) -> Self {
        engines.sort();
        Self {
            theme: style.theme.clone(),
            colorscheme: style.colorscheme.clone(),
            engines,
            safe_search_level,
        }
    }
}
