//! This module provides the public models for handling web requests made by engines to upstream search engines.

use error_stack::{Result, ResultExt};
use std::{fmt::Display, time::Duration};

use crate::config::parser::Config;
use reqwest::{redirect::Policy, Client, ClientBuilder, Proxy, Response};

#[derive(Debug)]
pub enum ClientError {
    /// Raised when a web request is failed.
    RequestError,
    /// Raised when the proxy url is invalid.
    InvalidProxy,
    /// Raised when the proxy is marked as tor enabled but is not routing the requests
    /// through tor exit nodes.
    TorInactive,
    /// Raised when the client builder faces an error
    UnexpectedError,
    /// Raised when response could not be serialized.
    SerializationError,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::RequestError => {
                write!(f, "Request client couldn't complete the request.")
            }
            ClientError::InvalidProxy => write!(f, "The provided proxy url is invalid"),
            ClientError::TorInactive => write!(f, "Proxy is not using tor exit nodes"),
            ClientError::UnexpectedError => write!(f, "Unexpected client error occured"),
            ClientError::SerializationError => write!(f, "Unable to deserialise response"),
        }
    }
}

impl std::error::Error for ClientError {}

#[derive(Clone)]
pub struct HttpClient(Client);

impl HttpClient {
    pub fn new(config: &Config) -> Result<Self, ClientError> {
        let config = config.request_client.clone();

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout.into()))
            .https_only(config.https_only)
            .redirect(Policy::limited(config.max_redirects.into()))
            .gzip(true)
            .brotli(true);

        let client = if !config.use_http2 {
            client.http1_only()
        } else {
            client
        };

        let client = if let Some(p_url) = config.proxy_url {
            let proxy = Proxy::all(p_url).change_context(ClientError::InvalidProxy)?;
            client.proxy(proxy)
        } else {
            client
        };

        Ok(HttpClient(
            client
                .build()
                .change_context(ClientError::UnexpectedError)?,
        ))
    }

    // TODO: Should this be made public?
    async fn fetch(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        timeout: Option<u8>,
    ) -> Result<Response, ClientError> {
        let client = self.0.get(url).headers(header_map);

        let client = if let Some(timeout) = timeout {
            client.timeout(Duration::from_secs(timeout.into()))
        } else {
            client
        };

        client // add spoofed headers to emulate human behavior
            .send()
            .await
            .change_context(ClientError::RequestError)
    }

    pub async fn fetch_html(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        timeout: Option<u8>,
    ) -> Result<String, ClientError> {
        self.fetch(url, header_map, timeout)
            .await?
            .text()
            .await
            .change_context(ClientError::RequestError)
    }

    pub async fn fetch_json(
        &self,
        url: &str,
        header_map: reqwest::header::HeaderMap,
        timeout: Option<u8>,
    ) -> Result<String, ClientError> {
        self.fetch(url, header_map, timeout)
            .await?
            .json()
            .await
            .change_context(ClientError::SerializationError)
    }
}
