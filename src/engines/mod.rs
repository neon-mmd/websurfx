//! This module provides different modules which handles the functionlity to fetch results from the
//! upstream search engines based on user requested queries. Also provides different models to
//! provide a standard functions to be implemented for all the upstream search engine handling
//! code. Moreover, it also provides a custom error for the upstream search engine handling code.

pub mod bing;
pub mod brave;
pub mod duckduckgo;
pub mod librex;
pub mod mojeek;
pub mod qwant;
mod search_result_parser;
pub mod searx;
pub mod sepiasearch;
pub mod startpage;
pub mod wikipedia;
pub mod yahoo;

/// Build a query from a list of key value pairs.
///
/// # Arguments
///
/// * `query_params` - Takes the query parameters key value a slice of tuples of type string.
///
/// # Returns
///
/// It returns the query key value pairs formatted in the url query parameter format.
fn build_query(query_params: &[(&str, &str)]) -> String {
    let mut query_params_string = String::new();
    for (k, v) in query_params {
        query_params_string.push_str(&format!("&{k}={v}"));
    }
    query_params_string
}

/// Build a cookie from a list of key value pairs.
///
/// # Arguments
///
/// * `cookie_params` - Takes the query parameters key value a slice of tuples of type string.
///
/// # Returns
///
/// It returns the query key value pairs formatted in the cookie key value format.
fn build_cookie(cookie_params: &[(&str, &str)]) -> String {
    let mut cookie_string = String::new();
    for (k, v) in cookie_params {
        cookie_string.push_str(&format!("{k}={v}; "));
    }
    cookie_string
}
