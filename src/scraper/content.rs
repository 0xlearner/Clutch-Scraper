pub use crate::{log_error, log_info};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    country: String,
    locality: String,
    region: String,
    street: String,
    postal_code: String,
    telephone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rating {
    average: Option<f32>,
    review_count: Option<i32>,
    best_rating: Option<f32>,
    worst_rating: Option<f32>,
    rating_value: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyData {
    title: String,
    profile_url: String,
    min_project_size: String,
    hourly_rate: String,
    employees: String,
    location: Option<String>,
    services: Vec<String>,
    focus: Vec<String>,
    address: Address,
    rating: Option<Rating>,
}

pub struct ContentScraper<'a> {
    document: &'a Html,
}

impl<'a> ContentScraper<'a> {
    pub(crate) fn new(document: &'a Html) -> Self {
        Self { document }
    }

    pub fn extract_companies_data(&self) -> Vec<CompanyData> {
        let providers_list_selector =
            Selector::parse("ul.providers__list#providers__list").unwrap();

        if let Some(providers_list) = self.document.select(&providers_list_selector).next() {
            let provider_selector = Selector::parse("li.provider-list-item").unwrap();
            let provider_count = providers_list.select(&provider_selector).count();
            log_info!("Found {} provider items in the list", provider_count);

            let companies = providers_list
                .select(&provider_selector)
                .filter_map(|provider| {
                    let result = self.extract_company_data(provider);
                    if result.is_none() {
                        log_error!("Failed to extract data for a company");
                    }
                    result
                })
                .collect::<Vec<_>>();

            log_info!("Successfully extracted {} companies", companies.len());
            companies
        } else {
            log_info!("Could not find the providers list");
            Vec::new()
        }
    }

    fn extract_company_data(&self, provider: ElementRef) -> Option<CompanyData> {
        Some(CompanyData {
            title: self.extract_title(&provider)?,
            profile_url: self.extract_profile_url(&provider)?,
            min_project_size: self.extract_min_project_size(&provider)?,
            hourly_rate: self.extract_hourly_rate(&provider)?,
            employees: self.extract_employees(&provider)?,
            location: self.extract_location(&provider),
            services: self.extract_services(&provider),
            focus: self.extract_focus_areas(&provider),
            address: self.extract_address(&provider)?,
            rating: self.extract_rating(&provider),
        })
    }

    fn extract_title(&self, provider: &ElementRef) -> Option<String> {
        let selector = Selector::parse("a.provider__title-link").unwrap();
        provider
            .select(&selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .into()
    }

    fn extract_profile_url(&self, provider: &ElementRef) -> Option<String> {
        let selector = Selector::parse("a.provider__title-link").unwrap();
        provider
            .select(&selector)
            .next()?
            .value()
            .attr("href")?
            .to_string()
            .into()
    }

    fn extract_min_project_size(&self, provider: &ElementRef) -> Option<String> {
        let selector = Selector::parse("div.provider__highlights-item.min-project-size").unwrap();
        provider
            .select(&selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .into()
    }

    fn extract_hourly_rate(&self, provider: &ElementRef) -> Option<String> {
        let selector = Selector::parse("div.provider__highlights-item.hourly-rate").unwrap();
        provider
            .select(&selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .into()
    }

    fn extract_employees(&self, provider: &ElementRef) -> Option<String> {
        let selector = Selector::parse("div.provider__highlights-item.employees-count").unwrap();
        provider
            .select(&selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .into()
    }

    fn extract_location(&self, provider: &ElementRef) -> Option<String> {
        let selector = Selector::parse("span.locality").unwrap();
        provider
            .select(&selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .into()
    }

    fn extract_services(&self, provider: &ElementRef) -> Vec<String> {
        let selector =
            Selector::parse(".provider__services--provided .provider__services-chart-item")
                .unwrap();
        provider
            .select(&selector)
            .filter_map(|el| {
                el.value()
                    .attr("data-tooltip-content")
                    .map(|s| s.replace("<i>", "").replace("</i>", ""))
            })
            .collect()
    }

    fn extract_focus_areas(&self, provider: &ElementRef) -> Vec<String> {
        let selector =
            Selector::parse(".provider__services--focus-areas .provider__services-chart-item")
                .unwrap();
        provider
            .select(&selector)
            .filter_map(|el| {
                el.value()
                    .attr("data-tooltip-content")
                    .map(|s| s.replace("<i>", "").replace("</i>", ""))
            })
            .collect()
    }

    fn extract_address(&self, provider: &ElementRef) -> Option<Address> {
        Some(Address {
            country: self.extract_meta_content(provider, "meta[itemprop='addressCountry']")?,
            locality: self.extract_meta_content(provider, "meta[itemprop='addressLocality']")?,
            region: self.extract_meta_content(provider, "meta[itemprop='addressRegion']")?,
            street: self.extract_meta_content(provider, "meta[itemprop='streetAddress']")?,
            postal_code: self.extract_meta_content(provider, "meta[itemprop='postalCode']")?,
            telephone: self.extract_meta_content(provider, "meta[itemprop='telephone']")?,
        })
    }

    fn extract_rating(&self, provider: &ElementRef) -> Option<Rating> {
        Some(Rating {
            average: self.extract_rating_number(provider),
            review_count: self
                .extract_meta_content_as_number(provider, "meta[itemprop='reviewCount']"),
            best_rating: self
                .extract_meta_content_as_number(provider, "meta[itemprop='bestRating']"),
            worst_rating: self
                .extract_meta_content_as_number(provider, "meta[itemprop='worstRating']"),
            rating_value: self
                .extract_meta_content_as_number(provider, "meta[itemprop='ratingValue']"),
        })
    }

    fn extract_meta_content(&self, provider: &ElementRef, selector_str: &str) -> Option<String> {
        let selector = Selector::parse(selector_str).unwrap();
        provider
            .select(&selector)
            .next()?
            .value()
            .attr("content")?
            .to_string()
            .into()
    }

    fn extract_meta_content_as_number<T: std::str::FromStr>(
        &self,
        provider: &ElementRef,
        selector_str: &str,
    ) -> Option<T> {
        self.extract_meta_content(provider, selector_str)?
            .parse()
            .ok()
    }

    fn extract_rating_number(&self, provider: &ElementRef) -> Option<f32> {
        let selector = Selector::parse("span.sg-rating__number").unwrap();
        provider
            .select(&selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .parse()
            .ok()
    }
}
