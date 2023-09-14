//! This module provides the models to parse cookies and search parameters from the search
//! engine website.
use serde::Deserialize;

/// A named struct which deserializes all the user provided search parameters and stores them.
#[derive(Deserialize)]
pub struct SearchParams {
    /// It stores the search parameter option `q` (or query in simple words)
    /// of the search url.
    pub q: Option<String>,
    /// It stores the search parameter `page` (or pageno in simple words)
    /// of the search url.
    pub page: Option<u32>,
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
}
