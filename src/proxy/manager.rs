use super::stats::ProxyStats;
use crate::client::Client;
use crate::config::Config;
use crate::error::{ProxyError, Result};
pub use crate::{log_error, log_info, log_warn};
use futures::{stream::FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;

#[derive(Debug, Clone)]
struct ProxyState {
    url: String,
    failures: u32,
    last_used: Instant,
    stats: Arc<Mutex<ProxyStats>>,
}

pub struct ProxyManager {
    working_proxies: Arc<Mutex<HashMap<String, ProxyState>>>,
    dead_proxies: Arc<Mutex<Vec<String>>>,
    all_stats: Arc<Mutex<HashMap<String, Arc<Mutex<ProxyStats>>>>>,
    config: Config,
}

impl ProxyManager {
    pub async fn new<P: AsRef<Path>>(proxy_file: P, config: Config) -> Result<Self> {
        let contents = std::fs::read_to_string(proxy_file)?;
        let proxies: Vec<String> = contents
            .lines()
            .map(|s| format!("socks5://{}", s.trim()))
            .filter(|s| !s.is_empty())
            .collect();

        let manager = Self {
            working_proxies: Arc::new(Mutex::new(HashMap::new())),
            dead_proxies: Arc::new(Mutex::new(Vec::new())),
            all_stats: Arc::new(Mutex::new(HashMap::new())),
            config,
        };

        manager.validate_proxies(proxies).await?;
        Ok(manager)
    }

    fn get_max_retries(&self) -> u32 {
        self.config.proxy.max_retries
    }

    fn get_request_timeout(&self) -> u64 {
        self.config.proxy.request_timeout
    }

    fn get_concurrent_validations(&self) -> usize {
        self.config.proxy.concurrent_validations
    }

    async fn validate_proxies(&self, proxies: Vec<String>) -> Result<()> {
        let mut tasks = FuturesUnordered::new();

        let request_timeout = self.get_request_timeout();
        let concurrent_validations = self.get_concurrent_validations();

        for proxy in proxies {
            let working_proxies = Arc::clone(&self.working_proxies);
            let dead_proxies = Arc::clone(&self.dead_proxies);

            tasks.push(tokio::spawn(async move {
                // Add timeout to validation
                match timeout(
                    Duration::from_secs(request_timeout),
                    Self::validate_single_proxy(&proxy, request_timeout),
                )
                .await
                {
                    Ok(validation_result) => match validation_result {
                        Ok(_) => {
                            let stats = Arc::new(Mutex::new(ProxyStats::new()));
                            stats
                                .lock()
                                .await
                                .set_validation_status("success".to_string());
                            working_proxies.lock().await.insert(
                                proxy.clone(),
                                ProxyState {
                                    url: proxy,
                                    failures: 0,
                                    last_used: Instant::now(),
                                    stats,
                                },
                            );
                            Ok(())
                        }
                        Err(e) => {
                            dead_proxies.lock().await.push(proxy.clone());
                            log_error!("[proxy] Validation failed for {}: {}", proxy, e);
                            Err(e)
                        }
                    },
                    Err(_) => {
                        dead_proxies.lock().await.push(proxy.clone());
                        log_error!(
                            "[proxy] Validation timed out for {} after {} seconds",
                            proxy,
                            request_timeout
                        );
                        Err(ProxyError::TimeoutError(proxy).into())
                    }
                }
            }));

            if tasks.len() >= concurrent_validations {
                while let Some(result) = tasks.next().await {
                    if let Err(e) = result {
                        log_error!("[proxy] Validation task error: {}", e);
                    }
                }
            }
        }

        while let Some(result) = tasks.next().await {
            if let Err(e) = result {
                log_error!("[proxy] Validation task error: {}", e);
            }
        }

        let working_count = self.working_proxies.lock().await.len();
        if working_count == 0 {
            return Err(ProxyError::NoWorkingProxies.into());
        }

        Ok(())
    }

    async fn validate_single_proxy(proxy_url: &str, request_timeout: u64) -> Result<()> {
        let client = Client::builder()
            .base_url("https://api.ipify.org")
            .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")?
            .header("accept", "en-US,en;q=0.7")?
            .proxy(proxy_url.to_string())
            .chrome_impersonation(true)
            .build()?;

        let test_urls = [
            // "https://api.ipify.org",
            // "https://ifconfig.me/ip",
            // "https://api.myip.com",
            "https://clutch.co", // Add actual target site to validation
        ];

        for url in test_urls {
            match timeout(Duration::from_secs(request_timeout), client.get(url)).await {
                Ok(request_result) => match request_result {
                    Ok(resp) => {
                        if resp.status == 200 {
                            log_info!(
                                "[proxy] Successfully validated proxy {} with {}",
                                proxy_url,
                                url
                            );
                            return Ok(());
                        } else {
                            log_error!(
                                "[proxy] Validation failed for {} with {}: status {}",
                                proxy_url,
                                url,
                                resp.status
                            );
                        }
                    }
                    Err(e) => log_error!("[proxy] Test failed for {}: {}", url, e),
                },
                Err(_) => {
                    log_info!(
                        "[proxy] Request timed out for {} after {} seconds",
                        url,
                        request_timeout
                    );
                }
            }
        }

        Err(ProxyError::ValidationFailed(format!(
            "All validation attempts failed for {}",
            proxy_url
        ))
        .into())
    }

    pub async fn get_proxy(&self) -> Result<String> {
        let mut proxies = self.working_proxies.lock().await;
        let max_retries = self.get_max_retries();

        // Clean up failed proxies first
        let mut to_remove = Vec::new();
        for (url, state) in proxies.iter() {
            if state.failures >= max_retries {
                to_remove.push(url.clone());
            }
        }

        // Move failed proxies to dead_proxies
        if !to_remove.is_empty() {
            let mut dead = self.dead_proxies.lock().await;
            for url in &to_remove {
                proxies.remove(url);
                dead.push(url.clone());
                log_warn!("[proxy] Moving failed proxy to dead list: {}", url);
            }
        }

        // Check if we have any working proxies left
        if proxies.is_empty() {
            let dead_proxies = self.dead_proxies.lock().await;
            let failed_proxies: Vec<_> = dead_proxies
                .iter()
                .map(|url| (url.clone(), "Max retries exceeded".to_string()))
                .collect();

            if !failed_proxies.is_empty() {
                return Err(ProxyError::AllProxiesExhausted { failed_proxies }.into());
            } else {
                return Err(ProxyError::NoWorkingProxies.into());
            }
        }

        // Get the least recently used proxy with lowest failure count
        let proxy = proxies
            .iter_mut()
            .min_by(|a, b| {
                let failure_cmp = a.1.failures.cmp(&b.1.failures);
                if failure_cmp == std::cmp::Ordering::Equal {
                    a.1.last_used.cmp(&b.1.last_used)
                } else {
                    failure_cmp
                }
            })
            .map(|(_, state)| {
                state.last_used = Instant::now();
                state.url.clone()
            })
            .ok_or(ProxyError::NoWorkingProxies)?;

        log_info!("[proxy] Selected proxy: {}", proxy);
        Ok(proxy)
    }

    pub async fn mark_proxy_success(
        &self,
        proxy_url: &str,
        url: &str,
        status_code: u16,
    ) -> Result<()> {
        let mut proxies = self.working_proxies.lock().await;
        if let Some(state) = proxies.get_mut(proxy_url) {
            state.failures = 0; // Reset failures on success
            state.last_used = Instant::now();
            let mut stats = state.stats.lock().await;
            stats.record_success(url.to_string(), status_code);
            log_info!("[proxy] Successful request with proxy {}", proxy_url);
        }
        Ok(())
    }

    pub async fn mark_proxy_failure(
        &self,
        proxy_url: &str,
        error: &str,
        status_code: Option<u16>,
        request_url: &str, // Add this parameter
    ) -> Result<()> {
        let mut proxies = self.working_proxies.lock().await;
        let max_retries = self.get_max_retries();
        if let Some(state) = proxies.get_mut(proxy_url) {
            state.failures += 1;
            state.stats.lock().await.record_failure(
                request_url.to_string(), // Use actual URL
                error.to_string(),
                status_code,
            );

            if state.failures >= max_retries {
                let removed_state = proxies.remove(proxy_url).unwrap();
                self.dead_proxies.lock().await.push(proxy_url.to_string());

                // Store stats before removing the proxy
                let mut all_stats = self.all_stats.lock().await;
                all_stats.insert(proxy_url.to_string(), removed_state.stats.clone());

                log_warn!(
                    "[proxy] Moved proxy {} to dead proxies after {} failures",
                    proxy_url,
                    removed_state.failures
                );
            }
        }
        Ok(())
    }

    pub async fn print_report(&self) {
        let working_proxies = self.working_proxies.lock().await;
        let dead_proxies = self.dead_proxies.lock().await;
        let all_stats = self.all_stats.lock().await;

        log_info!("=== Proxy Performance Report ===");
        log_info!("Validation Summary:");
        log_info!("Working Proxies: {}", working_proxies.len());
        log_info!("Failed Proxies: {}", dead_proxies.len());

        log_info!("Detailed Statistics per Proxy:");
        log_info!("-----------------------------");

        // Print working proxies stats
        for (proxy_url, state) in working_proxies.iter() {
            log_info!("Proxy: {} (Active)", proxy_url);
            let stats = state.stats.lock().await;
            print_proxy_stats(&stats).await;
            log_info!("-----------------------------");
        }

        // Print dead proxies stats
        for proxy_url in dead_proxies.iter() {
            if let Some(stats) = all_stats.get(proxy_url) {
                log_info!("Proxy: {} (Dead)", proxy_url);
                let stats = stats.lock().await;
                print_proxy_stats(&stats).await;
                log_info!("-----------------------------");
            }
        }
    }
}

async fn print_proxy_stats(stats: &ProxyStats) {
    log_info!(
        "Validation Status: {}",
        stats.validation_status.as_deref().unwrap_or("unknown")
    );
    log_info!("Total Requests: {}", stats.total_requests);
    log_info!("Successful Requests: {}", stats.successful_requests);
    log_error!("Failed Requests: {}", stats.failed_requests);

    log_info!("Status Code Distribution:");
    for (code, count) in &stats.status_codes {
        println!("  HTTP {}: {} requests", code, count);
    }

    if !stats.successful_urls.is_empty() {
        log_info!("Successful URLs:");
        for url in &stats.successful_urls {
            log_info!("  ✓ {}", url);
        }
    }

    if !stats.failed_urls.is_empty() {
        log_error!("Failed URLs:");
        for (url, reason) in &stats.failed_urls {
            log_error!("  ✗ {} (Reason: {})", url, reason);
        }
    }
}
