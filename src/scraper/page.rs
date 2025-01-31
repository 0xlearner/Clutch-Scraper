use crate::error::{Result, ScraperError};
pub use crate::log_info;
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct PageInfo {
    pub current_page: usize,
    pub next_url: Option<String>,
    pub total_pages: Option<usize>,
}

pub struct PageScraper<'a> {
    document: &'a Html,
    base_url: String,
}

impl<'a> PageScraper<'a> {
    pub(crate) fn new(document: &'a Html) -> Self {
        Self {
            document,
            base_url: "https://clutch.co".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn analyze(&self) -> Result<PageInfo> {
        let current_page = self.get_current_page()?;
        let next_url = self.get_next_page_url()?;
        let total_pages = self.get_total_pages();

        // Log pagination information
        log_info!(
            "[scraper] Page Analysis - Current: {}, Total: {}, Next: {}",
            current_page,
            total_pages.unwrap_or(0),
            next_url.as_deref().unwrap_or("None")
        );

        Ok(PageInfo {
            current_page,
            next_url,
            total_pages,
        })
    }

    fn get_current_page(&self) -> Result<usize> {
        let selector = Selector::parse(".sg-pagination-v2-page-active")
            .map_err(|e| ScraperError::SelectorError(e.to_string()))?;

        if let Some(element) = self.document.select(&selector).next() {
            element
                .text()
                .next()
                .and_then(|t| t.trim().parse().ok())
                .ok_or_else(|| {
                    ScraperError::ParseError("Could not parse page number".into()).into()
                })
        } else {
            Ok(1) // Default to page 1 if no pagination found
        }
    }

    fn get_next_page_url(&self) -> Result<Option<String>> {
        let selector = Selector::parse(".sg-pagination-v2-next")
            .map_err(|e| ScraperError::SelectorError(e.to_string()))?;

        if let Some(next_element) = self.document.select(&selector).next() {
            if next_element
                .value()
                .classes()
                .any(|c| c == "sg-pagination-v2-disabled")
            {
                return Ok(None);
            }

            let current_page = self.get_current_page()?;
            let next_url = format!("/developers/rust?page={}", current_page);
            Ok(Some(format!("{}{}", self.base_url, next_url)))
        } else {
            Ok(None)
        }
    }

    fn get_total_pages(&self) -> Option<usize> {
        let selector = Selector::parse(".sg-pagination-v2-page").ok()?;
        let max_page = self
            .document
            .select(&selector)
            .map(|el| {
                el.text()
                    .next()
                    .and_then(|t| t.trim().parse::<usize>().ok())
            })
            .flatten()
            .max();

        if let Some(total) = max_page {
            log_info!("[scraper] Found total pages: {}", total);
        }

        max_page
    }
}
