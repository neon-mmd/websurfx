//! A module that handles the view for the about page in the `websurfx` frontend.

use maud::{html, Markup, PreEscaped};

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
             div class="logo-container" {
                (PreEscaped(logo_svg))
             }

             div class="text-block" {
                h3 class="text-block-title" {"Why Websurfx?"}
                div class="hero-text-container" {
                    p class="hero-text" {"Websurfx aggregates results from multiple search engines and presents them in an unbiased manner, filtering out trackers and ads."}
                }
            }

            div class="feature-list" {
                h3 class="feature-list-title" {"Features"}
                div class="features" {

                    div class="feature-card" {
                        div class="feature-card-header" {
                            div class="feature-card-icon" { (PreEscaped(feature_lightning)) }
                            h4 {
                                "Lightning-fast"
                            }
                        }
                        div class="feature-card-body" {
                            p {
                                "Results load within milliseconds for an instant search experience."
                            }
                        }
                    }

                    div class="feature-card" {
                        div class="feature-card-header" {
                            div class="feature-card-icon" { (PreEscaped(feature_secure)) }
                            h4 {
                                "Secure Search"
                            }
                        }
                        div class="feature-card-body" {
                            p {
                                "All searches are performed over an encrypted connection to prevent snooping."
                            }
                        }
                    }

                    div class="feature-card" {
                        div class="feature-card-header" {
                            div class="feature-card-icon" { (PreEscaped(feature_clean)) }
                            h4 {
                                "Ad-free Results"
                            }
                        }
                        div class="feature-card-body" {
                            p {
                                "All search results are ad free and clutter free for a clean search experience."
                            }
                        }
                    }

                    div class="feature-card" {
                        div class="feature-card-header" {
                            div class="feature-card-icon" { (PreEscaped(feature_privacy)) }
                            h4 {
                                "Privacy-focused"
                            }
                        }
                        div class="feature-card-body" {
                            p {
                                "Websurfx does not track, store or sell your search data. Your privacy is our priority."
                            }
                        }
                    }

                    div class="feature-card" {
                        div class="feature-card-header" {
                            div class="feature-card-icon" { (PreEscaped(feature_foss)) }
                            h4 {
                                "Free and Open-source"
                            }
                        }
                        div class="feature-card-body" {
                            p {
                                "The entire project's code is open source and available for free on "{a href="https://github.com/neon-mmd/websurfx"{"GitHub"}}"."
                            }
                        }
                    }

                    div class="feature-card" {
                        div class="feature-card-header" {
                            div class="feature-card-icon" { (PreEscaped(feature_customizable)) }
                            h4 {
                                "Highly Customizable"
                            }
                        }
                        div class="feature-card-body" {
                            p {
                                "Websurfx comes with 9 built-in color themes and supports creating custom themes effortlessly."
                            }
                        }
                    }
                }
             }

         }

         h3 class="about-footnote" {"Developed by the "{a href="https://github.com/neon-mmd/websurfx"{"Websurfx team"}}}
        }
        (footer())
    )
}
