pub mod crawl;
pub mod document;
pub mod export;
pub mod html_to_md;
pub mod link_extractor;
pub mod url_manager;
pub mod webshooter;

// Re-export commonly used types
pub use crawl::{CrawlConfig, CrawlResult};
pub use document::Document;
pub use export::Exporter;

#[derive(Debug, Default)]
pub struct Spiderman<'a> {
    url: &'a str,
    html: Option<String>,
}

impl<'a> Spiderman<'a> {
    pub fn new(url: &'a str) -> Self {
        Self { url, html: None }
    }

    /// Get the fetched HTML content
    pub fn get_html(&self) -> Option<&String> {
        self.html.as_ref()
    }
}
