//! A module that handles `navbar` partial for the header partial in the `websurfx` frontend.

use maud::{html, Markup};

/// A functions that handles the html code for the header partial.
///
/// # Returns
///
/// It returns the compiled html code for the navbar as a result.
pub fn navbar() -> Markup {
    html!(
        nav{
            ul{
               li{a href="about"{"about"}}
               li{a href="settings"{"settings"}}
            }
        }
    )
}
