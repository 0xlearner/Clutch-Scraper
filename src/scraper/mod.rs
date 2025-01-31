mod content;
mod page;

pub use content::ContentScraper;
pub use page::PageScraper;

use scraper::Html;

pub struct Scraper {
    document: Html,
}

impl Scraper {
    pub fn new(html: &str) -> Self {
        Self {
            document: Html::parse_document(html),
        }
    }

    pub fn page(&self) -> PageScraper {
        PageScraper::new(&self.document)
    }

    pub fn content(&self) -> ContentScraper {
        ContentScraper::new(&self.document)
    }
}
