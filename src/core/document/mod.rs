//! Document Module
//!
//! This module provides the data structure for representing crawled web pages
//! and utilities for extracting metadata from HTML content.
//!
//! # Overview
//!
//! The Document module is the core data model for the web crawler. It represents
//! a crawled web page with all its associated metadata, content, and links.
//!
//! # Components
//!
//! 1. **Document Struct** - The main data structure representing a crawled page
//! 2. **Metadata Extraction** - Functions to extract title, description, etc. from HTML
//! 3. **Serialization** - JSON serialization/deserialization support
//! 4. **Builder Pattern** - Easy document creation
//!
//! # Document Structure
//!
//! ```text
//! Document
//! ├── url: String              (The page URL)
//! ├── title: String            (Page title from <title> tag)
//! ├── description: Option      (Meta description)
//! ├── content: String          (Markdown content)
//! ├── raw_html: Option         (Original HTML, optional)
//! ├── links: Vec<String>       (Outbound links found)
//! ├── crawled_at: DateTime     (When it was crawled)
//! └── metadata: HashMap        (Additional metadata)
//! ```
//!
//! # Examples
//!
//! ## Creating a Document
//!
//! ```
//! use spiderman::core::document::{Document, extract_metadata};
//!
//! let html = r#"
//!     <html>
//!         <head>
//!             <title>Example Page</title>
//!             <meta name="description" content="This is an example">
//!         </head>
//!         <body>
//!             <h1>Welcome</h1>
//!             <a href="/about">About</a>
//!         </body>
//!     </html>
//! "#;
//!
//! let metadata = extract_metadata(html);
//! let document = Document::new(
//!     "http://example.com",
//!     "# Welcome\n\nThis is the content",
//!     vec!["http://example.com/about".to_string()]
//! )
//! .with_title(metadata.title.unwrap_or_default())
//! .with_description(metadata.description);
//! ```
//!
//! ## Serializing to JSON
//!
//! ```
//! use spiderman::core::document::Document;
//!
//! let doc = Document::new(
//!     "http://example.com",
//!     "# Content",
//!     vec![]
//! );
//!
//! let json = serde_json::to_string(&doc).unwrap();
//! println!("{}", json);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a crawled web page document
///
/// This is the primary data structure for storing crawled web page information.
/// It contains the URL, content, metadata, and timestamps.
///
/// # Fields
///
/// * `url` - The URL of the crawled page
/// * `title` - The page title (extracted from `<title>` tag)
/// * `description` - Optional meta description
/// * `content` - The main content in Markdown format
/// * `raw_html` - Optional original HTML (for storage/debugging)
/// * `links` - List of outbound links found on the page
/// * `crawled_at` - UTC timestamp of when the page was crawled
/// * `metadata` - Additional key-value metadata
///
/// # Examples
///
/// ```
/// use spiderman::core::document::Document;
///
/// let doc = Document::new(
///     "http://example.com",
///     "# Example\n\nThis is the content",
///     vec!["http://example.com/link1".to_string()]
/// );
///
/// println!("Title: {}", doc.title());
/// println!("URL: {}", doc.url());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The URL of the crawled page
    url: String,

    /// Page title (from <title> tag)
    title: String,

    /// Meta description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// Main content in Markdown format
    content: String,

    /// Original HTML (optional, for storage)
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_html: Option<String>,

    /// Outbound links found on the page
    links: Vec<String>,

    /// When the page was crawled (UTC)
    crawled_at: DateTime<Utc>,

    /// Additional metadata (keywords, author, etc.)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    metadata: HashMap<String, String>,
}

impl Document {
    /// Creates a new Document with the given URL, content, and links
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the page
    /// * `content` - The page content in Markdown format
    /// * `links` - List of outbound links found on the page
    ///
    /// # Returns
    ///
    /// A new Document with current timestamp and empty title
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let doc = Document::new(
    ///     "http://example.com",
    ///     "# Page Content",
    ///     vec!["http://example.com/about".to_string()]
    /// );
    /// ```
    pub fn new(url: &str, content: String, links: Vec<String>) -> Self {
        Self {
            url: url.to_string(),
            title: String::new(),
            description: None,
            content,
            raw_html: None,
            links,
            crawled_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Sets the title and returns self (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![])
    ///     .with_title("Page Title");
    /// ```
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    /// Sets the description and returns self (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![])
    ///     .with_description(Some("Page description".to_string()));
    /// ```
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the raw HTML and returns self (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![])
    ///     .with_raw_html("<html>...</html>".to_string());
    /// ```
    pub fn with_raw_html(mut self, html: String) -> Self {
        self.raw_html = Some(html);
        self
    }

    /// Adds a metadata key-value pair and returns self (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![])
    ///     .with_metadata("author", "John Doe")
    ///     .with_metadata("keywords", "rust, crawler");
    /// ```
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the crawled_at timestamp and returns self (builder pattern)
    ///
    /// Useful for testing or when restoring from storage
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    /// use chrono::Utc;
    ///
    /// let timestamp = Utc::now();
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![])
    ///     .with_timestamp(timestamp);
    /// ```
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.crawled_at = timestamp;
        self
    }

    // Getters

    /// Returns the URL of the document
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the title of the document
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the description if available
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the content (Markdown)
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the raw HTML if stored
    pub fn raw_html(&self) -> Option<&str> {
        self.raw_html.as_deref()
    }

    /// Returns the list of links
    pub fn links(&self) -> &[String] {
        &self.links
    }

    /// Returns the crawled timestamp
    pub fn crawled_at(&self) -> DateTime<Utc> {
        self.crawled_at
    }

    /// Returns the metadata map
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Returns a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Returns the number of links in the document
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Returns the content length in bytes
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// Converts the document to JSON string
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![]);
    /// let json = doc.to_json().unwrap();
    /// ```
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts the document to pretty JSON string
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![]);
    /// let json = doc.to_json_pretty().unwrap();
    /// println!("{}", json);
    /// ```
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Creates a Document from a JSON string
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::document::Document;
    ///
    /// let json = r#"{"url":"http://example.com","title":"","content":"test","links":[],"crawled_at":"2024-01-01T00:00:00Z","metadata":{}}"#;
    /// let doc = Document::from_json(json).unwrap();
    /// ```
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Metadata extracted from HTML
///
/// This struct holds metadata extracted from HTML `<head>` tags.
///
/// # Fields
///
/// * `title` - Page title from `<title>` tag
/// * `description` - Meta description
/// * `keywords` - Meta keywords
/// * `author` - Meta author
/// * `other` - Other meta tags as key-value pairs
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    /// Page title from <title> tag
    pub title: Option<String>,

    /// Meta description
    pub description: Option<String>,

    /// Meta keywords
    pub keywords: Option<String>,

    /// Meta author
    pub author: Option<String>,

    /// Other meta tags
    pub other: HashMap<String, String>,
}

/// Extracts metadata from HTML content
///
/// This function parses HTML to extract common metadata from the `<head>` section:
/// - Title from `<title>` tag
/// - Meta description
/// - Meta keywords
/// - Meta author
/// - Other meta tags
///
/// # Arguments
///
/// * `html` - The HTML content to extract metadata from
///
/// # Returns
///
/// A `Metadata` struct with extracted values
///
/// # Examples
///
/// ```
/// use spiderman::core::document::extract_metadata;
///
/// let html = r#"
///     <html>
///         <head>
///             <title>Example Page</title>
///             <meta name="description" content="This is an example">
///             <meta name="keywords" content="example, test">
///         </head>
///     </html>
/// "#;
///
/// let metadata = extract_metadata(html);
/// assert_eq!(metadata.title, Some("Example Page".to_string()));
/// assert_eq!(metadata.description, Some("This is an example".to_string()));
/// ```
pub fn extract_metadata(html: &str) -> Metadata {
    let mut metadata = Metadata::default();

    // Extract title
    metadata.title = extract_title(html);

    // Extract meta tags
    extract_meta_tags(html, &mut metadata);

    metadata
}

/// Extracts the title from HTML
///
/// Finds and extracts content from the `<title>` tag.
///
/// # Arguments
///
/// * `html` - The HTML content
///
/// # Returns
///
/// The title text if found, None otherwise
fn extract_title(html: &str) -> Option<String> {
    let re = regex::Regex::new(r"<title[^>]*>(.*?)</title>").ok()?;
    re.captures(html)
        .and_then(|cap| cap.get(1))
        .map(|m| decode_html_entities(m.as_str().trim()))
}

/// Extracts meta tags from HTML
///
/// Parses `<meta>` tags and populates the metadata struct.
///
/// # Arguments
///
/// * `html` - The HTML content
/// * `metadata` - The metadata struct to populate
fn extract_meta_tags(html: &str, metadata: &mut Metadata) {
    let re = regex::Regex::new(r#"<meta\s+([^>]+)>"#).unwrap();

    for cap in re.captures_iter(html) {
        if let Some(attrs) = cap.get(1) {
            let attrs_str = attrs.as_str();

            // Extract name and content attributes
            let name = extract_attribute(attrs_str, "name");
            let content = extract_attribute(attrs_str, "content");

            if let (Some(n), Some(c)) = (name, content) {
                let name_lower = n.to_lowercase();
                let content_decoded = decode_html_entities(&c);

                match name_lower.as_str() {
                    "description" => metadata.description = Some(content_decoded),
                    "keywords" => metadata.keywords = Some(content_decoded),
                    "author" => metadata.author = Some(content_decoded),
                    _ => {
                        metadata.other.insert(n, content_decoded);
                    }
                }
            }
        }
    }
}

/// Extracts an attribute value from an HTML tag's attributes string
///
/// # Arguments
///
/// * `attrs` - The attributes string
/// * `attr_name` - The attribute name to extract
///
/// # Returns
///
/// The attribute value if found
fn extract_attribute(attrs: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}=["']([^"']*)["']"#, attr_name);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(attrs)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Decodes common HTML entities in text
///
/// Handles common entities like &amp;, &lt;, &gt;, &quot;, &#39;
///
/// # Arguments
///
/// * `text` - The text with HTML entities
///
/// # Returns
///
/// Text with entities decoded
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Document Creation Tests =====

    #[test]
    fn test_document_new() {
        let doc = Document::new(
            "http://example.com",
            "# Content".to_string(),
            vec!["http://example.com/link1".to_string()],
        );

        assert_eq!(doc.url(), "http://example.com");
        assert_eq!(doc.content(), "# Content");
        assert_eq!(doc.links().len(), 1);
        assert_eq!(doc.title(), "");
    }

    #[test]
    fn test_document_builder_pattern() {
        let doc = Document::new("http://example.com", "content".to_string(), vec![])
            .with_title("Test Title".to_string())
            .with_description(Some("Test Description".to_string()))
            .with_metadata("author", "John Doe");

        assert_eq!(doc.title(), "Test Title");
        assert_eq!(doc.description(), Some("Test Description"));
        assert_eq!(doc.get_metadata("author"), Some("John Doe"));
    }

    #[test]
    fn test_document_with_raw_html() {
        let html = "<html><body>Test</body></html>";
        let doc = Document::new("http://example.com", "content".to_string(), vec![])
            .with_raw_html(html.to_string());

        assert_eq!(doc.raw_html(), Some(html));
    }

    // ===== Getter Tests =====

    #[test]
    fn test_document_getters() {
        let links = vec!["http://example.com/1".to_string()];
        let doc = Document::new("http://example.com", "# Test".to_string(), links.clone())
            .with_title("Title".to_string());

        assert_eq!(doc.url(), "http://example.com");
        assert_eq!(doc.title(), "Title");
        assert_eq!(doc.content(), "# Test");
        assert_eq!(doc.links(), &links[..]);
        assert_eq!(doc.link_count(), 1);
        assert_eq!(doc.content_length(), 6);
    }

    #[test]
    fn test_document_metadata() {
        let doc = Document::new("http://example.com", "content".to_string(), vec![])
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(doc.get_metadata("key1"), Some("value1"));
        assert_eq!(doc.get_metadata("key2"), Some("value2"));
        assert_eq!(doc.get_metadata("key3"), None);
        assert_eq!(doc.metadata().len(), 2);
    }

    // ===== Serialization Tests =====

    #[test]
    fn test_document_to_json() {
        let doc = Document::new("http://example.com", "content".to_string(), vec![]);
        let json = doc.to_json();

        assert!(json.is_ok());
        assert!(json.unwrap().contains("http://example.com"));
    }

    #[test]
    fn test_document_from_json() {
        let doc = Document::new("http://example.com", "test content".to_string(), vec![])
            .with_title("Test".to_string());

        let json = doc.to_json().unwrap();
        let restored = Document::from_json(&json).unwrap();

        assert_eq!(restored.url(), "http://example.com");
        assert_eq!(restored.title(), "Test");
        assert_eq!(restored.content(), "test content");
    }

    #[test]
    fn test_document_json_pretty() {
        let doc = Document::new("http://example.com", "content".to_string(), vec![]);
        let json = doc.to_json_pretty().unwrap();

        assert!(json.contains('\n')); // Pretty print has newlines
        assert!(json.contains("http://example.com"));
    }

    // ===== Metadata Extraction Tests =====

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Title</title></head></html>";
        let title = extract_title(html);

        assert_eq!(title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_extract_title_with_whitespace() {
        let html = "<html><head><title>  Test Title  </title></head></html>";
        let title = extract_title(html);

        assert_eq!(title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_extract_title_not_found() {
        let html = "<html><head></head></html>";
        let title = extract_title(html);

        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_metadata_complete() {
        let html = r#"
            <html>
                <head>
                    <title>Page Title</title>
                    <meta name="description" content="Page description">
                    <meta name="keywords" content="keyword1, keyword2">
                    <meta name="author" content="John Doe">
                </head>
            </html>
        "#;

        let metadata = extract_metadata(html);

        assert_eq!(metadata.title, Some("Page Title".to_string()));
        assert_eq!(metadata.description, Some("Page description".to_string()));
        assert_eq!(metadata.keywords, Some("keyword1, keyword2".to_string()));
        assert_eq!(metadata.author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_extract_metadata_partial() {
        let html = r#"
            <html>
                <head>
                    <title>Page Title</title>
                    <meta name="description" content="Description only">
                </head>
            </html>
        "#;

        let metadata = extract_metadata(html);

        assert_eq!(metadata.title, Some("Page Title".to_string()));
        assert_eq!(metadata.description, Some("Description only".to_string()));
        assert_eq!(metadata.keywords, None);
        assert_eq!(metadata.author, None);
    }

    #[test]
    fn test_extract_metadata_custom() {
        let html = r#"<meta name="custom-tag" content="custom value">"#;

        let metadata = extract_metadata(html);

        assert_eq!(
            metadata.other.get("custom-tag"),
            Some(&"custom value".to_string())
        );
    }

    #[test]
    fn test_decode_html_entities() {
        let text = "Test &amp; Example &lt;tag&gt; &quot;quoted&quot; &#39;apostrophe&#39;";
        let decoded = decode_html_entities(text);

        assert_eq!(decoded, "Test & Example <tag> \"quoted\" 'apostrophe'");
    }

    #[test]
    fn test_extract_attribute() {
        let attrs = r#"name="description" content="test content""#;

        let name = extract_attribute(attrs, "name");
        let content = extract_attribute(attrs, "content");

        assert_eq!(name, Some("description".to_string()));
        assert_eq!(content, Some("test content".to_string()));
    }

    #[test]
    fn test_extract_attribute_not_found() {
        let attrs = r#"name="description""#;
        let missing = extract_attribute(attrs, "content");

        assert_eq!(missing, None);
    }

    // ===== Edge Cases Tests =====

    #[test]
    fn test_document_empty_links() {
        let doc = Document::new("http://example.com", "content".to_string(), vec![]);

        assert_eq!(doc.link_count(), 0);
        assert_eq!(doc.links().len(), 0);
    }

    #[test]
    fn test_document_many_links() {
        let links: Vec<String> = (0..100)
            .map(|i| format!("http://example.com/{}", i))
            .collect();

        let doc = Document::new("http://example.com", "content".to_string(), links);

        assert_eq!(doc.link_count(), 100);
    }

    #[test]
    fn test_metadata_empty_html() {
        let metadata = extract_metadata("");

        assert_eq!(metadata.title, None);
        assert_eq!(metadata.description, None);
    }

    #[test]
    fn test_document_timestamp() {
        let timestamp = Utc::now();
        let doc = Document::new("http://example.com", "content".to_string(), vec![])
            .with_timestamp(timestamp);

        assert_eq!(doc.crawled_at(), timestamp);
    }
}
