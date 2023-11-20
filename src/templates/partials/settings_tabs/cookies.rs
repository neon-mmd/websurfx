//! A module that handles the engines tab for setting page view in the `websurfx` frontend.

use maud::{html, Markup};

/// A functions that handles the html code for the cookies tab for the settings page for the search page.
///
/// # Returns
///
/// It returns the compiled html markup code for the cookies tab.
pub fn cookies() -> Markup {
    html!(
        div class="cookies tab"{
           h1{"Cookies"}
           p class="description"{
               "This is the cookies are saved on your system and it contains the preferences
               you chose in the settings page"
           }
           input type="text" name="cookie_field" value="" readonly;
           p class="description"{
               "The cookies stored are not used by us for any malicious intend or for
               tracking you in any way."
           }
        }
    )
}
