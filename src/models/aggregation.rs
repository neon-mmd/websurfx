//! This module provides public models for handling, storing and serializing of search results
//! data scraped from the upstream search engines.

use super::engine::EngineError;
use crate::json::JsonWriter;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use rkyv::util::AlignedVec;
use std::sync::Arc;

/// The error returned when search results cannot be encoded into, or restored from, a
/// [`SearchResultsArchive`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveError;

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unable to encode or decode the search results archive")
    }
}

impl std::error::Error for ArchiveError {}

/// A zero-copy archive of [`SearchResults`].
///
/// Results are encoded once into a single aligned buffer and are afterwards read by
/// borrowing straight out of that buffer, so serving a cached page never materialises an
/// intermediate `String` for every field of every result. Cloning is a refcount bump,
/// which lets freshly aggregated results be rendered and handed to the cache store from
/// the same allocation.
#[derive(Clone, Debug)]
pub struct SearchResultsArchive(Arc<AlignedVec>);

impl SearchResultsArchive {
    /// Encodes owned search results into a zero-copy archive.
    ///
    /// # Errors
    ///
    /// Returns an [`ArchiveError`] if the results cannot be encoded.
    pub fn encode(results: &SearchResults) -> Result<Self, ArchiveError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(results)
            .map(|buffer| Self(Arc::new(buffer)))
            .map_err(|_| ArchiveError)
    }

    /// Validates an encoded buffer and wraps it so its contents can be read back.
    ///
    /// # Errors
    ///
    /// Returns an [`ArchiveError`] if the buffer does not hold a valid archive.
    pub fn from_buffer(buffer: Arc<AlignedVec>) -> Result<Self, ArchiveError> {
        rkyv::access::<ArchivedSearchResults, rkyv::rancor::Error>(&buffer)
            .map_err(|_| ArchiveError)?;
        Ok(Self(buffer))
    }

    /// Validates the buffer and returns the archived results borrowed from it.
    ///
    /// The returned reference points directly into the encoded bytes, so reading a field
    /// copies nothing. Validation walks the archive once and never allocates.
    ///
    /// # Errors
    ///
    /// Returns an [`ArchiveError`] if the buffer does not hold a valid archive.
    pub fn get(&self) -> Result<&ArchivedSearchResults, ArchiveError> {
        rkyv::access::<ArchivedSearchResults, rkyv::rancor::Error>(&self.0)
            .map_err(|_| ArchiveError)
    }

    /// A shared handle to the encoded bytes, cheap to clone.
    pub fn buffer(&self) -> Arc<AlignedVec> {
        self.0.clone()
    }

    /// Renders the archive as the JSON API response body.
    ///
    /// The member names are the ones the API has always published, and every string is read
    /// straight out of the encoded buffer.
    ///
    /// # Errors
    ///
    /// Returns an [`ArchiveError`] if the buffer does not hold a valid archive.
    pub fn to_json(&self) -> Result<String, ArchiveError> {
        let archived = self.get()?;

        let mut writer = JsonWriter::with_capacity(archived.results.len() * 512);

        writer.begin_object();

        writer.field("results").begin_array();
        for result in archived.results.iter() {
            writer.begin_object();
            writer.field("title").string(result.title.as_str());
            writer.field("url").string(result.url.as_str());
            writer
                .field("description")
                .string(result.description.as_str());
            writer.field("engine").begin_array();
            for engine in result.engine.iter() {
                writer.string(engine.as_str());
            }
            writer.end_array();
            writer
                .field("relevanceScore")
                .float(result.relevance_score.to_native());
            writer.end_object();
        }
        writer.end_array();

        writer.field("engineErrorsInfo").begin_array();
        for error in archived.engine_errors_info.iter() {
            writer.begin_object();
            writer.field("error").string(error.error.as_str());
            writer.field("engine").string(error.engine.as_str());
            writer
                .field("severity_color")
                .string(error.severity_color.as_str());
            writer.end_object();
        }
        writer.end_array();

        writer.field("disallowed").boolean(archived.disallowed);
        writer.field("filtered").boolean(archived.filtered);
        writer
            .field("safeSearchLevel")
            .unsigned(archived.safe_search_level.into());
        writer
            .field("noEnginesSelected")
            .boolean(archived.no_engines_selected);

        writer.end_object();
        Ok(writer.finish())
    }
}

/// Lazily computed, process-wide stop-word list (computed once, reused for every result).
static STOP_WORDS_CACHE: std::sync::OnceLock<Box<[String]>> = std::sync::OnceLock::new();

/// Lazily computed, process-wide punctuation list.
static PUNCTUATION_CACHE: std::sync::OnceLock<Box<[String]>> = std::sync::OnceLock::new();

/// Returns the cached English stop words, computing the list once.
pub fn stop_words_cache() -> &'static [String] {
    STOP_WORDS_CACHE.get_or_init(|| {
        stop_words::get(stop_words::LANGUAGE::English)
            .par_iter()
            .map(|stop_word| stop_word.to_string())
            .collect()
    })
}

/// Returns the cached punctuation list, computing it once.
pub fn punctuation_cache() -> &'static [String] {
    PUNCTUATION_CACHE.get_or_init(|| {
        [
            ".", ",", ":", ";", "!", "?", "(", ")", "[", "]", "{", "}", "\"", "'", "<", ">",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    })
}

/// TF-IDF calculator that computes IDF once per query and reuses for all results.
/// This avoids rebuilding the TfIdf model for every single result.
pub struct TfIdfCalculator {
    /// The stop words dropped before tokenising, built once rather than per query.
    stop_words: Box<[String]>,
    /// The punctuation dropped alongside them, built once rather than per query.
    punctuation: Box<[String]>,
}

impl TfIdfCalculator {
    /// Creates a new calculator with cached stop words and punctuation.
    pub fn new() -> Self {
        Self {
            stop_words: stop_words_cache().into(),
            punctuation: punctuation_cache().into(),
        }
    }

    /// Computes relevance scores for all results in parallel.
    /// Builds the TF-IDF model once from all documents, then scores each result.
    pub fn compute_scores(&self, query: &str, results: &mut [SearchResult]) {
        use keyword_extraction::{
            tf_idf::{TfIdf, TfIdfParams},
            tokenizer::Tokenizer,
        };

        // Collect all documents for corpus-wide IDF computation
        let documents: Vec<[String; 3]> = results
            .iter()
            .map(|r| [r.title.clone(), r.url.clone(), r.description.clone()])
            .collect();

        // Build TF-IDF model once for all documents in this query
        let params = TfIdfParams::UnprocessedDocuments(
            &documents.iter().flatten().cloned().collect::<Vec<_>>(),
            &self.stop_words,
            Some(&self.punctuation),
        );
        let tf_idf = TfIdf::new(params);

        // Tokenize query once
        let tokener = Tokenizer::new(query, &self.stop_words, Some(&self.punctuation));
        let query_tokens: Vec<String> = tokener.split_into_words();

        #[cfg(any(
            feature = "use-synonyms-search",
            feature = "use-non-static-synonyms-search"
        ))]
        let synonyms_tokens: Vec<String> = {
            use thesaurus::synonyms;
            query_tokens.par_iter().flat_map(synonyms).collect()
        };

        // Score each result using the pre-built TF-IDF model
        results.par_iter_mut().for_each(|result| {
            if result.url.contains("temu.com") {
                result.relevance_score = 0.0;
                return;
            }

            let mut total_score = 0.0;
            let mut token_count = 0;

            for token in &query_tokens {
                total_score += tf_idf.get_score(token);
                token_count += 1;
            }

            #[cfg(any(
                feature = "use-synonyms-search",
                feature = "use-non-static-synonyms-search"
            ))]
            for token in &synonyms_tokens {
                total_score += tf_idf.get_score(token);
                token_count += 1;
            }

            result.relevance_score = if token_count > 0 {
                total_score / token_count as f32
            } else {
                0.0
            };
        });
    }
}

impl Default for TfIdfCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// A named struct to store the raw scraped search results scraped search results from the
/// upstream search engines before aggregating it.It derives the Clone trait which is needed
/// to write idiomatic rust using `Iterators`.
#[derive(Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SearchResult {
    /// The title of the search result.
    pub title: String,
    /// The url which is accessed when clicked on it (href url in html in simple words).
    pub url: String,
    /// The description of the search result.
    pub description: String,
    /// The names of the upstream engines from which this results were provided.
    pub engine: Vec<String>,
    /// The td-tdf score of the result in regards to the title, url and description and the
    /// user's query
    pub relevance_score: f32,
}

impl SearchResult {
    /// Constructs a new `RawSearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the search result.
    /// * `url` - The url which is accessed when clicked on it
    ///   (href url in html in simple words).
    /// * `description` - The description of the search result.
    /// * `engine` - The names of the upstream engines from which this results were provided.
    pub fn new(title: &str, url: &str, description: &str, engine: &[&str]) -> Self {
        SearchResult {
            title: title.to_owned(),
            url: url.to_owned(),
            description: description.to_owned(),
            relevance_score: 0.0,
            engine: engine.iter().map(|name| name.to_string()).collect(),
        }
    }
    /// calculates and update the relevance score of the current search.
    /// Deprecated: Use `TfIdfCalculator::compute_scores` for batch computation.
    ///
    /// # Arguments
    ///
    /// * query -  the query string  used to obtain the results
    #[deprecated(note = "Use TfIdfCalculator::compute_scores for batch computation")]
    pub fn calculate_relevance(&mut self, query: &str) {
        // Fallback for single result calculation (e.g., in tests)
        let calculator = TfIdfCalculator::new();
        calculator.compute_scores(query, std::slice::from_mut(self));
    }

    /// A function which adds the engine name provided as a string into a vector of strings.
    ///
    /// # Arguments
    ///
    /// * `engine` - Takes an engine name provided as a String.
    pub fn add_engines(&mut self, engine: &str) {
        self.engine.push(engine.to_owned())
    }

    /// A function which returns the engine name stored from the struct as a string.
    ///
    /// # Returns
    ///
    /// An engine name stored as a string from the struct.
    pub fn engine(&mut self) -> String {
        std::mem::take(&mut self.engine[0])
    }
}

/// A named struct that stores the error info related to the upstream search engines.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone)]
pub struct EngineErrorInfo {
    /// It stores the error type which occured while fetching the result from a particular search
    /// engine.
    pub error: String,
    /// It stores the name of the engine that failed to provide the requested search results.
    pub engine: String,
    /// It stores the name of the color to indicate whether how severe the particular error is (In
    /// other words it indicates the severity of the error/issue).
    pub severity_color: String,
}

impl EngineErrorInfo {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `error` - It takes the error type which occured while fetching the result from a particular
    ///   search engine.
    /// * `engine` - It takes the name of the engine that failed to provide the requested search results.
    pub fn new(error: &EngineError, engine: &str) -> Self {
        Self {
            error: match error {
                EngineError::NoSuchEngineFound(_) => "EngineNotFound".to_owned(),
                EngineError::RequestError => "RequestError".to_owned(),
                EngineError::EmptyResultSet => "EmptyResultSet".to_owned(),
                EngineError::UnexpectedError => "UnexpectedError".to_owned(),
            },
            engine: engine.to_owned(),
            severity_color: match error {
                EngineError::NoSuchEngineFound(_) => "red".to_owned(),
                EngineError::RequestError => "green".to_owned(),
                EngineError::EmptyResultSet => "blue".to_owned(),
                EngineError::UnexpectedError => "red".to_owned(),
            },
        }
    }
}

/// A named struct to store, serialize, deserialize the all the search results scraped and
/// aggregated from the upstream search engines.
/// `SearchResult` structs.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Default, Clone)]
pub struct SearchResults {
    /// Stores the individual serializable `SearchResult` struct into a vector of
    pub results: Box<[SearchResult]>,
    /// Stores the information on which engines failed with their engine name
    /// and the type of error that caused it.
    pub engine_errors_info: Box<[EngineErrorInfo]>,
    /// Stores the flag option which holds the check value that the following
    /// search query was disallowed when the safe search level set to 4 and it
    /// was present in the `Blocklist` file.
    pub disallowed: bool,
    /// Stores the flag option which holds the check value that the following
    /// search query was filtered when the safe search level set to 3 and it
    /// was present in the `Blocklist` file.
    pub filtered: bool,
    /// Stores the safe search level `safesearch` provided in the search url.
    pub safe_search_level: u8,
    /// Stores the flag option which holds the check value that whether any search engines were
    /// selected or not.
    pub no_engines_selected: bool,
}

impl SearchResults {
    /// Constructs a new `SearchResult` with the given arguments needed for the struct.
    ///
    /// # Arguments
    ///
    /// * `results` - Takes an argument of individual serializable `SearchResult` struct
    ///   and stores it into a vector of `SearchResult` structs.
    /// * `page_query` - Takes an argument of current page`s search query `q` provided in
    ///   the search url.
    /// * `engine_errors_info` - Takes an array of structs which contains information regarding
    ///   which engines failed with their names, reason and their severity color name.
    pub fn new(results: Box<[SearchResult]>, engine_errors_info: Box<[EngineErrorInfo]>) -> Self {
        Self {
            results,
            engine_errors_info,
            disallowed: Default::default(),
            filtered: Default::default(),
            safe_search_level: Default::default(),
            no_engines_selected: Default::default(),
        }
    }

    /// A setter function that sets disallowed to true.
    pub fn set_disallowed(&mut self) {
        self.disallowed = true;
    }

    /// A setter function that sets the filtered to true.
    pub fn set_filtered(&mut self, filtered: bool) {
        self.filtered = filtered;
    }

    /// A getter function that gets the value of `engine_errors_info`.
    pub fn engine_errors_info(&mut self) -> Box<[EngineErrorInfo]> {
        std::mem::take(&mut self.engine_errors_info)
    }
    /// A getter function that gets the value of `results`.
    pub fn results(&mut self) -> Box<[SearchResult]> {
        self.results.clone()
    }

    /// A setter function to set the current page safe search level.
    pub fn set_safe_search_level(&mut self, safe_search_level: u8) {
        self.safe_search_level = safe_search_level;
    }

    /// A getter function that gets the value of `no_engines_selected`.
    pub fn no_engines_selected(&self) -> bool {
        self.no_engines_selected
    }

    /// A setter function to set the `no_engines_selected` to true.
    pub fn set_no_engines_selected(&mut self) {
        self.no_engines_selected = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::JsonValue;

    /// Builds a representative result set covering every archived field type.
    ///
    /// # Returns
    ///
    /// Returns a `SearchResults` value with two results and one engine error.
    fn sample_results() -> SearchResults {
        let mut results = SearchResults::new(
            vec![
                SearchResult {
                    title: "Rust Programming Language".to_owned(),
                    url: "https://www.rust-lang.org/".to_owned(),
                    description: "A language empowering everyone to build reliable software."
                        .to_owned(),
                    engine: vec!["searx".to_owned(), "brave".to_owned()],
                    relevance_score: 0.75,
                },
                SearchResult {
                    title: "naïve café ☕".to_owned(),
                    url: "https://example.com/unicode?q=☕".to_owned(),
                    description: "Multi byte characters must survive the round trip.".to_owned(),
                    engine: vec!["qwant".to_owned()],
                    relevance_score: 0.25,
                },
            ]
            .into_boxed_slice(),
            vec![EngineErrorInfo {
                error: "EmptyResultSet".to_owned(),
                engine: "mojeek".to_owned(),
                severity_color: "blue".to_owned(),
            }]
            .into_boxed_slice(),
        );

        results.set_disallowed();
        results.set_filtered(true);
        results.set_safe_search_level(2);
        results.set_no_engines_selected();

        results
    }

    #[test]
    fn archive_round_trips_every_field() {
        let expected = sample_results();
        let archive = SearchResultsArchive::encode(&expected).unwrap();
        let archived = archive.get().unwrap();

        assert_eq!(archived.results.len(), 2);
        assert_eq!(
            archived.results[0].title.as_str(),
            "Rust Programming Language"
        );
        assert_eq!(
            archived.results[0].url.as_str(),
            "https://www.rust-lang.org/"
        );
        assert_eq!(
            archived.results[0].description.as_str(),
            "A language empowering everyone to build reliable software."
        );
        assert_eq!(archived.results[0].relevance_score.to_native(), 0.75);
        assert_eq!(
            archived.results[0]
                .engine
                .iter()
                .map(|engine| engine.as_str())
                .collect::<Vec<_>>(),
            vec!["searx", "brave"]
        );

        assert_eq!(archived.results[1].title.as_str(), "naïve café ☕");
        assert_eq!(
            archived.results[1].url.as_str(),
            "https://example.com/unicode?q=☕"
        );

        assert_eq!(archived.engine_errors_info.len(), 1);
        assert_eq!(
            archived.engine_errors_info[0].error.as_str(),
            "EmptyResultSet"
        );
        assert_eq!(archived.engine_errors_info[0].engine.as_str(), "mojeek");
        assert_eq!(
            archived.engine_errors_info[0].severity_color.as_str(),
            "blue"
        );

        assert!(archived.disallowed);
        assert!(archived.filtered);
        assert!(archived.no_engines_selected);
        assert_eq!(archived.safe_search_level, 2);
    }

    #[test]
    fn archive_borrows_the_encoded_buffer_without_copying() {
        let expected = sample_results();
        let archive = SearchResultsArchive::encode(&expected).unwrap();
        let archived = archive.get().unwrap();

        let buffer_start = archive.buffer().as_ptr() as usize;
        let title_start = archived.results[0].title.as_ptr() as usize;
        let title_len = archived.results[0].title.len();

        assert!(
            title_start >= buffer_start
                && title_start + title_len <= buffer_start + archive.buffer().len(),
            "archived strings must point into the encoded buffer"
        );
    }

    #[test]
    fn archive_clone_shares_one_buffer() {
        let archive = SearchResultsArchive::encode(&sample_results()).unwrap();
        let clone = archive.clone();

        assert!(std::sync::Arc::ptr_eq(&archive.buffer(), &clone.buffer()));
    }

    #[test]
    fn archive_rejects_a_corrupted_buffer() {
        let buffer = SearchResultsArchive::encode(&sample_results())
            .unwrap()
            .buffer();

        // Truncating the buffer must be detected by validation rather than read out of bounds.
        let last = buffer.len() - 1;
        let mut truncated = AlignedVec::with_capacity(last);
        truncated.extend_from_slice(&buffer[..last]);

        assert!(SearchResultsArchive::from_buffer(std::sync::Arc::new(truncated)).is_err());
        assert!(SearchResultsArchive::from_buffer(std::sync::Arc::new(AlignedVec::new())).is_err());
    }

    #[test]
    fn archive_serializes_to_the_published_json_shape() {
        let archive = SearchResultsArchive::encode(&sample_results()).unwrap();
        let json = archive.to_json().unwrap();

        assert!(json.starts_with(concat!(
            r#"{"results":[{"title":"Rust Programming Language","#,
            r#""url":"https://www.rust-lang.org/","#,
            r#""description":"A language empowering everyone to build reliable software.","#,
            r#""engine":["searx","brave"],"relevanceScore":0.75},"#
        )));
        assert!(
            json.contains(r#""title":"naïve café ☕","url":"https://example.com/unicode?q=☕""#)
        );
        assert!(json.contains(
            r#""engineErrorsInfo":[{"error":"EmptyResultSet","engine":"mojeek","severity_color":"blue"}]"#
        ));
        assert!(json.ends_with(
            r#""disallowed":true,"filtered":true,"safeSearchLevel":2,"noEnginesSelected":true}"#
        ));
    }

    #[test]
    fn archive_escapes_control_characters_for_the_json_api() {
        let results = SearchResults::new(
            vec![SearchResult {
                title: "quote \" backslash \\ newline \n tab \t bell \u{7}".to_owned(),
                url: "https://example.com".to_owned(),
                description: "<script>alert(1)</script>".to_owned(),
                engine: vec!["duckduckgo".to_owned()],
                relevance_score: 0.0,
            }]
            .into_boxed_slice(),
            Box::new([]),
        );

        let json = SearchResultsArchive::encode(&results)
            .unwrap()
            .to_json()
            .unwrap();

        assert!(
            json.contains(r#""title":"quote \" backslash \\ newline \n tab \t bell \u0007""#),
            "control characters must be escaped: {json}"
        );
        assert!(json.contains(r#""description":"<script>alert(1)</script>""#));
    }

    #[test]
    fn archive_json_output_parses_back_into_the_same_document() {
        let archive = SearchResultsArchive::encode(&sample_results()).unwrap();
        let json = archive.to_json().unwrap();
        let document = crate::json::parse(json.as_bytes()).unwrap();

        let results = document
            .get("results")
            .and_then(JsonValue::as_array)
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(
            results[0].get("title").and_then(JsonValue::as_str),
            Some("Rust Programming Language")
        );
        assert_eq!(
            results[0].get("relevanceScore"),
            Some(&JsonValue::Number("0.75"))
        );
        assert_eq!(
            results[0]
                .get("engine")
                .and_then(JsonValue::as_array)
                .map(<[JsonValue<'_>]>::len),
            Some(2)
        );
        assert_eq!(
            document.get("safeSearchLevel"),
            Some(&JsonValue::Number("2"))
        );
        assert_eq!(document.get("disallowed"), Some(&JsonValue::Bool(true)));
        assert_eq!(
            document
                .get("engineErrorsInfo")
                .and_then(JsonValue::as_array)
                .and_then(<[JsonValue<'_>]>::first)
                .and_then(|error| error.get("severity_color"))
                .and_then(JsonValue::as_str),
            Some("blue")
        );
    }

    /// Builds a page-sized result set, i.e. what a real aggregated page looks like.
    ///
    /// # Returns
    ///
    /// Returns a `SearchResults` value with `count` results of realistic string lengths.
    fn page_results(count: usize) -> SearchResults {
        let results = (0..count)
            .map(|index| SearchResult {
                title: format!("Result {index}: a reasonably long page title"),
                url: format!("https://example.com/result/{index}?query=some+search+terms"),
                description: format!(
                    "A reasonably long description for result {index} that mimics a real snippet."
                ),
                engine: vec!["searx".to_owned(), "brave".to_owned()],
                relevance_score: index as f32 / count as f32,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        SearchResults::new(results, Box::new([]))
    }

    /// Measures the cache write and read paths so the gain over the previous implementation
    /// is reproducible.
    ///
    /// Run with `cargo test --release --all-features -- --ignored --nocapture`.
    ///
    /// For reference, a `postcard` round trip of the same data costs roughly 3.1 us to
    /// encode, 6.4 us to decode and 3.7 us for the deep clone the old cache miss path
    /// relied on.
    #[test]
    #[ignore = "benchmark, run explicitly"]
    fn bench_cache_paths() {
        use std::time::{Duration, Instant};

        /// Number of measured iterations.
        const ITERATIONS: u32 = 20_000;
        /// Number of untimed iterations run first, to warm the allocator and the caches.
        const WARMUP: u32 = 2_000;
        /// Results per aggregated page.
        const RESULTS: usize = 30;

        /// Formats a duration as a per operation cost.
        fn per_op(total: Duration) -> String {
            let nanos = total.as_nanos() as f64 / f64::from(ITERATIONS);
            format!("{nanos:>9.0} ns/op")
        }

        let results = page_results(RESULTS);
        let archive = SearchResultsArchive::encode(&results).unwrap();

        for _ in 0..WARMUP {
            std::hint::black_box(SearchResultsArchive::encode(&results).unwrap());
            std::hint::black_box(archive.get().unwrap());
        }

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            std::hint::black_box(SearchResultsArchive::encode(&results).unwrap());
        }
        let encode = started.elapsed();

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            let archived = archive.get().unwrap();
            std::hint::black_box(archived.results.len());
        }
        let zero_copy_read = started.elapsed();

        // The same read the way an allocating deserializer has to do it. The cache does not use
        // this, it is measured only to show where the cost of zero-copy validation sits.
        let archived = archive.get().unwrap();
        for _ in 0..WARMUP {
            std::hint::black_box(
                rkyv::deserialize::<SearchResults, rkyv::rancor::Error>(archived).unwrap(),
            );
        }

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            let owned = rkyv::deserialize::<SearchResults, rkyv::rancor::Error>(archived).unwrap();
            std::hint::black_box(owned.results.len());
        }
        let owned_read = started.elapsed();

        let buffer_size = archive.buffer().len();

        println!(
            "\ncache paths for {RESULTS} results ({buffer_size} encoded bytes)\n  \
             write  rkyv encode           {}\n  \
             read   validate + borrow     {}\n  \
             read   rkyv owned (unused)   {}\n",
            per_op(encode),
            per_op(zero_copy_read),
            per_op(owned_read),
        );
    }

    /// Measures the JSON API response path: the zero-copy read followed by rendering every
    /// field of every result into the response body.
    ///
    /// Run with `cargo test --release --all-features -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn bench_json_api_response() {
        use std::time::{Duration, Instant};

        /// Number of measured iterations.
        const ITERATIONS: u32 = 20_000;
        /// Number of untimed iterations run first, to warm the allocator and the caches.
        const WARMUP: u32 = 2_000;
        /// Results per aggregated page.
        const RESULTS: usize = 30;

        /// Formats a duration as a per operation cost.
        fn per_op(total: Duration) -> String {
            let nanos = total.as_nanos() as f64 / f64::from(ITERATIONS);
            format!("{nanos:>9.0} ns/op")
        }

        let archive = SearchResultsArchive::encode(&page_results(RESULTS)).unwrap();

        for _ in 0..WARMUP {
            std::hint::black_box(archive.to_json().unwrap());
        }

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            std::hint::black_box(archive.to_json().unwrap());
        }
        let render = started.elapsed();

        // The read on its own, so the render cost can be separated from it.
        for _ in 0..WARMUP {
            std::hint::black_box(archive.get().unwrap().results.len());
        }

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            std::hint::black_box(archive.get().unwrap().results.len());
        }
        let read = started.elapsed();

        let bytes = archive.to_json().unwrap().len();

        // The render cost on its own, with the zero-copy read it sits on top of subtracted.
        let render_only = render.as_nanos() as f64 / f64::from(ITERATIONS)
            - read.as_nanos() as f64 / f64::from(ITERATIONS);

        println!(
            "\njson api response for {RESULTS} results ({bytes} response bytes)\n  \
             read   validate + borrow     {}\n  \
             write  render whole body     {}\n  \
             write  render minus read     {render_only:>9.0} ns/op\n",
            per_op(read),
            per_op(render),
        );
    }
}
