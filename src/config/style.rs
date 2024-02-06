#![allow(missing_docs)]

use serde::Deserialize;

/// Stores configurations related to style of the UI.
#[derive(Clone, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Style {
    /// The theme to use for the website.
    pub theme: String,
    /// The colorscheme to use for the theme.
    pub colorscheme: String,
    /// The animation to use for the theme.
    pub animation: Option<String>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            theme: "simple".to_string(),
            colorscheme: "catppuccin-mocha".to_string(),
            animation: Some("simple-frosted-glow".to_string()),
        }
    }
}
