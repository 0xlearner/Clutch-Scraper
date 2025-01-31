# Clutch Scraper

A robust and scalable web scraper built in Rust that supports custom proxy rotation, browser impersonation, and asynchronous request handling. This project demonstrates advanced techniques for scraping websites while adhering to best practices in Rust development.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Technologies Used](#technologies-used)
- [Proxy Rotation and Impersonation](#proxy-rotation-and-impersonation)
  - [Proxy Rotation](#proxy-rotation)
  - [Browser Impersonation](#browser-impersonation)
- [Learning Journey](#learning-journey)
- [Setup and Usage](#setup-and-usage)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)

## Overview

The Clutch Scraper is designed to scrape data from websites efficiently and reliably. It incorporates advanced features like proxy rotation to avoid IP blocking, browser impersonation to mimic real user behavior, and error handling to ensure smooth operation even in challenging scenarios.

This project serves as a practical example of how to build a production-grade scraper in Rust, leveraging asynchronous programming and modern libraries.

## Features

- **Custom Proxy Rotation**: Automatically rotates proxies to prevent IP bans and improve reliability.
- **Browser Impersonation**: Uses the `rquest` crate to impersonate Chrome browsers, making requests appear more legitimate.
- **Asynchronous Requests**: Handles multiple requests concurrently using Tokio's async runtime.
- **Error Handling and Retries**: Implements robust error handling and retry mechanisms for failed requests.
- **Logging and Reporting**: Provides detailed logs and performance reports for proxies and scraping operations.
- **Modular Design**: Organized into reusable modules for configuration, logging, proxy management, and scraping logic.

## Technologies Used

- **Rust**: The primary programming language, chosen for its performance, safety, and concurrency capabilities.
- **Tokio**: An asynchronous runtime for handling concurrent tasks.
- **rquest**: A fork of `reqwest` used for HTTP requests with browser impersonation support.
- **Tracing**: For structured logging and monitoring.
- **Config Parsing**: TOML-based configuration files for easy customization.
- **Proxy Management**: Custom logic for validating and rotating proxies dynamically.

## Proxy Rotation and Impersonation

### Proxy Rotation

The scraper includes a custom proxy manager that handles the validation and rotation of proxies. Key features include:

- **Validation**: Proxies are validated against multiple test URLs before being marked as "working."
- **Dynamic Rotation**: The scraper selects the least recently used proxy with the lowest failure count to ensure optimal performance.
- **Failure Handling**: Proxies that exceed the maximum number of failures are moved to a "dead" list and excluded from future requests.

### Browser Impersonation

Using the `rquest` crate, the scraper can impersonate browsers like Chrome to bypass anti-bot measures. Key aspects include:

- **Chrome Impersonation**: Mimics Chrome 130+ headers and TLS fingerprints.
- **Custom Headers**: Allows setting custom headers (e.g., User-Agent, Authorization) to further enhance legitimacy.

## Learning Journey

This project has been an incredible learning experience in Rust, particularly in the following areas:

- **Asynchronous Programming**: Leveraging Tokio and async/await to handle concurrent HTTP requests efficiently.
- **Error Handling**: Implementing robust error handling using Rust's Result and Option types.
- **Ownership and Borrowing**: Understanding Rust's ownership model to manage shared state (e.g., `Arc<Mutex<T>>` for thread-safe data structures).
- **Modular Design**: Breaking the codebase into reusable modules for better maintainability.
- **Testing and Debugging**: Writing unit tests and debugging complex asynchronous workflows.
- **Performance Optimization**: Optimizing memory usage and request throughput for large-scale scraping.

Through this project, I've gained a deeper appreciation for Rust's strengths in building high-performance, reliable systems.

## Setup and Usage

### Prerequisites

- Rust 1.70 or higher
- A valid `config.toml` file with proxy and scraping settings
- A `proxy.txt` file with proxies listed in the format 'proxy:port'.
  If you want to use 'http' proxy then you just have to change this "socks5://{}" to "http://{}" in proxy manager.
  Also, you can modify the proxy settings in `config.toml` file.

### Installation

Clone the repository:

```bash
git clone https://github.com/yourusername/clutch-scraper.git
cd clutch-scraper
cargo build --release
cargo run --release```

Configuration
Edit the config.toml file to specify:

Base URL for scraping
Proxy file path
Logging settings
Retry and timeout configurations
