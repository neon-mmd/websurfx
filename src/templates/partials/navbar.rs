//!

use maud::{html, Markup};

///
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
