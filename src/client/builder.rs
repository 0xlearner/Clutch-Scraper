use super::Client;
use crate::error::{ClientError, Result};
use http::{
    header::{HeaderMap, HeaderName},
    HeaderValue,
};
use rquest::{Client as RquestClient, Impersonate, Proxy};
use std::str::FromStr;
use url::Url;

#[derive(Default)]
pub struct ClientBuilder {
    base_url: Option<String>,
    proxy: Option<String>,
    chrome_impersonation: bool,
    headers: HeaderMap,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            ..Default::default()
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    pub fn chrome_impersonation(mut self, enabled: bool) -> Self {
        self.chrome_impersonation = enabled;
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let header_name = HeaderName::from_str(key.as_ref())
            .map_err(|e| ClientError::BuildError(format!("Invalid header name: {}", e)))?;

        let header_value = HeaderValue::from_str(value.as_ref())
            .map_err(|e| ClientError::BuildError(format!("Invalid header value: {}", e)))?;

        self.headers.insert(header_name, header_value);
        Ok(self)
    }

    pub fn build(self) -> Result<Client> {
        let base_url = self
            .base_url
            .ok_or_else(|| ClientError::BuildError("Base URL is required".to_string()))?;

        // Validate base URL
        Url::parse(&base_url)
            .map_err(|e| ClientError::InvalidUrl(format!("Invalid base URL: {}", e)))?;

        let mut client_builder = RquestClient::builder();

        if let Some(proxy_url) = self.proxy {
            client_builder = client_builder.proxy(Proxy::all(&proxy_url).map_err(|e| {
                ClientError::BuildError(format!("Failed to configure proxy: {}", e))
            })?);
        }

        if self.chrome_impersonation {
            client_builder = client_builder.impersonate(Impersonate::Chrome131);
        }

        let mut inner = client_builder
            .build()
            .map_err(|e| ClientError::BuildError(format!("Failed to build client: {}", e)))?;

        // Set the headers on the client
        *inner.as_mut().headers() = self.headers;

        Ok(Client { inner, base_url })
    }
}
