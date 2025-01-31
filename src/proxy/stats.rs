use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ProxyStats {
    pub validation_status: Option<String>,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub status_codes: HashMap<u16, usize>,
    pub successful_urls: Vec<String>,
    pub failed_urls: Vec<(String, String)>, // (url, reason)
}

impl ProxyStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_validation_status(&mut self, status: String) {
        self.validation_status = Some(status);
    }

    pub fn record_success(&mut self, url: String, status_code: u16) {
        self.total_requests += 1;
        self.successful_requests += 1;
        *self.status_codes.entry(status_code).or_default() += 1;
        self.successful_urls.push(url);
    }

    pub fn record_failure(&mut self, url: String, reason: String, status_code: Option<u16>) {
        self.total_requests += 1;
        self.failed_requests += 1;
        if let Some(code) = status_code {
            *self.status_codes.entry(code).or_default() += 1;
        }
        self.failed_urls.push((url, reason));
    }
}
