use super::document::{extract_metadata, Document};
use super::export::Exporter;
use super::html_to_md::parser;
use super::link_extractor::extract_links;
use super::url_manager::UrlManager;
use super::Spiderman;

/// Configuration for the web crawler
///
/// This struct holds all configuration options for customizing crawler behavior.
///
/// # Examples
///
/// ```
/// use spiderman::core::CrawlConfig;
///
/// let config = CrawlConfig::default()
///     .with_max_pages(100)
///     .with_allowed_domains(vec!["example.com".to_string()])
///     .with_output_dir("crawled_data");
/// ```
#[derive(Debug, Clone)]
pub struct CrawlConfig {
    /// Maximum number of pages to crawl (None = unlimited)
    pub max_pages: Option<usize>,

    /// List of allowed domains (None = all domains)
    pub allowed_domains: Option<Vec<String>>,

    /// Output directory for exported documents
    pub output_dir: String,

    /// Output filename for JSONL export
    pub output_file: String,

    /// Whether to store raw HTML in documents
    pub store_raw_html: bool,

    /// Whether to print progress during crawl
    pub verbose: bool,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            max_pages: Some(50),
            allowed_domains: None,
            output_dir: "output".to_string(),
            output_file: "crawl.jsonl".to_string(),
            store_raw_html: false,
            verbose: true,
        }
    }
}

impl CrawlConfig {
    /// Creates a new CrawlConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of pages to crawl
    pub fn with_max_pages(mut self, max: usize) -> Self {
        self.max_pages = Some(max);
        self
    }

    /// Sets the allowed domains
    pub fn with_allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = Some(domains);
        self
    }

    /// Sets the output directory
    pub fn with_output_dir(mut self, dir: &str) -> Self {
        self.output_dir = dir.to_string();
        self
    }

    /// Sets the output filename
    pub fn with_output_file(mut self, file: &str) -> Self {
        self.output_file = file.to_string();
        self
    }

    /// Enables storing raw HTML in documents
    pub fn with_raw_html(mut self, store: bool) -> Self {
        self.store_raw_html = store;
        self
    }

    /// Enables or disables verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// Result of a crawl operation
///
/// Contains statistics and the list of crawled documents.
#[derive(Debug, Clone)]
pub struct CrawlResult {
    /// Number of pages successfully crawled
    pub pages_crawled: usize,

    /// Number of pages that failed
    pub pages_failed: usize,

    /// Total number of unique URLs discovered
    pub urls_discovered: usize,

    /// List of all crawled documents
    pub documents: Vec<Document>,
}

impl<'a> Spiderman<'a> {
    /// Crawls the website starting from the seed URL
    ///
    /// This is the main entry point for the crawler. It orchestrates all components
    /// to fetch, parse, and export web pages.
    ///
    /// # How It Works
    ///
    /// 1. Initialize URL Manager with seed URL
    /// 2. Loop while there are URLs to crawl:
    ///    a. Get next URL from queue
    ///    b. Fetch HTML content
    ///    c. Extract links and add to queue
    ///    d. Convert HTML to Markdown
    ///    e. Extract metadata
    ///    f. Create Document
    ///    g. Export Document
    /// 3. Return crawl results
    ///
    /// # Arguments
    ///
    /// * `config` - Crawl configuration options
    ///
    /// # Returns
    ///
    /// * `Ok(CrawlResult)` - Successful crawl with statistics
    /// * `Err` - If a critical error occurs
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderman::core::{Spiderman, CrawlConfig};
    ///
    /// async_std::task::block_on(async {
    ///     let mut spider = Spiderman::new("example.com");
    ///     let config = CrawlConfig::default().with_max_pages(10);
    ///     let result = spider.crawl(config).await.unwrap();
    ///
    ///     println!("Crawled {} pages", result.pages_crawled);
    /// });
    /// ```
    pub async fn crawl(
        &mut self,
        config: CrawlConfig,
    ) -> Result<CrawlResult, Box<dyn std::error::Error>> {
        if config.verbose {
            println!("🕷️  Starting Spiderman Web Crawler");
            println!("📍 Seed URL: {}", self.url);
            println!("📁 Output: {}/{}", config.output_dir, config.output_file);
            if let Some(max) = config.max_pages {
                println!("📊 Max pages: {}", max);
            }
            println!();
        }

        // Initialize URL Manager
        let mut manager = UrlManager::new(self.url);

        // Configure URL Manager
        if let Some(max) = config.max_pages {
            manager.set_max_pages(max);
        }
        if let Some(ref domains) = config.allowed_domains {
            manager.set_allowed_domains(domains.clone());
        }

        // Initialize Exporter
        let exporter = Exporter::new(&config.output_dir);

        // Statistics
        let mut pages_crawled = 0;
        let mut pages_failed = 0;
        let mut documents = Vec::new();

        // Main crawl loop
        while let Some(current_url) = manager.get_next() {
            if config.verbose {
                let (total, queued, processed) = manager.stats();
                println!("[{}/{}] Crawling: {}", processed + 1, total, current_url);
            }

            // Fetch HTML
            match self.fetch_url(&current_url).await {
                Ok(html) => {
                    // Extract links and add to queue
                    let links = extract_links(&html, &current_url);
                    let mut added = 0;
                    for link in &links {
                        if manager.add_url(link) {
                            added += 1;
                        }
                    }

                    if config.verbose && added > 0 {
                        println!("  ├─ Found {} links ({} new)", links.len(), added);
                    }

                    // Convert HTML to Markdown
                    let markdown = parser(html.clone());

                    // Extract metadata
                    let metadata = extract_metadata(&html);
                    let title = metadata.title.unwrap_or_else(|| {
                        // Fallback: extract from URL
                        current_url
                            .split('/')
                            .last()
                            .unwrap_or("Untitled")
                            .to_string()
                    });

                    // Create document
                    let mut doc = Document::new(&current_url, markdown, links)
                        .with_title(title)
                        .with_description(metadata.description);

                    // Add metadata
                    if let Some(keywords) = metadata.keywords {
                        doc = doc.with_metadata("keywords", &keywords);
                    }
                    if let Some(author) = metadata.author {
                        doc = doc.with_metadata("author", &author);
                    }

                    // Store raw HTML if configured
                    if config.store_raw_html {
                        doc = doc.with_raw_html(html);
                    }

                    // Export document
                    if let Err(e) = exporter.export_document(&doc, &config.output_file) {
                        eprintln!("  ├─ ⚠️  Export error: {}", e);
                    } else if config.verbose {
                        println!(
                            "  └─ ✓ Exported to {}/{}",
                            config.output_dir, config.output_file
                        );
                    }

                    documents.push(doc);
                    pages_crawled += 1;
                }
                Err(e) => {
                    if config.verbose {
                        eprintln!("  └─ ✗ Error: {}", e);
                    }
                    pages_failed += 1;
                }
            }

            if config.verbose {
                println!();
            }
        }

        // Final statistics
        let (total_urls, _, _) = manager.stats();

        if config.verbose {
            println!("✅ Crawl Complete!");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📈 Statistics:");
            println!("   • Pages crawled: {}", pages_crawled);
            println!("   • Pages failed: {}", pages_failed);
            println!("   • URLs discovered: {}", total_urls);
            println!("   • Output: {}/{}", config.output_dir, config.output_file);
            println!();
        }

        Ok(CrawlResult {
            pages_crawled,
            pages_failed,
            urls_discovered: total_urls,
            documents,
        })
    }

    /// Fetches HTML from a URL
    ///
    /// Internal helper method that directly fetches HTML without modifying self.url.
    async fn fetch_url(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        use async_std::io::{BufReader, ReadExt, WriteExt};
        use async_std::net::TcpStream;

        // Parse URL
        let (host, path) = parse_url(url)?;

        // Connect to host
        let address = format!("{}:80", host);
        let mut stream = TcpStream::connect(&address).await?;

        // Build HTTP request
        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             User-Agent: Spiderman/0.1.0 (Rust Web Crawler)\r\n\
             Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
             Connection: close\r\n\
             \r\n",
            path, host
        );

        // Send request
        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_to_string(&mut response).await?;

        // Extract body
        extract_body(&response)
    }
}

/// Parses URL to extract host and path
fn parse_url(url: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let url = url
        .trim_start_matches("http://")
        .trim_start_matches("https://");

    let parts: Vec<&str> = url.splitn(2, '/').collect();
    let host = parts[0].to_string();
    let path = if parts.len() > 1 {
        format!("/{}", parts[1])
    } else {
        "/".to_string()
    };

    if host.is_empty() {
        return Err("Invalid URL: empty host".into());
    }

    Ok((host, path))
}

/// Extracts body from HTTP response
fn extract_body(response: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(pos) = response.find("\r\n\r\n") {
        Ok(response[pos + 4..].to_string())
    } else if let Some(pos) = response.find("\n\n") {
        Ok(response[pos + 2..].to_string())
    } else {
        Err("Invalid HTTP response: no body separator found".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawl_config_default() {
        let config = CrawlConfig::default();
        assert_eq!(config.max_pages, Some(50));
        assert_eq!(config.output_dir, "output");
        assert_eq!(config.output_file, "crawl.jsonl");
        assert_eq!(config.verbose, true);
    }

    #[test]
    fn test_crawl_config_builder() {
        let config = CrawlConfig::new()
            .with_max_pages(100)
            .with_output_dir("custom_output")
            .with_output_file("results.jsonl")
            .with_verbose(false);

        assert_eq!(config.max_pages, Some(100));
        assert_eq!(config.output_dir, "custom_output");
        assert_eq!(config.output_file, "results.jsonl");
        assert_eq!(config.verbose, false);
    }

    #[test]
    fn test_crawl_config_with_domains() {
        let domains = vec!["example.com".to_string(), "test.com".to_string()];
        let config = CrawlConfig::new().with_allowed_domains(domains.clone());

        assert_eq!(config.allowed_domains, Some(domains));
    }

    #[test]
    fn test_crawl_config_raw_html() {
        let config = CrawlConfig::new().with_raw_html(true);
        assert_eq!(config.store_raw_html, true);
    }
}
