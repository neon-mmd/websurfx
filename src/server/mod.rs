//! This module provides modules that handle the functionality of handling different routes/paths
//! for the `websurfx` search engine website. Also it handles the parsing of search parameters in
//! the search route. Also, caches the next, current and previous search results in the search
//! routes with the help of the redis server.

pub mod router;
pub mod routes;
