//! Engine integrations for querying upstream search providers.
//!
//! Submodules implement the [`SearchEngine`] trait to query upstream engines,
//! handle network responses, and parse search results.

pub mod bing;
pub mod brave;
pub mod duckduckgo;
pub mod librex;
pub mod mojeek;
pub mod qwant;
mod search_result_parser;
pub mod searx;
pub mod sepiasearch;
pub mod startpage;
pub mod wikipedia;
pub mod yahoo;

/// Encodes a list of key-value pairs into a URL query string.
///
/// Keys and values are percent-encoded following standard `application/x-www-form-urlencoded`
/// rules to prevent parameter injection and URL corruption.
///
/// # Arguments
///
/// * `query_params` - Slice of key-value string slices to encode.
///
/// # Returns
///
/// Returns the serialized query string without a leading question mark.
///
/// # Examples
///
/// ```rust
/// use websurfx::engines::build_query;
///
/// let query = build_query(&[("q", "hello world"), ("page", "1")]);
/// assert_eq!(query, "q=hello+world&page=1");
/// ```
pub fn build_query(query_params: &[(&str, &str)]) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (k, v) in query_params {
        serializer.append_pair(k, v);
    }
    serializer.finish()
}

/// Encodes a list of key-value pairs into an HTTP Cookie header string.
///
/// Formats key-value pairs separated by semicolons as required by the Cookie header syntax.
///
/// # Arguments
///
/// * `cookie_params` - Slice of key-value string slices to format into a cookie string.
///
/// # Returns
///
/// Returns the formatted cookie string.
///
/// # Examples
///
/// ```rust
/// use websurfx::engines::build_cookie;
///
/// let cookie = build_cookie(&[("session", "abc"), ("theme", "dark")]);
/// assert_eq!(cookie, "session=abc; theme=dark; ");
/// ```
pub fn build_cookie(cookie_params: &[(&str, &str)]) -> String {
    let mut cookie_string = String::new();
    for (k, v) in cookie_params {
        cookie_string.push_str(&format!("{k}={v}; "));
    }
    cookie_string
}

/// Percent-encodes a single URL query string value.
///
/// Encodes query text so it can safely be included as a query parameter value
/// without injecting syntax or fragment identifiers.
///
/// # Arguments
///
/// * `input` - Raw string slice to percent-encode.
///
/// # Returns
///
/// Returns the percent-encoded string.
///
/// # Examples
///
/// ```rust
/// use websurfx::engines::url_encode;
///
/// assert_eq!(url_encode("hello world"), "hello+world");
/// assert_eq!(url_encode("q&a=true"), "q%26a%3Dtrue");
/// ```
pub fn url_encode(input: &str) -> String {
    form_urlencoded::byte_serialize(input.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("foo&bar=baz+1#2"), "foo%26bar%3Dbaz%2B1%232");
        assert_eq!(url_encode("café"), "caf%C3%A9");
        assert_eq!(
            url_encode("<script>alert(1)</script>"),
            "%3Cscript%3Ealert%281%29%3C%2Fscript%3E"
        );
    }

    #[test]
    fn test_build_query() {
        assert_eq!(
            build_query(&[("q", "hello world"), ("page", "2")]),
            "q=hello+world&page=2"
        );
        assert_eq!(
            build_query(&[("search", "cats & dogs"), ("filter", "safe/mode")]),
            "search=cats+%26+dogs&filter=safe%2Fmode"
        );
    }

    #[test]
    fn test_build_cookie() {
        assert_eq!(
            build_cookie(&[("safe", "1"), ("lang", "en")]),
            "safe=1; lang=en; "
        );
    }
}
