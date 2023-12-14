//! A module that handles the view for the about page in the `websurfx` frontend.

use maud::{html, Markup};

use crate::templates::partials::{footer::footer, header::header};

/// A function that handles the html code for the about page view in the search engine frontend.
///
/// # Arguments
///
/// * `colorscheme` - It takes the colorscheme name as an argument.
/// * `theme` - It takes the theme name as an argument.
///
/// # Returns
///
/// It returns the compiled html markup code as a result.
pub fn about(colorscheme: &str, theme: &str, animation: &Option<String>) -> Markup {
    html!(
        (header(colorscheme, theme, animation))
        main class="about-container"{
         article {
             div{
                h1{"Websurfx"}
                hr size="4" width="100%" color="#a6e3a1"{}
             }
             p{"A modern-looking, lightning-fast, privacy-respecting, secure meta search engine written in Rust. It provides a fast and secure search experience while respecting user privacy."br{}" It aggregates results from multiple search engines and presents them in an unbiased manner, filtering out trackers and ads."
             }

             h2{"Some of the Top Features:"}

             ul{strong{"Lightning fast "}"- Results load within milliseconds for an instant search experience."}

             ul{strong{"Secure search"}" - All searches are performed over an encrypted connection to prevent snooping."}

             ul{strong{"Ad free results"}" - All search results are ad free and clutter free for a clean search experience."}

             ul{strong{"Privacy focused"}" - Websurfx does not track, store or sell your search data. Your privacy is our priority."}

             ul{strong{"Free and Open source"}" - The entire project's code is open source and available for free on "{a href="https://github.com/neon-mmd/websurfx"{"GitHub"}}" under an GNU Affero General Public License."}

             ul{strong{"Highly customizable"}" - Websurfx comes with 9 built-in color themes and supports creating custom themes effortlessly."}
         }

         h3{"Devoloped by: "{a href="https://github.com/neon-mmd/websurfx"{"Websurfx team"}}}
        }
        (footer())
    )
}
