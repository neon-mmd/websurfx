//! This module provides different modules which handles the functionlity to fetch results from the
//! upstream search engines based on user requested queries. Also provides different models to
//! provide a standard functions to be implemented for all the upstream search engine handling
//! code. Moreover, it also provides a custom error for the upstream search engine handling code.

pub mod brave;
pub mod duckduckgo;
pub mod search_result_parser;
pub mod searx;
