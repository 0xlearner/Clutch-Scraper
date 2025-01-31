mod builder;

use crate::error::{ClientError, Result};
pub use builder::ClientBuilder;
use rquest::Client as RquestClient;
use url::Url;

#[derive(Debug)]
pub struct ClientResponse {
    pub status: u16,
    pub content: String,
}

pub struct Client {
    inner: RquestClient,
    base_url: String,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn get(&self, path: &str) -> Result<ClientResponse> {
        let url = self.build_url(path)?;
        self.request(&url).await
    }

    fn build_url(&self, path: &str) -> Result<String> {
        // Validate the base URL
        let base = Url::parse(&self.base_url)
            .map_err(|e| ClientError::InvalidUrl(format!("Invalid base URL: {}", e)))?;

        // Join the path to the base URL
        let full_url = base
            .join(path)
            .map_err(|e| ClientError::InvalidUrl(format!("Invalid path: {}", e)))?;

        Ok(full_url.to_string())
    }

    async fn request(&self, url: &str) -> Result<ClientResponse> {
        let response = self
            .inner
            .get(url)
            .send()
            .await
            .map_err(|e| ClientError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let is_success = response.status().is_success();
        let content = response.text().await.map_err(|e| {
            ClientError::RequestFailed(format!("Failed to get response text: {}", e))
        })?;

        if !is_success {
            return Err(ClientError::ResponseError {
                status_code: status,
                message: String::new(),
            }
            .into());
        }

        Ok(ClientResponse { status, content })
    }
}
