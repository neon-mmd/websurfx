//! This module provides modules that handle the functionality to aggregate the fetched search
//! results from the upstream search engines and filters it if safe search is set to 3 or 4. Also,
//! provides various models to aggregate search results into a standardized form.

pub mod aggregator;
mod user_agent;
