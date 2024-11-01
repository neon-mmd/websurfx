//! A module that handles the view for the search page in the `websurfx` frontend.

use maud::{html, Markup, PreEscaped};

use crate::{
    models::aggregation_models::SearchResults,
    templates::partials::{footer::footer, header::header, search_bar::search_bar},
};

/// A function that handles the html code for the search page view in the search engine frontend.
///
/// # Arguments
///
/// * `colorscheme` - It takes the colorscheme name as an argument.
/// * `theme` - It takes the theme name as an argument.
/// * `query` - It takes the current search query provided by the user as an argument.
/// * `search_results` - It takes the aggregated search results as an argument.
///
/// # Returns
///
/// It returns the compiled html markup code as a result.
pub fn search(
    colorscheme: &str,
    theme: &str,
    animation: &Option<String>,
    query: &str,
    page: u32,
    search_results: &SearchResults,
) -> Markup {
    html!(
        (header(colorscheme, theme, animation))
        main class="results"{
           (search_bar(&search_results.engine_errors_info, search_results.safe_search_level, query))
           .results_aggregated{
              @if !search_results.results.is_empty() {
                  @for result in search_results.results.iter(){
                      .result {
                         h1{a href=(result.url){(PreEscaped(&result.title))}}
                         small{(result.url)}
                         p{(PreEscaped(&result.description))}
                         .upstream_engines{
                            @for name in &result.engine {
                               span{(name)}
                            }
                         }
                      }
                  }
              }
              @else if search_results.disallowed{
                 .result_disallowed{
                    .description{
                       p{
                          "Your search - "{span class="user_query"{(query)}}" -
                          has been disallowed."
                       }
                       p class="description_paragraph"{"Dear user,"}
                       p class="description_paragraph"{
                          "The query - "{span class="user_query"{(query)}}" - has
                          been blacklisted via server configuration and hence disallowed by the
                          server. Henceforth no results could be displayed for your query."
                       }
                    }
                    img src="./images/barricade.png" alt="Image of a Barricade";
                 }
              }
              @else if search_results.filtered {
                 .result_filtered{
                    .description{
                       p{
                          "Your search - "{span class="user_query"{(query)}}" -
                          has been filtered."
                       }
                       p class="description_paragraph"{"Dear user,"}
                       p class="description_paragraph"{
                          "All the search results contain results that has been configured to be
                          filtered out via server configuration and henceforth has been
                          completely filtered out."
                       }
                    }
                    img src="./images/filter.png" alt="Image of a paper inside a funnel";
                 }
              }
              @else if search_results.no_engines_selected {
                 .result_engine_not_selected{
                    .description{
                       p{
                          "No results could be fetched for your search '{span class="user_query"{(query)}}'."
                       }
                       p class="description_paragraph"{"Dear user,"}
                       p class="description_paragraph"{
                          "No results could be retrieved from the upstream search engines as no
                          upstream search engines were selected from the settings page."
                       }
                    }
                    img src="./images/no_selection.png" alt="Image of a white cross inside a red circle";
                 }
              }
              @else {
                 .result_not_found {
                    p{"Your search - "{(query)}" - did not match any documents."}
                    p class="suggestions"{"Suggestions:"}
                    ul{
                       li{"Make sure that all words are spelled correctly."}
                       li{"Try different keywords."}
                       li{"Try more general keywords."}
                    }
                    img src="./images/no_results.gif" alt="Man fishing gif";
                 }
              }
            }
            .page_navigation {
               a href=(format!("/search?q={}&safesearch={}&page={}", query, search_results.safe_search_level, if page > 1 {page-1} else {1})) {
                   (PreEscaped("&#8592;")) "previous"
               }
               a href=(format!("/search?q={}&safesearch={}&page={}", query, search_results.safe_search_level, page+2)) {
                  "next" (PreEscaped("&#8594;"))}
            }
        }
        script src="static/index.js"{}
        script src="static/error_box.js"{}
        (footer())
    )
}
