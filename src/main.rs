mod client;
mod config;
mod error;
mod logging;
mod proxy;
mod scraper;
mod utils;

use crate::client::Client;
use crate::config::Config;
use crate::error::Result;
use crate::logging::{init_logging, parse_log_level, LoggerConfig};
use crate::proxy::ProxyManager;
use crate::scraper::Scraper;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    log_info!("[main] Starting scraper...");

    // Load configuration
    let config = Config::from_file("config.toml")?;
    // Initialize logging with custom configuration
    let logger_config = LoggerConfig {
        directory: config.logging.directory.clone(),
        file_name: config.logging.filename.clone(),
        rotation: tracing_appender::rolling::Rotation::DAILY,
        level: parse_log_level(&config.logging.level)?,
    };

    init_logging(logger_config)?;

    log_info!("Starting scraper...");
    let base_url = config.base_url.clone();

    // Initialize proxy manager
    log_info!("[main] Initializing proxy manager...");
    let proxy_manager = ProxyManager::new(&config.proxy.file, config.clone()).await?;

    // First phase: Download and save all pages
    log_info!("[main] Starting download phase...");
    let mut current_path = config.start_path.clone();
    let mut page_number = 1;
    let mut retry_count = 0;
    let mut proxy_retry_count = 0;

    'download: loop {
        log_info!(
            "[main] Fetching page {} from: {}{}",
            page_number,
            base_url,
            current_path
        );

        // Get a proxy
        let proxy = match proxy_manager.get_proxy().await {
            Ok(p) => p,
            Err(e) => {
                log_error!("[main] Failed to get proxy: {}", e);
                if retry_count >= config.max_retries {
                    log_info!("[main] Max retries reached, stopping.");
                    break 'download;
                }
                retry_count += 1;
                log_info!(
                    "[main] Waiting {} seconds before retry...",
                    config.retry_delay
                );
                tokio::time::sleep(Duration::from_secs(config.retry_delay)).await;
                continue;
            }
        };

        log_info!(
            "[main] Using proxy: {} (Attempt {}/{})",
            proxy,
            proxy_retry_count + 1,
            config.max_retries
        );

        // Initialize client with proxy
        let client = Client::builder()
            .base_url(&base_url)
            .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")?
            .header("accept", "en-US,en;q=0.7")?
            .proxy(&proxy)
            .chrome_impersonation(true)
            .build()?;

        // Make request
        match client.get(&current_path).await {
            Ok(response) => {
                if response.status == 403 {
                    log_error!("[main] Received 403 from proxy {}", proxy);
                    proxy_manager
                        .mark_proxy_failure(&proxy, "403 Forbidden", Some(403), &current_path)
                        .await?;

                    if proxy_retry_count >= config.max_retries {
                        if retry_count >= config.max_retries {
                            log_warn!("[main] Max retries reached, stopping.");
                            break 'download;
                        }
                        retry_count += 1;
                        proxy_retry_count = 0;
                    } else {
                        proxy_retry_count += 1;
                    }

                    log_info!(
                        "[main] Waiting {} seconds before switching proxy...",
                        config.proxy.switch_delay
                    );
                    tokio::time::sleep(Duration::from_secs(config.proxy.switch_delay)).await;
                    continue;
                }

                proxy_manager
                    .mark_proxy_success(&proxy, &current_path, response.status)
                    .await?;
                proxy_retry_count = 0;
                retry_count = 0;

                log_info!(
                    "[main] Received response: Status: {}, Content Length: {} bytes",
                    response.status,
                    response.content.len()
                );

                // Save the HTML content
                let saved_path = utils::save_html(&response.content, page_number)?;
                log_info!("[main] Saved page {} to {:?}", page_number, saved_path);

                // Check for next page
                let scraper = Scraper::new(&response.content);
                let page_info = scraper.page().with_base_url(&base_url).analyze()?;

                log_info!(
                    "[main] Processing page {}/{} of results",
                    page_info.current_page,
                    page_info.total_pages.unwrap_or(0)
                );

                match page_info.next_url {
                    Some(next_url) => {
                        current_path = next_url.replace(&base_url, "");
                        page_number = page_info.current_page + 1;
                    }
                    None => {
                        log_info!("[main] Reached last page ({})", page_info.current_page);
                        break;
                    }
                }

                // Add a small delay between successful requests
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                log_error!("[main] Request failed with proxy {}: {}", proxy, error_msg);
                proxy_manager
                    .mark_proxy_failure(&proxy, &error_msg, None, &current_path)
                    .await?;

                if proxy_retry_count >= config.max_retries {
                    if retry_count >= config.max_retries {
                        log_info!("[main] Max retries reached, stopping.");
                        break 'download;
                    }
                    retry_count += 1;
                    proxy_retry_count = 0;
                } else {
                    proxy_retry_count += 1;
                }

                log_info!(
                    "[main] Waiting {} seconds before switching proxy...",
                    config.proxy.switch_delay
                );
                tokio::time::sleep(Duration::from_secs(config.proxy.switch_delay)).await;
                continue;
            }
        }
    }

    // Print proxy performance report
    log_info!("\n[main] Download phase completed. Generating proxy report...");
    proxy_manager.print_report().await;

    // Check if we have any files to process
    let html_dir = std::path::Path::new("local_html");
    if !html_dir.exists() || html_dir.read_dir()?.next().is_none() {
        log_error!(
            "[main] No HTML files found in local_html directory. Skipping processing phase."
        );
        return Ok(());
    }

    // Second phase: Process saved files
    log_info!("\n[main] Starting processing phase...");
    let saved_files = utils::read_html_files()?;

    if saved_files.is_empty() {
        log_error!("[main] No HTML files found to process.");
        return Ok(());
    }

    for (path, content) in saved_files {
        log_info!("[main] Processing {:?}", path);

        let scraper = Scraper::new(&content);

        let companies_data = scraper.content().extract_companies_data();

        if companies_data.is_empty() {
            log_error!("[main] No companies found in {:?}", path);
            continue;
        }

        // Process each company in the file
        for (index, company_data) in companies_data.into_iter().enumerate() {
            if let Some(file_name) = path.file_name() {
                let json_path = std::path::Path::new("json_data").join(
                    file_name
                        .to_string_lossy()
                        .replace(".html", &format!("_company_{}.json", index + 1)),
                );

                utils::save_json(&company_data, &json_path)?;
                log_info!("[main] Saved company data to {:?}", json_path);
            }
        }
    }

    log_info!("[main] Processing completed successfully");
    Ok(())
}
