//! URL Manager Module
//!
//! This module provides URL queue management and deduplication for the web crawler.
//! It's responsible for organizing which URLs to crawl and preventing duplicate crawls.
//!
//! # Overview
//!
//! The URL Manager is the "brain" of the crawler that:
//! 1. **Manages the crawl queue**: Keeps track of URLs waiting to be crawled
//! 2. **Prevents duplicates**: Ensures each URL is only crawled once
//! 3. **Enforces limits**: Controls maximum number of pages to crawl
//! 4. **Normalizes URLs**: Standardizes URL format for proper deduplication
//! 5. **Domain filtering**: Optionally restricts crawling to specific domains
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         URL Manager                 │
//! ├─────────────────────────────────────┤
//! │                                     │
//! │  to_visit (Queue - VecDeque)       │
//! │  ┌──────────────────────────────┐  │
//! │  │ url1.com → url2.com → url3   │  │
//! │  └──────────────────────────────┘  │
//! │          ↓ pop_front()             │
//! │                                     │
//! │  visited (HashSet)                 │
//! │  ┌──────────────────────────────┐  │
//! │  │ {url1, url2, url3, url4...}  │  │
//! │  └──────────────────────────────┘  │
//! │                                     │
//! │  Config:                           │
//! │  - max_pages: Option<usize>        │
//! │  - allowed_domains: Vec<String>    │
//! │                                     │
//! └─────────────────────────────────────┘
//! ```
//!
//! # How It Works
//!
//! ## Crawl Flow Example
//!
//! ```text
//! Step 1: Initialize with seed URL
//! to_visit: [example.com]
//! visited: {}
//!
//! Step 2: Get next URL to crawl
//! current: example.com
//! to_visit: []
//! visited: {example.com}
//!
//! Step 3: Extract links from example.com
//! found: [example.com/about, example.com/contact, external.com]
//!
//! Step 4: Add new URLs (filtered & deduplicated)
//! to_visit: [example.com/about, example.com/contact]
//! visited: {example.com}
//! (external.com rejected - different domain)
//!
//! Step 5: Continue crawling
//! current: example.com/about
//! to_visit: [example.com/contact]
//! visited: {example.com, example.com/about}
//! ...and so on
//! ```
//!
//! # URL Normalization
//!
//! URLs are normalized before storage to ensure proper deduplication:
//!
//! ```text
//! Input URLs → Normalized Form
//! ────────────────────────────
//! http://example.com/     → http://example.com
//! http://example.com      → http://example.com
//! http://EXAMPLE.COM      → http://example.com
//! http://example.com:80/  → http://example.com
//! ```
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use spiderman::core::url_manager::UrlManager;
//!
//! // Create a URL manager with seed URL
//! let mut manager = UrlManager::new("http://example.com");
//!
//! // Get next URL to crawl
//! if let Some(url) = manager.get_next() {
//!     println!("Crawling: {}", url);
//!     // ... crawl the URL ...
//!
//!     // Add discovered links
//!     manager.add_url("http://example.com/about");
//!     manager.add_url("http://example.com/contact");
//! }
//!
//! // Check if more URLs to crawl
//! if manager.has_next() {
//!     println!("More URLs in queue!");
//! }
//! ```
//!
//! ## With Domain Filtering
//!
//! ```
//! use spiderman::core::url_manager::UrlManager;
//!
//! let mut manager = UrlManager::new("http://example.com");
//! manager.set_allowed_domains(vec!["example.com".to_string()]);
//!
//! // This will be added
//! manager.add_url("http://example.com/page");
//!
//! // This will be rejected (different domain)
//! manager.add_url("http://external.com/page");
//! ```
//!
//! ## With Page Limit
//!
//! ```
//! use spiderman::core::url_manager::UrlManager;
//!
//! let mut manager = UrlManager::new("http://example.com");
//! manager.set_max_pages(10); // Only crawl 10 pages
//!
//! // Crawl until limit reached
//! while let Some(url) = manager.get_next() {
//!     println!("Crawling: {}", url);
//!     // ... crawl logic ...
//! }
//! ```

use std::collections::{HashSet, VecDeque};

/// URL Manager for crawl queue and deduplication
///
/// This struct manages the crawling process by maintaining:
/// - A queue of URLs waiting to be crawled (`to_visit`)
/// - A set of URLs already crawled (`visited`)
/// - Configuration options (max pages, allowed domains)
///
/// # Fields
///
/// * `to_visit` - Queue of URLs waiting to be crawled (FIFO order)
/// * `visited` - Set of URLs that have already been crawled (for deduplication)
/// * `max_pages` - Optional limit on total pages to crawl
/// * `allowed_domains` - Optional list of domains to restrict crawling to
#[derive(Debug, Clone)]
pub struct UrlManager {
    /// Queue of URLs waiting to be crawled
    to_visit: VecDeque<String>,

    /// Set of URLs that have been visited (crawled or queued)
    visited: HashSet<String>,

    /// Maximum number of pages to crawl (None = unlimited)
    max_pages: Option<usize>,

    /// List of allowed domains (None = all domains allowed)
    allowed_domains: Option<Vec<String>>,
}

impl UrlManager {
    /// Creates a new URL Manager with a seed URL
    ///
    /// The seed URL is the starting point for the crawl. It's automatically
    /// added to the queue and will be the first URL returned by `get_next()`.
    ///
    /// # Arguments
    ///
    /// * `seed_url` - The initial URL to start crawling from
    ///
    /// # Returns
    ///
    /// A new `UrlManager` instance with the seed URL in the queue
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let manager = UrlManager::new("http://example.com");
    /// assert!(manager.has_next());
    /// ```
    pub fn new(seed_url: &str) -> Self {
        let mut manager = Self {
            to_visit: VecDeque::new(),
            visited: HashSet::new(),
            max_pages: None,
            allowed_domains: None,
        };

        // Add seed URL to queue
        manager.add_url(seed_url);

        manager
    }

    /// Sets the maximum number of pages to crawl
    ///
    /// Once this limit is reached, `get_next()` will return `None` even if
    /// there are more URLs in the queue.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum number of pages to crawl
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// manager.set_max_pages(100); // Only crawl 100 pages
    /// ```
    pub fn set_max_pages(&mut self, max: usize) {
        self.max_pages = Some(max);
    }

    /// Sets the allowed domains for crawling
    ///
    /// When set, only URLs from these domains will be added to the queue.
    /// This is useful for restricting the crawler to specific sites.
    ///
    /// # Arguments
    ///
    /// * `domains` - List of allowed domain names (without protocol)
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// manager.set_allowed_domains(vec![
    ///     "example.com".to_string(),
    ///     "www.example.com".to_string()
    /// ]);
    /// ```
    pub fn set_allowed_domains(&mut self, domains: Vec<String>) {
        self.allowed_domains = Some(domains);
    }

    /// Adds a URL to the crawl queue
    ///
    /// The URL will be normalized and checked against:
    /// 1. Visited set (no duplicates)
    /// 2. Allowed domains (if configured)
    /// 3. Max pages limit (if configured)
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to add to the queue
    ///
    /// # Returns
    ///
    /// * `true` if the URL was added successfully
    /// * `false` if the URL was rejected (duplicate, wrong domain, or limit reached)
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// assert!(manager.add_url("http://example.com/about"));
    /// assert!(!manager.add_url("http://example.com/about")); // Duplicate
    /// ```
    pub fn add_url(&mut self, url: &str) -> bool {
        // Normalize the URL
        let normalized = normalize_url_for_storage(url);

        // Check if already visited
        if self.visited.contains(&normalized) {
            return false;
        }

        // Check domain restrictions
        if let Some(ref domains) = self.allowed_domains {
            if let Some(domain) = extract_domain(&normalized) {
                if !domains.iter().any(|d| d == &domain) {
                    return false;
                }
            }
        }

        // Check max pages limit
        if let Some(max) = self.max_pages {
            if self.visited.len() >= max {
                return false;
            }
        }

        // Add to queue and mark as visited
        self.to_visit.push_back(normalized.clone());
        self.visited.insert(normalized);

        true
    }

    /// Gets the next URL to crawl from the queue
    ///
    /// This removes and returns the next URL from the front of the queue.
    /// Returns `None` if the queue is empty or the max pages limit is reached.
    ///
    /// # Returns
    ///
    /// * `Some(String)` - The next URL to crawl
    /// * `None` - If no more URLs to crawl or limit reached
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    ///
    /// if let Some(url) = manager.get_next() {
    ///     println!("Crawling: {}", url);
    /// }
    /// ```
    pub fn get_next(&mut self) -> Option<String> {
        // Check max pages limit
        if let Some(max) = self.max_pages {
            // Count how many pages we've already processed
            // (visited - to_visit = processed)
            let processed = self.visited.len() - self.to_visit.len();
            if processed >= max {
                return None;
            }
        }

        self.to_visit.pop_front()
    }

    /// Checks if there are more URLs to crawl
    ///
    /// # Returns
    ///
    /// * `true` if there are URLs in the queue
    /// * `false` if the queue is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// assert!(manager.has_next());
    ///
    /// manager.get_next();
    /// assert!(!manager.has_next());
    /// ```
    pub fn has_next(&self) -> bool {
        !self.to_visit.is_empty()
    }

    /// Checks if a URL has already been visited
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check
    ///
    /// # Returns
    ///
    /// * `true` if the URL has been visited (crawled or queued)
    /// * `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// assert!(manager.is_visited("http://example.com"));
    /// assert!(!manager.is_visited("http://example.com/other"));
    /// ```
    pub fn is_visited(&self, url: &str) -> bool {
        let normalized = normalize_url_for_storage(url);
        self.visited.contains(&normalized)
    }

    /// Returns the number of URLs that have been visited
    ///
    /// This includes URLs that have been crawled and URLs currently in the queue.
    ///
    /// # Returns
    ///
    /// The total count of visited URLs
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// assert_eq!(manager.visited_count(), 1);
    ///
    /// manager.add_url("http://example.com/about");
    /// assert_eq!(manager.visited_count(), 2);
    /// ```
    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }

    /// Returns the number of URLs currently in the queue
    ///
    /// # Returns
    ///
    /// The count of URLs waiting to be crawled
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// assert_eq!(manager.queue_size(), 1);
    ///
    /// manager.get_next();
    /// assert_eq!(manager.queue_size(), 0);
    /// ```
    pub fn queue_size(&self) -> usize {
        self.to_visit.len()
    }

    /// Returns statistics about the crawl progress
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * Total URLs visited (crawled + queued)
    /// * URLs currently in queue
    /// * URLs processed (crawled)
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderman::core::url_manager::UrlManager;
    ///
    /// let mut manager = UrlManager::new("http://example.com");
    /// let (total, queued, processed) = manager.stats();
    /// println!("Total: {}, Queued: {}, Processed: {}", total, queued, processed);
    /// ```
    pub fn stats(&self) -> (usize, usize, usize) {
        let total = self.visited.len();
        let queued = self.to_visit.len();
        let processed = total - queued;
        (total, queued, processed)
    }
}

/// Normalizes a URL for storage and comparison
///
/// This function standardizes URLs to ensure proper deduplication:
/// - Converts to lowercase
/// - Removes trailing slash (except for root path)
/// - Removes default ports (80 for HTTP, 443 for HTTPS)
/// - Removes URL fragments (#section)
///
/// # Arguments
///
/// * `url` - The URL to normalize
///
/// # Returns
///
/// A normalized URL string
///
/// # Examples
///
/// ```
/// use spiderman::core::url_manager::normalize_url_for_storage;
///
/// assert_eq!(
///     normalize_url_for_storage("HTTP://EXAMPLE.COM/"),
///     "http://example.com"
/// );
///
/// assert_eq!(
///     normalize_url_for_storage("http://example.com:80/page"),
///     "http://example.com/page"
/// );
/// ```
pub fn normalize_url_for_storage(url: &str) -> String {
    let mut url = url.trim().to_lowercase();

    // Remove fragment
    if let Some(pos) = url.find('#') {
        url = url[..pos].to_string();
    }

    // Remove default ports
    url = url.replace(":80/", "/");
    url = url.replace(":443/", "/");

    // Handle URLs ending with :80 or :443 (no trailing slash)
    if url.ends_with(":80") {
        url = url[..url.len() - 3].to_string();
    }
    if url.ends_with(":443") {
        url = url[..url.len() - 4].to_string();
    }

    // Remove trailing slash (except for root)
    if url.ends_with('/') && url.len() > 8 {
        // Check if it's not just "http://" or "https://"
        if let Some(protocol_end) = url.find("://") {
            if url[protocol_end + 3..].contains('/') {
                url = url[..url.len() - 1].to_string();
            }
        }
    }

    url
}

/// Extracts the domain name from a URL
///
/// # Arguments
///
/// * `url` - The URL to extract domain from
///
/// # Returns
///
/// `Some(String)` with the domain name, or `None` if parsing fails
///
/// # Examples
///
/// ```
/// use spiderman::core::url_manager::extract_domain;
///
/// assert_eq!(
///     extract_domain("http://example.com/page"),
///     Some("example.com".to_string())
/// );
///
/// assert_eq!(
///     extract_domain("https://www.example.com:8080/page"),
///     Some("www.example.com".to_string())
/// );
/// ```
pub fn extract_domain(url: &str) -> Option<String> {
    // Remove protocol
    let without_protocol = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };

    // Extract domain (before first slash)
    let domain_part = if let Some(pos) = without_protocol.find('/') {
        &without_protocol[..pos]
    } else {
        without_protocol
    };

    // Remove port if present
    let domain = if let Some(pos) = domain_part.find(':') {
        &domain_part[..pos]
    } else {
        domain_part
    };

    if domain.is_empty() {
        None
    } else {
        Some(domain.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UrlManager Basic Tests =====

    #[test]
    fn test_new_url_manager() {
        let manager = UrlManager::new("http://example.com");
        assert!(manager.has_next());
        assert_eq!(manager.visited_count(), 1);
        assert_eq!(manager.queue_size(), 1);
    }

    #[test]
    fn test_get_next() {
        let mut manager = UrlManager::new("http://example.com");

        let url = manager.get_next();
        assert_eq!(url, Some("http://example.com".to_string()));
        assert!(!manager.has_next());
    }

    #[test]
    fn test_add_url() {
        let mut manager = UrlManager::new("http://example.com");

        assert!(manager.add_url("http://example.com/about"));
        assert_eq!(manager.visited_count(), 2);
        assert_eq!(manager.queue_size(), 2);
    }

    #[test]
    fn test_add_duplicate_url() {
        let mut manager = UrlManager::new("http://example.com");

        assert!(manager.add_url("http://example.com/about"));
        assert!(!manager.add_url("http://example.com/about")); // Duplicate

        assert_eq!(manager.visited_count(), 2); // Still only 2
    }

    #[test]
    fn test_is_visited() {
        let mut manager = UrlManager::new("http://example.com");

        assert!(manager.is_visited("http://example.com"));
        assert!(!manager.is_visited("http://example.com/other"));

        manager.add_url("http://example.com/other");
        assert!(manager.is_visited("http://example.com/other"));
    }

    // ===== Queue Operations Tests =====

    #[test]
    fn test_queue_order() {
        let mut manager = UrlManager::new("http://example.com");
        manager.add_url("http://example.com/page1");
        manager.add_url("http://example.com/page2");
        manager.add_url("http://example.com/page3");

        // FIFO order
        assert_eq!(manager.get_next(), Some("http://example.com".to_string()));
        assert_eq!(
            manager.get_next(),
            Some("http://example.com/page1".to_string())
        );
        assert_eq!(
            manager.get_next(),
            Some("http://example.com/page2".to_string())
        );
        assert_eq!(
            manager.get_next(),
            Some("http://example.com/page3".to_string())
        );
        assert_eq!(manager.get_next(), None);
    }

    #[test]
    fn test_has_next() {
        let mut manager = UrlManager::new("http://example.com");
        assert!(manager.has_next());

        manager.get_next();
        assert!(!manager.has_next());

        manager.add_url("http://example.com/page");
        assert!(manager.has_next());
    }

    // ===== Max Pages Limit Tests =====

    #[test]
    fn test_max_pages_limit() {
        let mut manager = UrlManager::new("http://example.com");
        manager.set_max_pages(2);

        manager.add_url("http://example.com/page1");
        manager.add_url("http://example.com/page2");

        // Should get first 2 URLs
        assert!(manager.get_next().is_some());
        assert!(manager.get_next().is_some());

        // Should stop at max_pages
        assert!(manager.get_next().is_none());
    }

    #[test]
    fn test_max_pages_prevents_adding() {
        let mut manager = UrlManager::new("http://example.com");
        manager.set_max_pages(2);

        assert!(manager.add_url("http://example.com/page1"));
        assert!(!manager.add_url("http://example.com/page2")); // Exceeds limit

        assert_eq!(manager.visited_count(), 2); // Only 2 URLs
    }

    // ===== Domain Filtering Tests =====

    #[test]
    fn test_allowed_domains() {
        let mut manager = UrlManager::new("http://example.com");
        manager.set_allowed_domains(vec!["example.com".to_string()]);

        assert!(manager.add_url("http://example.com/about"));
        assert!(!manager.add_url("http://external.com/page"));

        assert_eq!(manager.visited_count(), 2); // Only example.com URLs
    }

    #[test]
    fn test_multiple_allowed_domains() {
        let mut manager = UrlManager::new("http://example.com");
        manager.set_allowed_domains(vec!["example.com".to_string(), "example.org".to_string()]);

        assert!(manager.add_url("http://example.com/page"));
        assert!(manager.add_url("http://example.org/page"));
        assert!(!manager.add_url("http://other.com/page"));
    }

    // ===== URL Normalization Tests =====

    #[test]
    fn test_normalize_url_lowercase() {
        assert_eq!(
            normalize_url_for_storage("HTTP://EXAMPLE.COM"),
            "http://example.com"
        );
    }

    #[test]
    fn test_normalize_url_trailing_slash() {
        assert_eq!(
            normalize_url_for_storage("http://example.com/page/"),
            "http://example.com/page"
        );

        // Root should keep trailing slash concept
        assert_eq!(
            normalize_url_for_storage("http://example.com/"),
            "http://example.com"
        );
    }

    #[test]
    fn test_normalize_url_default_ports() {
        assert_eq!(
            normalize_url_for_storage("http://example.com:80/page"),
            "http://example.com/page"
        );

        assert_eq!(
            normalize_url_for_storage("https://example.com:443/page"),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_normalize_url_fragment() {
        assert_eq!(
            normalize_url_for_storage("http://example.com/page#section"),
            "http://example.com/page"
        );
    }

    #[test]
    fn test_normalize_url_complex() {
        assert_eq!(
            normalize_url_for_storage("HTTP://EXAMPLE.COM:80/Page/#section"),
            "http://example.com/page"
        );
    }

    // ===== Domain Extraction Tests =====

    #[test]
    fn test_extract_domain_basic() {
        assert_eq!(
            extract_domain("http://example.com/page"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_with_port() {
        assert_eq!(
            extract_domain("http://example.com:8080/page"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_with_subdomain() {
        assert_eq!(
            extract_domain("http://www.example.com/page"),
            Some("www.example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_no_path() {
        assert_eq!(
            extract_domain("http://example.com"),
            Some("example.com".to_string())
        );
    }

    // ===== Statistics Tests =====

    #[test]
    fn test_stats() {
        let mut manager = UrlManager::new("http://example.com");
        manager.add_url("http://example.com/page1");
        manager.add_url("http://example.com/page2");

        let (total, queued, processed) = manager.stats();
        assert_eq!(total, 3);
        assert_eq!(queued, 3);
        assert_eq!(processed, 0);

        manager.get_next();
        let (total, queued, processed) = manager.stats();
        assert_eq!(total, 3);
        assert_eq!(queued, 2);
        assert_eq!(processed, 1);
    }

    #[test]
    fn test_visited_count() {
        let mut manager = UrlManager::new("http://example.com");
        assert_eq!(manager.visited_count(), 1);

        manager.add_url("http://example.com/page1");
        assert_eq!(manager.visited_count(), 2);

        manager.add_url("http://example.com/page1"); // Duplicate
        assert_eq!(manager.visited_count(), 2); // Should not increase
    }

    #[test]
    fn test_queue_size() {
        let mut manager = UrlManager::new("http://example.com");
        assert_eq!(manager.queue_size(), 1);

        manager.get_next();
        assert_eq!(manager.queue_size(), 0);

        manager.add_url("http://example.com/page1");
        manager.add_url("http://example.com/page2");
        assert_eq!(manager.queue_size(), 2);
    }
}
