//! This module provides the functionality to generate random user agent string.

use fake_useragent::{Browsers, UserAgents, UserAgentsBuilder};

static USER_AGENTS: once_cell::sync::Lazy<UserAgents> = once_cell::sync::Lazy::new(|| {
    UserAgentsBuilder::new()
        .cache(false)
        .dir("/tmp")
        .thread(1)
        .set_browsers(
            Browsers::new()
                .set_chrome()
                .set_safari()
                .set_edge()
                .set_firefox()
                .set_mozilla(),
        )
        .build()
});

/// A function to generate random user agent to improve privacy of the user.
///
/// # Returns
///
/// A randomly generated user agent string.
pub fn random_user_agent() -> String {
    USER_AGENTS.random().to_string()
}
