//! A module that handles the view for the search page in the `websurfx` frontend.

use maud::{Markup, PreEscaped, html};

use crate::{
    models::aggregation::ArchivedSearchResults,
    templates::partials::{footer::footer, header::header, search_bar::search_bar},
};

/// A function that handles the html code for the search page view in the search engine frontend.
///
/// The results are read straight out of an rkyv archive, so every field below borrows from
/// the encoded buffer rather than from an owned `String`.
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
    search_results: &ArchivedSearchResults,
) -> Markup {
    html!(
        (header(colorscheme, theme, animation))
        main class="results"{
           (search_bar(&search_results.engine_errors_info[..], search_results.safe_search_level, query))
           .results_aggregated{
              @if !search_results.results.is_empty() {
                  @for result in search_results.results.iter(){
                      .result {
                         h1{a href=(result.url.as_str()){(PreEscaped(result.title.as_str()))}}
                         small{(result.url.as_str())}
                         p{(PreEscaped(result.description.as_str()))}
                         .upstream_engines{
                            @for name in result.engine.iter(){
                               span{(name.as_str())}
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
              @else if !search_results.engine_errors_info.is_empty() {
                 .result_upstream_error {
                    .description{
                       p{
                          "Your search - "{span class="user_query"{(query)}}" -
                          could not be completed."
                       }
                       p class="description_paragraph"{"Dear user,"}
                       p class="description_paragraph"{
                          "Results could not be fetched from one or more upstream search
                          engines (see the error indicators next to the search bar). This is
                          usually a temporary issue with the upstream engines — please try
                          again."
                       }
                    }
                    img src="./images/no_results.gif" alt="Man fishing gif";
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::aggregation::{
        EngineErrorInfo, SearchResult, SearchResults, SearchResultsArchive,
    };

    /// Renders the search view for the given results and returns the generated markup.
    fn render(results: &SearchResults) -> String {
        let archive = SearchResultsArchive::encode(results).unwrap();

        search(
            "catppuccin-mocha",
            "catppuccin-latte",
            &None,
            "rust",
            1,
            archive.get().unwrap(),
        )
        .0
    }

    /// Builds a result set with one result attributed to two upstream engines.
    fn sample_results() -> SearchResults {
        SearchResults::new(
            vec![SearchResult {
                title: "Rust <Programming> Language".to_owned(),
                url: "https://www.rust-lang.org/?a=1&b=2".to_owned(),
                description: "A language & a community".to_owned(),
                engine: vec!["searx".to_owned(), "brave".to_owned()],
                relevance_score: 1.0,
            }]
            .into_boxed_slice(),
            Box::new([]),
        )
    }

    #[test]
    fn renders_results_straight_out_of_the_archive() {
        let markup = render(&sample_results());

        // The url is escaped by maud when it is spliced into the attribute and the body,
        // while the title and description are emitted pre-escaped by the template.
        assert!(markup.contains("<h1><a href=\"https://www.rust-lang.org/?a=1&amp;b=2\">Rust <Programming> Language</a></h1>"));
        assert!(markup.contains("<p>A language & a community</p>"));
        assert!(markup.contains("<span>searx</span>"));
        assert!(markup.contains("<span>brave</span>"));
    }

    #[test]
    fn renders_the_disallowed_notice_when_the_query_is_blocked() {
        let mut results = SearchResults::default();
        results.set_disallowed();

        let markup = render(&results);

        assert!(markup.contains("result_disallowed"));
        assert!(markup.contains("has been disallowed"));
    }

    #[test]
    fn renders_the_not_found_notice_for_an_empty_result_set() {
        let markup = render(&SearchResults::default());

        assert!(markup.contains("result_not_found"));
        assert!(markup.contains("did not match any documents"));
    }

    #[test]
    fn renders_the_no_engines_notice() {
        let mut results = SearchResults::default();
        results.set_no_engines_selected();

        let markup = render(&results);

        assert!(markup.contains("result_engine_not_selected"));
    }

    #[test]
    fn renders_the_upstream_error_notice_with_the_engine_names() {
        let results = SearchResults::new(
            Box::new([]),
            vec![EngineErrorInfo {
                error: "EmptyResultSet".to_owned(),
                engine: "mojeek".to_owned(),
                severity_color: "blue".to_owned(),
            }]
            .into_boxed_slice(),
        );

        let markup = render(&results);

        assert!(markup.contains("result_upstream_error"));
        assert!(markup.contains("be completed"));
        assert!(markup.contains("error_item"));
        assert!(markup.contains("mojeek"));
        assert!(markup.contains("EmptyResultSet"));
    }
}
