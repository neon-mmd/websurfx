//! This module provides the public models for handling web requests made by engines to upstream search engines.

use std::time::Duration;

use crate::config::parser::Config;
use reqwest::{Client as rClient, ClientBuilder, redirect::Policy, Proxy};

pub struct Client(rClient);

impl Client {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.request_client;

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout.into()))
            .https_only(config.https_only)
            .redirect(Policy::limited(config.max_redirects.into()))
            .gzip(true)
            .brotli(true);
        
        if !config.use_http2{
            let client = client.http1_only();
        }
        
        if let Some(p_url) = config.proxy_url{
            let proxy = Proxy::all(p_url)?;
            let client = client.proxy(proxy);
        }
        
        Ok(Client(client.build()?))
    }
    
}
