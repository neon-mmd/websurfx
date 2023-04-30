//! This module provides public models for handling, storing and serializing parsed config file
//! options from config.lua by grouping them togather.

use serde::Serialize;

/// A named struct which stores, serializes and groups the parsed config file options of theme and
/// colorscheme names into the Style struct which derives the `Clone` and `Serialize` traits
/// where the `Clone` trait is derived for allowing the struct to be cloned and passed to the
/// server as a shared data between all routes except `/robots.txt` and the `Serialize` trait
/// has been derived for allowing the object to be serialized so that it can be passed to
/// handlebars template files.
///
/// # Fields
//
/// * `theme` - It stores the parsed theme option used to set a theme for the website.
/// * `colorscheme` - It stores the parsed colorscheme option used to set a colorscheme for the
/// theme being used.
#[derive(Serialize, Clone)]
pub struct Style {
    pub theme: String,
    pub colorscheme: String,
}

impl Style {
    /// Constructs a new `Style` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `theme` - It takes the parsed theme option used to set a theme for the website.
    /// * `colorscheme` - It takes the parsed colorscheme option used to set a colorscheme
    /// for the theme being used.
    pub fn new(theme: String, colorscheme: String) -> Self {
        Style { theme, colorscheme }
    }
}
