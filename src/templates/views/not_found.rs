//!

use crate::templates::partials::{footer::footer, header::header};
use maud::{html, Markup};

///
pub fn not_found(colorscheme: &str, theme: &str) -> Markup {
    html!(
        (header(colorscheme, theme))
        main class="error_container"{
         img src="images/robot-404.svg" alt="Image of broken robot.";
         div class="error_content"{
          h1{"Aw! snap"}
          h2{"404 Page Not Found!"}
          p{"Go to "{a href="/"{"search page"}}}
         }
        }
        (footer())
    )
}
