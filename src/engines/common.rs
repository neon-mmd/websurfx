//! This module provides common functionalities for engines

/**
 * Build a query from a list of key value pairs.
 */
pub fn build_query(query_params: &[(&str, &str)]) -> String {
    let mut query_params_string = String::new();
    for (k, v) in query_params {
        query_params_string.push_str(&format!("&{k}={v}"));
    }
    query_params_string
}

/**
 * Build a cookie from a list of key value pairs.
 */
pub fn build_cookie(cookie_params: &[(&str, &str)]) -> String {
    let mut cookie_string = String::new();
    for (k, v) in cookie_params {
        cookie_string.push_str(&format!("{k}={v}; "));
    }
    cookie_string
}
