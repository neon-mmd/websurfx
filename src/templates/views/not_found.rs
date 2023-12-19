//! A module that handles the view for the 404 page in the `websurfx` frontend.

use crate::templates::partials::{footer::footer, header::header};
use maud::{html, Markup};

/// A function that handles the html code for the 404 page view in the search engine frontend.
///
/// # Arguments
///
/// * `colorscheme` - It takes the colorscheme name as an argument.
/// * `theme` - It takes the theme name as an argument.
///
/// # Returns
///
/// It returns the compiled html markup code as a result.
pub fn not_found(colorscheme: &str, theme: &str, animation: &Option<String>) -> Markup {
    html!(
        (header(colorscheme, theme, animation))
        main class="error_container"{
         img src="images/robot-404.svg" alt="Image of broken robot.";
         .error_content{
          h1{"Aw! snap"}
          h2{"404 Page Not Found!"}
          p{"Go to "{a href="/"{"search page"}}}
         }
        }
        (footer())
    )
}
