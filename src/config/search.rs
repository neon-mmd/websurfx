#![allow(missing_docs)]
use serde::Deserialize;
use std::collections::HashMap;

/// Stores configurations related to caching.
#[derive(Clone, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Search {
    /// The search engines to enable/disable.
    pub upstream_search_engines: HashMap<String, bool>,
    /// The safe search level to set
    /// * 0 - None
    /// * 1 - Low
    /// * 2 - Moderate
    /// * 3 - High
    /// * 4 - Aggressive
    pub safe_search: u8,
}

impl Default for Search {
    fn default() -> Self {
        Search {
            upstream_search_engines: {
                let mut map = HashMap::new();
                map.insert("DuckDuckGo".to_string(), true);
                map.insert("Searx".to_string(), false);
                map.insert("Brave".to_string(), false);
                map.insert("Startpage".to_string(), false);
                map.insert("LibreX".to_string(), false);
                map.insert("Mojeek".to_string(), false);
                map.insert("Bing".to_string(), false);
                map
            },
            safe_search: 2,
        }
    }
}
