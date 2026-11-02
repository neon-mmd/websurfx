//! This module provides the models to parse cookies and search parameters from the search
//! engine website.
use super::parser::Style;
use crate::json::{self, JsonError, JsonValue, JsonWriter};
use std::borrow::Cow;

/// A named struct which holds all the user provided search parameters.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SearchParams {
    /// It stores the search parameter option `q` (or query in simple words)
    /// of the search url.
    pub q: Option<String>,
    /// It stores the search parameter `page` (or pageno in simple words)
    /// of the search url.
    pub page: Option<u32>,
    /// It stores the search parameter `safesearch` (or safe search level in simple words) of the
    /// search url.
    pub safesearch: Option<u8>,
    /// It stores the search parameter `json` which determines whether to return
    /// a JSON API response instead of the HTML page.
    pub json: Option<bool>,
}

/// The error returned when a search url carries a parameter that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamsError {
    /// The name of the parameter that could not be used.
    parameter: &'static str,
    /// Why it could not be used.
    reason: &'static str,
}

impl std::fmt::Display for ParamsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Query parameter `{}` is invalid: {}",
            self.parameter, self.reason
        )
    }
}

impl std::error::Error for ParamsError {}

impl SearchParams {
    /// Parses the raw query string of a search request.
    ///
    /// Percent decoding is handled by [`form_urlencoded`], so `+` and `%20` both become a
    /// space, exactly as they did when the parameters were deserialized by hand.
    ///
    /// # Arguments
    ///
    /// * `query` - It takes the raw query string of the request.
    ///
    /// # Error
    ///
    /// Returns a [`ParamsError`] if one of the numeric or boolean parameters cannot be used.
    pub fn parse(query: &str) -> Result<Self, ParamsError> {
        let mut params = Self::default();

        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            match key.as_ref() {
                "q" => params.q = Some(value.into_owned()),
                "page" => params.page = Some(Self::number("page", &value)?),
                "safesearch" => params.safesearch = Some(Self::number("safesearch", &value)?),
                "json" => params.json = Some(Self::boolean("json", &value)?),
                _ => {}
            }
        }

        Ok(params)
    }

    /// Parses a `page` or `safesearch` parameter.
    fn number<T>(parameter: &'static str, value: &str) -> Result<T, ParamsError>
    where
        T: std::str::FromStr,
    {
        value.parse().map_err(|_| ParamsError {
            parameter,
            reason: "expected a non-negative whole number",
        })
    }

    /// Parses a `json` parameter, accepting exactly the two JSON boolean spellings.
    fn boolean(parameter: &'static str, value: &str) -> Result<bool, ParamsError> {
        match value {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(ParamsError {
                parameter,
                reason: "expected `true` or `false`",
            }),
        }
    }
}

/// A named struct which holds the user preferences stored in the client cookie.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cookie<'a> {
    /// It stores the theme name used in the website.
    pub theme: Cow<'a, str>,
    /// It stores the colorscheme name used for the website theme.
    pub colorscheme: Cow<'a, str>,
    /// It stores the user selected upstream search engines selected from the UI.
    pub engines: Cow<'a, [Cow<'a, str>]>,
    /// It stores the user selected safe search level from the UI.
    pub safe_search_level: u8,
    /// It stores the animation name used for the website theme.
    pub animation: Option<Cow<'a, str>>,
}

impl<'a> Cookie<'a> {
    /// server_models::Cookie contructor function
    pub fn build(style: &'a Style, mut engines: Vec<Cow<'a, str>>, safe_search_level: u8) -> Self {
        engines.sort();
        Self {
            theme: Cow::Borrowed(&style.theme),
            colorscheme: Cow::Borrowed(&style.colorscheme),
            engines: Cow::Owned(engines),
            safe_search_level,
            animation: style
                .animation
                .as_ref()
                .map(|str| Cow::Borrowed(str.as_str())),
        }
    }

    /// Restores the preferences from the JSON stored in the `appCookie` cookie.
    ///
    /// Every string is borrowed from `json` when the payload has no escape sequences.
    ///
    /// # Arguments
    ///
    /// * `json` - It takes the cookie value holding the JSON document.
    ///
    /// # Error
    ///
    /// Returns a [`JsonError`] when the document is malformed or is missing a field.
    pub fn from_json(json: &'a str) -> Result<Self, JsonError> {
        Self::from_document(&json::parse(json.as_bytes())?)
    }

    /// Restores the preferences from an already parsed document.
    fn from_document(document: &JsonValue<'a>) -> Result<Self, JsonError> {
        let theme = document
            .get("theme")
            .and_then(JsonValue::to_cow)
            .ok_or_else(|| JsonError::missing("missing field `theme`"))?;
        let colorscheme = document
            .get("colorscheme")
            .and_then(JsonValue::to_cow)
            .ok_or_else(|| JsonError::missing("missing field `colorscheme`"))?;
        let safe_search_level = document
            .get("safe_search_level")
            .and_then(JsonValue::as_u8)
            .ok_or_else(|| JsonError::missing("missing field `safe_search_level`"))?;

        let engines = match document.get("engines").and_then(JsonValue::as_array) {
            Some(values) => values
                .iter()
                .map(|engine| {
                    engine
                        .to_cow()
                        .ok_or_else(|| JsonError::missing("`engines` must hold strings"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => return Err(JsonError::missing("missing field `engines`")),
        };

        let animation = document.get("animation").and_then(JsonValue::to_cow);

        Ok(Self {
            theme,
            colorscheme,
            engines: Cow::Owned(engines),
            safe_search_level,
            animation,
        })
    }

    /// Renders the preferences as the JSON document stored in the `appCookie` cookie.
    pub fn to_json(&self) -> String {
        let mut writer = JsonWriter::with_capacity(128);

        writer.begin_object();
        writer.field("theme").string(&self.theme);
        writer.field("colorscheme").string(&self.colorscheme);
        writer.field("engines").begin_array();
        for engine in self.engines.iter() {
            writer.string(engine);
        }
        writer.end_array();
        writer
            .field("safe_search_level")
            .unsigned(self.safe_search_level.into());

        match &self.animation {
            Some(animation) => {
                writer.field("animation").string(animation);
            }
            None => {
                writer.field("animation").null();
            }
        }

        writer.end_object();
        writer.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_documented_search_urls() {
        assert_eq!(
            SearchParams::parse("q=sweden&page=3&safesearch=2&json=true").unwrap(),
            SearchParams {
                q: Some("sweden".to_owned()),
                page: Some(3),
                safesearch: Some(2),
                json: Some(true),
            }
        );

        assert_eq!(SearchParams::parse("").unwrap(), SearchParams::default());
        assert_eq!(
            SearchParams::parse("q=a+b%20c").unwrap().q,
            Some("a b c".to_owned())
        );
        assert_eq!(
            SearchParams::parse("unknown=1&q=rust").unwrap(),
            SearchParams {
                q: Some("rust".to_owned()),
                ..SearchParams::default()
            }
        );
    }

    #[test]
    fn rejects_unusable_search_parameters() {
        for query in [
            "q=rust&page=zero",
            "q=rust&page=-1",
            "q=rust&page=99999999999999999999",
            "q=rust&safesearch=999",
            "q=rust&safesearch=x",
            "q=rust&json=1",
            "q=rust&json=",
        ] {
            assert!(
                SearchParams::parse(query).is_err(),
                "`{query}` must be rejected"
            );
        }
    }

    #[test]
    fn round_trips_a_cookie_through_json() {
        let style = Style::new(
            "latte".to_owned(),
            "mocha".to_owned(),
            Some("fade".to_owned()),
        );
        let cookie = Cookie::build(&style, vec![Cow::Borrowed("brave")], 2);
        let json = cookie.to_json();

        assert_eq!(
            json,
            concat!(
                r#"{"theme":"latte","colorscheme":"mocha","engines":["brave"],"#,
                r#""safe_search_level":2,"animation":"fade"}"#
            )
        );

        assert_eq!(Cookie::from_json(&json).unwrap(), cookie);
    }

    #[test]
    fn round_trips_a_cookie_without_an_animation() {
        let style = Style::new("latte".to_owned(), "mocha".to_owned(), None);
        let cookie = Cookie::build(&style, Vec::new(), 0);
        let json = cookie.to_json();

        assert!(json.ends_with(r#""animation":null}"#));
        assert_eq!(Cookie::from_json(&json).unwrap(), cookie);
    }

    #[test]
    fn borrows_strings_out_of_the_cookie_value() {
        let json = r#"{"theme":"latte","colorscheme":"mocha","engines":["brave","searx"],"safe_search_level":1,"animation":null}"#;
        let cookie = Cookie::from_json(json).unwrap();

        let body = json.as_ptr() as usize;
        let inside = |text: &str| {
            let start = text.as_ptr() as usize;
            start >= body && start + text.len() <= body + json.len()
        };

        assert!(inside(&cookie.theme));
        assert!(inside(&cookie.colorscheme));
        assert!(cookie.engines.iter().all(|engine| inside(engine)));
    }

    #[test]
    fn escapes_control_characters_in_a_cookie() {
        let cookie = Cookie {
            theme: Cow::Borrowed("a\"b\u{1}"),
            colorscheme: Cow::Borrowed("c\\d"),
            engines: Cow::Owned(Vec::new()),
            safe_search_level: 0,
            animation: None,
        };
        let json = cookie.to_json();

        assert!(json.contains(r#""theme":"a\"b\u0001""#), "{json}");
        assert_eq!(Cookie::from_json(&json).unwrap(), cookie);
    }

    #[test]
    fn rejects_cookies_that_are_missing_fields() {
        for json in [
            r#"{"colorscheme":"mocha","engines":[],"safe_search_level":0}"#,
            r#"{"theme":"latte","engines":[],"safe_search_level":0}"#,
            r#"{"theme":"latte","colorscheme":"mocha","safe_search_level":0}"#,
            r#"{"theme":"latte","colorscheme":"mocha","engines":[]}"#,
            r#"{"theme":"latte","colorscheme":"mocha","engines":[1],"safe_search_level":0}"#,
            r#"{"theme":"latte","colorscheme":"mocha","engines":[],"safe_search_level":999}"#,
            "not json",
        ] {
            assert!(
                Cookie::from_json(json).is_err(),
                "`{json}` must be rejected"
            );
        }
    }
}
