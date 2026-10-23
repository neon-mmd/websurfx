//! This module provides the functionality to generate and select random user agent strings.
//!
//! A curated list of realistic user agents is statically compiled into the binary to avoid
//! runtime HTTP fetching, eliminating crashes from upstream failures or empty responses.

/// A static slice of default user agent strings shipped directly in the binary.
///
/// This avoids runtime network requests and ensures realistic browser user agents
/// are immediately available offline and during startup without external dependencies.
pub static DEFAULT_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.7; rv:135.0) Gecko/20100101 Firefox/135.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 Edg/133.0.0.0",
];

/// A static fallback user agent string used when no user agents are available.
pub static FALLBACK_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0";

/// An error indicating that user agent generation or selection failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserAgentError {
    /// Error indicating that the user agent list is empty or no user agent is available.
    NoUserAgentsAvailable,
    /// Error indicating an HTTP error occurred while fetching user agents.
    HttpError(u16),
    /// Error indicating a network error or connection failure occurred.
    NetworkError(String),
}

impl std::fmt::Display for UserAgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoUserAgentsAvailable => write!(f, "No user agents are available"),
            Self::HttpError(status) => {
                write!(f, "HTTP error while fetching user agents: status {status}")
            }
            Self::NetworkError(msg) => write!(f, "Network error while fetching user agents: {msg}"),
        }
    }
}

impl std::error::Error for UserAgentError {}

/// Picks a random user agent from a provided slice of user agent strings using a checked RNG.
///
/// This function verifies that the slice is not empty before sampling the RNG, preventing
/// any `low >= high` panic when `user_agents` is empty. Safe bounds-checked indexing via
/// `.get()` is used rather than direct slice indexing.
///
/// # Arguments
///
/// * `user_agents` - A slice of user agent string slices.
///
/// # Errors
///
/// Returns [`UserAgentError::NoUserAgentsAvailable`] if `user_agents` is empty or if indexing fails.
pub fn pick_random_user_agent<'a>(user_agents: &[&'a str]) -> Result<&'a str, UserAgentError> {
    let len = user_agents.len();
    if len == 0 {
        return Err(UserAgentError::NoUserAgentsAvailable);
    }
    let idx = fastrand::usize(..len);
    user_agents
        .get(idx)
        .copied()
        .ok_or(UserAgentError::NoUserAgentsAvailable)
}

/// Picks a random user agent from a provided slice of user agent strings, falling back to
/// [`FALLBACK_USER_AGENT`] if the slice is empty or no user agent could be picked.
///
/// # Arguments
///
/// * `user_agents` - A slice of user agent string slices.
///
/// # Returns
///
/// A user agent string slice.
pub fn pick_random_user_agent_or_fallback<'a>(user_agents: &[&'a str]) -> &'a str {
    pick_random_user_agent(user_agents).unwrap_or(FALLBACK_USER_AGENT)
}

/// A collection of user agent strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAgents {
    /// The stored user agent strings.
    agents: Vec<String>,
}

impl UserAgents {
    /// Creates a new `UserAgents` collection with the given list of user agents.
    ///
    /// # Arguments
    ///
    /// * `agents` - A vector of user agent strings.
    ///
    /// # Returns
    ///
    /// A new `UserAgents` instance.
    pub fn new(agents: Vec<String>) -> Self {
        Self { agents }
    }

    /// Selects a random user agent from the collection using a checked RNG.
    ///
    /// # Errors
    ///
    /// Returns [`UserAgentError::NoUserAgentsAvailable`] if the collection is empty.
    pub fn random(&self) -> Result<&str, UserAgentError> {
        let len = self.agents.len();
        if len == 0 {
            return Err(UserAgentError::NoUserAgentsAvailable);
        }
        let idx = fastrand::usize(..len);
        self.agents
            .get(idx)
            .map(String::as_str)
            .ok_or(UserAgentError::NoUserAgentsAvailable)
    }

    /// Selects a random user agent, or returns [`FALLBACK_USER_AGENT`] if empty.
    ///
    /// # Returns
    ///
    /// A user agent string slice.
    pub fn random_or_fallback(&self) -> &str {
        self.random().unwrap_or(FALLBACK_USER_AGENT)
    }

    /// Returns the number of user agents in the collection.
    ///
    /// # Returns
    ///
    /// The count of user agent strings.
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Returns `true` if the collection contains no user agents.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

/// A trait defining a user agent provider.
pub trait UserAgentProvider {
    /// Fetches or retrieves a list of user agent strings.
    ///
    /// # Errors
    ///
    /// Returns [`UserAgentError`] if user agents cannot be retrieved or provided.
    fn get_user_agents(&self) -> Result<Vec<String>, UserAgentError>;
}

/// A static user agent provider that supplies the built-in [`DEFAULT_USER_AGENTS`].
#[derive(Debug, Default, Clone, Copy)]
pub struct StaticUserAgentProvider;

impl UserAgentProvider for StaticUserAgentProvider {
    fn get_user_agents(&self) -> Result<Vec<String>, UserAgentError> {
        if DEFAULT_USER_AGENTS.is_empty() {
            Err(UserAgentError::NoUserAgentsAvailable)
        } else {
            Ok(DEFAULT_USER_AGENTS
                .iter()
                .map(|&ua| ua.to_string())
                .collect())
        }
    }
}

/// Fetches user agents from an HTTP endpoint using a supplied client with timeouts and status checks.
///
/// Unlike the vulnerable `fake-useragent` implementation, this function:
/// 1. Enforces an HTTP request timeout to ensure it never hangs.
/// 2. Checks HTTP status codes: a 503 Service Unavailable (or any non-success status) yields an
///    explicit [`UserAgentError::HttpError`] instead of parsing invalid HTML.
/// 3. Safely handles connection refused and network errors without panicking.
/// 4. Returns [`UserAgentError::NoUserAgentsAvailable`] if the parsed list is empty (`[]`).
///
/// # Arguments
///
/// * `client` - The `reqwest::Client` to use for making the request.
/// * `url` - The URL of the user agent provider endpoint.
///
/// # Errors
///
/// Returns [`UserAgentError`] if the request fails, status is non-2xx, or parsed list is empty.
pub async fn fetch_user_agents(
    client: &reqwest::Client,
    url: &str,
) -> Result<UserAgents, UserAgentError> {
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| UserAgentError::NetworkError(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(UserAgentError::HttpError(status.as_u16()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| UserAgentError::NetworkError(e.to_string()))?;

    let agents: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    if agents.is_empty() {
        return Err(UserAgentError::NoUserAgentsAvailable);
    }

    Ok(UserAgents::new(agents))
}

/// A function to generate random user agent to improve privacy of the user.
///
/// Picks a random user agent from the static [`DEFAULT_USER_AGENTS`] list using a checked RNG.
/// No runtime network requests are performed.
///
/// # Arguments
///
/// * `_threads` - Unused parameter maintained for backwards compatibility.
///
/// # Returns
///
/// A randomly generated static user agent string.
///
/// # Errors
///
/// Returns an error if no user agent could be selected and fallback is unavailable.
pub async fn random_user_agent(_threads: u8) -> Result<&'static str, Box<dyn std::error::Error>> {
    Ok(pick_random_user_agent(DEFAULT_USER_AGENTS).unwrap_or(FALLBACK_USER_AGENT))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn test_default_user_agents_non_empty() {
        assert!(!DEFAULT_USER_AGENTS.is_empty());
        for &ua in DEFAULT_USER_AGENTS {
            assert!(!ua.is_empty());
            assert!(ua.contains("Mozilla/5.0"));
        }
    }

    #[tokio::test]
    async fn test_random_user_agent_success() {
        let ua = random_user_agent(1).await.unwrap();
        assert!(!ua.is_empty());
        assert!(DEFAULT_USER_AGENTS.contains(&ua));
    }

    #[test]
    fn test_pick_random_user_agent_empty_slice_never_panics() {
        let empty: &[&str] = &[];
        let result = pick_random_user_agent(empty);
        assert_eq!(result, Err(UserAgentError::NoUserAgentsAvailable));
    }

    #[test]
    fn test_pick_random_user_agent_or_fallback_empty_slice() {
        let empty: &[&str] = &[];
        let ua = pick_random_user_agent_or_fallback(empty);
        assert_eq!(ua, FALLBACK_USER_AGENT);
    }

    #[test]
    fn test_user_agents_empty_returns_err_never_panics() {
        let ua_collection = UserAgents::new(Vec::new());
        assert!(ua_collection.is_empty());
        assert_eq!(ua_collection.len(), 0);

        let res = ua_collection.random();
        assert_eq!(res, Err(UserAgentError::NoUserAgentsAvailable));

        let fallback = ua_collection.random_or_fallback();
        assert_eq!(fallback, FALLBACK_USER_AGENT);
    }

    #[test]
    fn test_user_agents_with_elements() {
        let elements = vec!["UserAgent1".to_string(), "UserAgent2".to_string()];
        let ua_collection = UserAgents::new(elements);
        assert_eq!(ua_collection.len(), 2);
        assert!(!ua_collection.is_empty());

        let res = ua_collection.random();
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val == "UserAgent1" || val == "UserAgent2");
    }

    #[test]
    fn test_static_user_agent_provider() {
        let provider = StaticUserAgentProvider;
        let uas = provider.get_user_agents().unwrap();
        assert_eq!(uas.len(), DEFAULT_USER_AGENTS.len());
    }

    /// Regression test: UA provider returning `[]` must never panic or hang.
    #[test]
    fn test_regression_ua_provider_returning_empty_list() {
        struct EmptyProvider;
        impl UserAgentProvider for EmptyProvider {
            fn get_user_agents(&self) -> Result<Vec<String>, UserAgentError> {
                Ok(Vec::new())
            }
        }

        let provider = EmptyProvider;
        let uas = provider.get_user_agents().unwrap();
        assert!(uas.is_empty());

        let collection = UserAgents::new(uas);
        // Must return normal error and never panic
        let err = collection.random();
        assert_eq!(err, Err(UserAgentError::NoUserAgentsAvailable));

        // Fallback must be safe and never panic
        let safe_ua = collection.random_or_fallback();
        assert_eq!(safe_ua, FALLBACK_USER_AGENT);
    }

    /// Regression test: UA provider returning HTTP 503 must never panic or hang.
    #[tokio::test]
    async fn test_regression_ua_provider_http_503_never_panics_or_hangs() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn a mock server returning HTTP 503
        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let response = "HTTP/1.1 503 Service Temporarily Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
            }
        });

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/");

        let result = fetch_user_agents(&client, &url).await;
        assert_eq!(result, Err(UserAgentError::HttpError(503)));

        // Graceful fallback handling
        let ua = result
            .map(|uas| uas.random_or_fallback().to_string())
            .unwrap_or_else(|_| FALLBACK_USER_AGENT.to_string());
        assert_eq!(ua, FALLBACK_USER_AGENT);
    }

    /// Regression test: UA provider connection-refused must never panic or hang.
    #[tokio::test]
    async fn test_regression_ua_provider_connection_refused_never_panics_or_hangs() {
        // Find an unused port and drop listener so the port is closed (connection refused)
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/");

        let result = fetch_user_agents(&client, &url).await;
        assert!(matches!(result, Err(UserAgentError::NetworkError(_))));

        // Graceful fallback handling
        let ua = result
            .map(|uas| uas.random_or_fallback().to_string())
            .unwrap_or_else(|_| FALLBACK_USER_AGENT.to_string());
        assert_eq!(ua, FALLBACK_USER_AGENT);
    }
}
