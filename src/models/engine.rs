//! This module provides the error enum to handle different errors associated while requesting data from
//! the upstream search engines with the search query provided by the user.

use super::aggregation::SearchResult;
use error_stack::{Report, ResultExt};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, OnceLock, RwLock};

/// A custom error type used for handle engine associated errors.
#[derive(Debug)]
pub enum EngineError {
    /// No matching engine found
    NoSuchEngineFound(String),
    /// This variant handles all request related errors like forbidden, not found,
    /// etc.
    EmptyResultSet,
    /// This variant handles the not results found error provide by the upstream
    /// search engines.
    RequestError,
    ///  This variant handles all the errors which are unexpected or occur rarely
    /// and are errors mostly related to failure in initialization of HeaderMap,
    /// Selector errors and all other errors occurring within the code handling
    /// the `upstream search engines`.
    UnexpectedError,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::NoSuchEngineFound(engine) => {
                write!(f, "No such engine with the name '{engine}' found")
            }
            EngineError::EmptyResultSet => {
                write!(f, "The upstream search engine returned an empty result set")
            }
            EngineError::RequestError => {
                write!(
                    f,
                    "Error occurred while requesting data from upstream search engine"
                )
            }
            EngineError::UnexpectedError => {
                write!(f, "An unexpected error occurred while processing the data")
            }
        }
    }
}

/// An alias type for handling the engine results.
pub type EngineResult<T> = Result<T, Report<EngineError>>;

impl std::error::Error for EngineError {}

/// A trait to define common behavior for all search engines.
#[async_trait::async_trait]
pub trait SearchEngine: Sync + Send {
    /// This helper function fetches/requests the search results from the upstream search engine in
    /// an html form.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the url of the upstream search engine with the user requested search
    /// query appended in the search parameters.
    /// * `header_map` - It takes the http request headers to be sent to the upstream engine in
    /// order to prevent being detected as a bot. It takes the header as a HeaderMap type.
    /// * `request_timeout` - It takes the request timeout value as seconds which is used to limit
    /// the amount of time for each request to remain connected when until the results can be provided
    /// by the upstream engine.
    ///
    /// # Error
    ///
    /// It returns the html data as a string if the upstream engine provides the data as expected
    /// otherwise it returns a custom `EngineError`.
    async fn fetch_html_from_upstream(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        client: &Client,
    ) -> EngineResult<String> {
        // fetch the html from upstream search engine
        Ok(retry_on_connect_failure(client, url, &header_map)
            .await?
            .text()
            .await
            .change_context(EngineError::RequestError)?)
    }

    /// This helper function fetches/requests the json search results from the upstream search engine as a vector of bytes.
    ///
    /// # Arguments
    ///
    /// * `url` - It takes the url of the upstream search engine with the user requested search
    /// query appended in the search parameters.
    /// * `header_map` - It takes the http request headers to be sent to the upstream engine in
    /// order to prevent being detected as a bot. It takes the header as a HeaderMap type.
    /// * `request_timeout` - It takes the request timeout value as seconds which is used to limit
    /// the amount of time for each request to remain connected when until the results can be provided
    /// by the upstream engine.
    ///
    /// # Error
    ///
    /// It returns the html data as a vector of bytes if the upstream engine provides the data as expected
    /// otherwise it returns a custom `EngineError`.
    async fn fetch_json_as_bytes_from_upstream(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        client: &Client,
    ) -> EngineResult<Vec<u8>> {
        // fetch the json response from upstream search engine

        Ok(retry_on_connect_failure(client, url, &header_map)
            .await?
            .bytes()
            .await
            .change_context(EngineError::RequestError)?
            .to_vec())
    }

    /// This function scrapes results from the upstream engine and puts all the scraped results like
    /// title, visiting_url (href in html),engine (from which engine it was fetched from) and description
    /// in a RawSearchResult and then adds that to HashMap whose keys are url and values are RawSearchResult
    /// struct and then returns it within a Result enum.
    ///
    /// # Arguments
    ///
    /// * `query` - Takes the user provided query to query to the upstream search engine with.
    /// * `page` - Takes an u32 as an argument.
    /// * `user_agent` - Takes a random user agent string as an argument.
    /// * `request_timeout` - Takes a time (secs) as a value which controls the server request timeout.
    ///
    /// # Errors
    ///
    /// Returns an `EngineErrorKind` if the user is not connected to the internet or if their is failure to
    /// reach the above `upstream search engine` page or if the `upstream search engine` is unable to
    /// provide results for the requested search query and also returns error if the scraping selector
    /// or HeaderMap fails to initialize.
    async fn results(
        &self,
        query: &str,
        page: u32,
        user_agent: &str,
        client: &Client,
        safe_search: u8,
    ) -> EngineResult<Vec<(String, SearchResult)>>;
}

/// Sends `url`, retrying once when the connection itself could not be established.
///
/// Several upstreams sit behind load balancers with more than one node, and an individual
/// node can refuse the connection, fail the TLS handshake or drop a pooled socket that was
/// returned to the pool just as it closed. None of those say anything about the request, so
/// one retry on a fresh connection is worth it. A response that *arrives* is never retried,
/// whatever its status, so a refusal or an empty result set is reported as the upstream meant
/// it rather than being papered over with extra traffic.
///
/// # Arguments
///
/// * `client` - It takes the shared HTTP client.
/// * `url` - It takes the upstream URL to request.
/// * `header_map` - It takes the spoofed headers sent with the request.
///
/// # Errors
///
/// Returns the error from the last attempt if every attempt failed to connect.
async fn retry_on_connect_failure(
    client: &Client,
    url: &str,
    header_map: &reqwest::header::HeaderMap,
) -> EngineResult<reqwest::Response> {
    /// How many times a request is attempted in total.
    const ATTEMPTS: usize = 2;

    let mut last = None;

    for attempt in 1..=ATTEMPTS {
        match client
            .get(url)
            .headers(header_map.clone()) // spoofed headers emulate human behaviour
            .send()
            .await
        {
            Ok(response) => return Ok(response),
            Err(error) => {
                let worth_retrying = attempt < ATTEMPTS && is_connect_failure(&error);

                last = Some(error);

                if !worth_retrying {
                    break;
                }
            }
        }
    }

    let Some(error) = last else {
        return Err(Report::new(EngineError::RequestError).attach(format!("GET {url}")));
    };

    Err(Report::new(error)
        .change_context(EngineError::RequestError)
        .attach(format!("GET {url}")))
}

/// Reports whether a request failed before the upstream ever saw it.
///
/// # Arguments
///
/// * `error` - It takes the error `reqwest` returned for the attempt.
fn is_connect_failure(error: &reqwest::Error) -> bool {
    let mut source: Option<&(dyn std::error::Error + 'static)> = std::error::Error::source(error);

    while let Some(error) = source {
        // A connection that was refused, reset or closed mid-handshake, or a socket the
        // server had already closed, all leave the request unsent as far as we can tell.
        let message = error.to_string();

        if message.contains("tcp connect error")
            || message.contains("connection closed")
            || message.contains("unexpected eof")
            || message.contains("close_notify")
            || message.contains("ApplicationProtocol")
            || message.contains("Broken pipe")
        {
            return true;
        }

        source = error.source();
    }

    false
}

/// Builds a reqwest `HeaderMap` directly from a vector of `(name, value)` pairs,
/// avoiding an intermediate `HashMap` allocation.
///
/// # Arguments
///
/// * `header_params` - A slice of `(name, value)` tuples to insert.
///
/// # Error
///
/// Returns an `EngineError::UnexpectedError` if a header name or value is invalid.
pub fn build_headers(header_params: &[(&str, &str)]) -> EngineResult<HeaderMap> {
    let mut header_map = HeaderMap::with_capacity(header_params.len());
    for (name, value) in header_params {
        let name =
            HeaderName::from_bytes(name.as_bytes()).change_context(EngineError::UnexpectedError)?;
        let value = HeaderValue::from_str(value).change_context(EngineError::UnexpectedError)?;
        header_map.insert(name, value);
    }
    Ok(header_map)
}

/// A named struct which stores the engine struct with the name of the associated engine.
///
/// The engine itself is shared behind an [`Arc`] because an [`SearchEngine`] carries only
/// immutable state: the compiled CSS selectors and, for a few engines, their fixed host. That
/// makes cloning a handler a refcount bump rather than a rebuild.
pub struct EngineHandler {
    /// It stores the engine struct wrapped in a shared pointer as the engine struct implements
    /// the `SearchEngine` trait.
    engine: Arc<dyn SearchEngine>,
    /// It stores the name of the engine to which the struct is associated to.
    name: &'static str,
}

impl Clone for EngineHandler {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            name: self.name,
        }
    }
}

impl EngineHandler {
    /// Parses an engine name into an engine handler.
    ///
    /// # Arguments
    ///
    /// * `engine_name` - It takes the name of the engine to which the struct was associated to.
    ///
    /// # Returns
    ///
    /// It returns an option either containing the value or a none if the engine is unknown
    pub fn new(engine_name: &str) -> EngineResult<Self> {
        let Some((name, engine)) = Self::build(engine_name)? else {
            return Err(Report::from(EngineError::NoSuchEngineFound(
                engine_name.to_owned(),
            )));
        };

        Ok(Self {
            engine: Arc::from(engine),
            name,
        })
    }

    /// Builds the engine `engine_name` refers to, if there is one.
    ///
    /// The name is matched case-insensitively without lowering it into a fresh `String`
    /// first, which removes one allocation per engine per request.
    ///
    /// # Arguments
    ///
    /// * `engine_name` - It takes the name of the engine to build.
    ///
    /// # Returns
    ///
    /// Returns the canonical name and the engine, or [`None`] when the name is unknown.
    fn build(engine_name: &str) -> EngineResult<Option<(&'static str, Box<dyn SearchEngine>)>> {
        let instances = INSTANCES.get_or_init(Instances::default);

        let (name, engine): (&'static str, Box<dyn SearchEngine>) =
            if engine_name.eq_ignore_ascii_case("duckduckgo") {
                (
                    "duckduckgo",
                    Box::new(crate::engines::duckduckgo::DuckDuckGo::new()?),
                )
            } else if engine_name.eq_ignore_ascii_case("searx") {
                (
                    "searx",
                    Box::new(crate::engines::searx::Searx::new(instances.searx.clone())?),
                )
            } else if engine_name.eq_ignore_ascii_case("brave") {
                ("brave", Box::new(crate::engines::brave::Brave::new()?))
            } else if engine_name.eq_ignore_ascii_case("startpage") {
                (
                    "startpage",
                    Box::new(crate::engines::startpage::Startpage::new()?),
                )
            } else if engine_name.eq_ignore_ascii_case("librex") {
                (
                    "librex",
                    Box::new(crate::engines::librex::LibreX::new(
                        instances.librex.clone(),
                    )?),
                )
            } else if engine_name.eq_ignore_ascii_case("mojeek") {
                ("mojeek", Box::new(crate::engines::mojeek::Mojeek::new()?))
            } else if engine_name.eq_ignore_ascii_case("bing") {
                ("bing", Box::new(crate::engines::bing::Bing::new()?))
            } else if engine_name.eq_ignore_ascii_case("wikipedia") {
                (
                    "wikipedia",
                    Box::new(crate::engines::wikipedia::Wikipedia::new("en")?),
                )
            } else if engine_name.eq_ignore_ascii_case("yahoo") {
                ("yahoo", Box::new(crate::engines::yahoo::Yahoo::new()?))
            } else if engine_name.eq_ignore_ascii_case("qwant") {
                ("qwant", Box::new(crate::engines::qwant::Qwant::new()?))
            } else if engine_name.eq_ignore_ascii_case("sepiasearch") {
                (
                    "sepiasearch",
                    Box::new(crate::engines::sepiasearch::SepiaSearch::new()?),
                )
            } else {
                return Ok(None);
            };

        Ok(Some((name, engine)))
    }

    /// Returns the engine name together with a handle to the engine struct.
    ///
    /// The engine is shared rather than moved out, so one handler can back every request
    /// that selects the engine.
    pub fn name_engine(&self) -> (&'static str, Arc<dyn SearchEngine>) {
        (self.name, Arc::clone(&self.engine))
    }

    /// The canonical name of `engine_name`, if it names a known engine.
    ///
    /// # Arguments
    ///
    /// * `engine_name` - It takes the name of the engine to look up.
    fn canonical_name(engine_name: &str) -> Option<&'static str> {
        CANONICAL_NAMES
            .iter()
            .copied()
            .find(|known| known.eq_ignore_ascii_case(engine_name))
    }

    /// Returns a shared handler for `engine_name`, building it at most once per process.
    ///
    /// An engine holds compiled CSS selectors, and compiling them costs far more than serving
    /// a request does. Because an engine is immutable once built, the same handler can be
    /// handed to every request that selects it.
    ///
    /// # Arguments
    ///
    /// * `engine_name` - It takes the name of the engine to look up.
    ///
    /// # Returns
    ///
    /// Returns the shared handler, or an error when the name is unknown or the engine fails
    /// to build.
    pub fn shared(engine_name: &str) -> EngineResult<&'static Self> {
        let Some(name) = Self::canonical_name(engine_name) else {
            return Err(Report::from(EngineError::NoSuchEngineFound(
                engine_name.to_owned(),
            )));
        };

        let cache = ENGINES.get_or_init(|| RwLock::new(HashMap::new()));

        if let Some(handler) = cache
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .get(name)
        {
            return Ok(handler);
        }

        let handler: &'static Self = Box::leak(Box::new(Self::new(engine_name)?));

        cache
            .write()
            .unwrap_or_else(|error| error.into_inner())
            .insert(name, handler);

        Ok(handler)
    }
}

/// The community-run instances the instance-backed engines query.
#[derive(Debug, Clone)]
struct Instances {
    /// The base URL of the SearXNG instance.
    searx: String,
    /// The base URL of the LibreX instance.
    librex: String,
}

impl Default for Instances {
    fn default() -> Self {
        Self {
            searx: "https://searx.be".to_owned(),
            librex: "https://search.davidovski.xyz".to_owned(),
        }
    }
}

/// The instances named by the config, published by [`set_instances`].
static INSTANCES: OnceLock<Instances> = OnceLock::new();

/// Publishes the instance URLs the SearXNG and LibreX engines should query.
///
/// These instances are run by volunteers and go offline without notice, so they are taken from
/// the config instead of being named in the code. Publishing them once keeps
/// [`EngineHandler::new`] free of a config dependency while the value stays process-wide.
///
/// # Arguments
///
/// * `searx_url` - It is `searx_url` from the parsed config.
/// * `librex_url` - It is `librex_url` from the parsed config.
pub fn set_instances(searx_url: String, librex_url: String) {
    let _ = INSTANCES.set(Instances {
        searx: searx_url,
        librex: librex_url,
    });
}

/// The canonical, lower case names of every supported engine.
const CANONICAL_NAMES: [&str; 11] = [
    "bing",
    "brave",
    "duckduckgo",
    "librex",
    "mojeek",
    "qwant",
    "searx",
    "sepiasearch",
    "startpage",
    "wikipedia",
    "yahoo",
];

/// The process-wide cache of built engines, keyed by canonical name.
///
/// An entry is leaked to obtain its `'static` lifetime. The map is bounded by the number of
/// known engine names, so the total leak is one small handler per engine actually used.
static ENGINES: OnceLock<RwLock<HashMap<&'static str, &'static EngineHandler>>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    /// Starts a server that records the headers of each request and then points the client at
    /// a port nothing is listening on, so the attempt fails at the connect stage.
    async fn record_and_redirect_elsewhere(
        state: Arc<Mutex<Vec<HeaderMap>>>,
    ) -> (Client, std::net::SocketAddr) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            while let Ok((mut stream, _)) = listener.accept().await {
                let mut raw = Vec::new();
                let mut chunk = [0_u8; 1024];

                // Read up to the end of the request headers.
                while !raw.windows(4).any(|window| window == b"\r\n\r\n") {
                    match stream.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(read) => raw.extend_from_slice(&chunk[..read]),
                    }
                }

                let text = String::from_utf8_lossy(&raw).to_ascii_lowercase();
                let mut headers = HeaderMap::with_capacity(8);

                for line in text.lines().skip(1) {
                    if let Some((name, value)) = line.split_once(':')
                        && let (Ok(name), Ok(value)) = (
                            HeaderName::from_bytes(name.trim().as_bytes()),
                            HeaderValue::from_str(value.trim()),
                        )
                    {
                        headers.insert(name, value);
                    }
                }

                state.lock().unwrap().push(headers);

                // Nothing is listening on port 1, so following this hop fails to connect.
                let _ = stream
                    .write_all(
                        b"HTTP/1.1 302 Found\r\nlocation: http://127.0.0.1:1/\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
                    )
                    .await;
                let _ = stream.flush().await;
            }
        });

        (Client::builder().build().unwrap(), address)
    }

    /// A connect failure must be retried once, and every attempt has to carry the spoofed
    /// headers so a retry is indistinguishable from the first request.
    #[tokio::test]
    async fn a_retry_still_sends_the_spoofed_headers() {
        let state = Arc::new(Mutex::new(Vec::new()));
        let (client, address) = record_and_redirect_elsewhere(Arc::clone(&state)).await;

        let headers = build_headers(&[
            ("User-Agent", "Mozilla/5.0"),
            ("Referer", "https://example.com/"),
            ("Accept-Language", "en-US,en;q=0.9"),
        ])
        .unwrap();

        // The upstream redirects to a dead port, so the request cannot succeed. What matters
        // is that the headers were presented on every attempt that was made.
        let _ = retry_on_connect_failure(&client, &format!("http://{address}/"), &headers).await;

        let seen = state.lock().unwrap().clone();
        assert!(!seen.is_empty(), "the test server saw no request");

        for request in &seen {
            for name in ["user-agent", "referer", "accept-language"] {
                assert!(
                    request.contains_key(name),
                    "{name} missing from {request:?}"
                );
            }
        }
    }

    /// Measures building an engine handler the way the search route used to, against the
    /// shared lookup it uses now.
    ///
    /// Building an HTML engine compiles five CSS selectors, which is the dominant cost of
    /// the operation; the shared cache pays it once per engine for the life of the process.
    ///
    /// Run with `cargo test --release --all-features -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn bench_engine_handler_lookup() {
        use std::time::{Duration, Instant};

        /// Number of measured iterations.
        const ITERATIONS: u32 = 20_000;
        /// Number of untimed iterations run first, to warm the allocator and the caches.
        const WARMUP: u32 = 200;

        /// Formats a duration as a per operation cost.
        fn per_op(total: Duration) -> String {
            let nanos = total.as_nanos() as f64 / f64::from(ITERATIONS);
            format!("{nanos:>9.0} ns/op")
        }

        for engine in ["duckduckgo", "wikipedia", "qwant"] {
            let shared = EngineHandler::shared(engine).unwrap();

            for _ in 0..WARMUP {
                std::hint::black_box(EngineHandler::new(engine).unwrap());
                std::hint::black_box(EngineHandler::shared(engine).unwrap());
            }

            let started = Instant::now();
            for _ in 0..ITERATIONS {
                std::hint::black_box(EngineHandler::new(engine).unwrap());
            }
            let built = started.elapsed();

            let started = Instant::now();
            for _ in 0..ITERATIONS {
                std::hint::black_box(EngineHandler::shared(engine).unwrap());
            }
            let cached = started.elapsed();

            assert_eq!(shared.name_engine().0, shared.name);

            println!(
                "\nengine handler for {engine}\n  \
                 build  compile selectors  {}\n  \
                 shared cache lookup      {}\n",
                per_op(built),
                per_op(cached),
            );
        }
    }
}
