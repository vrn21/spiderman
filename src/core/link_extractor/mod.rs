//! Link Extractor Module
//!
//! This module provides functionality to extract and normalize URLs from HTML content.
//! It's a core component of the web crawler that discovers new pages to crawl.
//!
//! # Overview
//!
//! The link extractor performs the following operations:
//! 1. **Extraction**: Finds all `<a href="...">` tags in HTML
//! 2. **Normalization**: Converts relative URLs to absolute URLs
//! 3. **Filtering**: Removes invalid URLs (anchors, javascript:, mailto:, etc.)
//! 4. **Deduplication**: Returns unique URLs only
//!
//! # How It Works
//!
//! ## URL Extraction Process
//!
//! ```text
//! HTML Input:
//! <a href="/about">About</a>
//! <a href="https://example.com/contact">Contact</a>
//! <a href="#section">Jump</a>
//! <a href="javascript:void(0)">Click</a>
//!
//! Step 1: Extract all href attributes
//! → ["/about", "https://example.com/contact", "#section", "javascript:void(0)"]
//!
//! Step 2: Filter out invalid URLs (anchors, javascript, mailto)
//! → ["/about", "https://example.com/contact"]
//!
//! Step 3: Normalize relative URLs to absolute
//! Base URL: http://example.com
//! → ["http://example.com/about", "https://example.com/contact"]
//!
//! Step 4: Deduplicate
//! → ["http://example.com/about", "https://example.com/contact"]
//! ```
//!
//! ## URL Types Handled
//!
//! - **Absolute URLs**: `http://example.com/page` - Used as-is
//! - **Relative URLs**: `/page`, `page.html` - Converted to absolute
//! - **Protocol-relative**: `//example.com/page` - Gets protocol from base URL
//! - **Anchor links**: `#section` - Filtered out (same page navigation)
//! - **Special protocols**: `javascript:`, `mailto:`, `tel:` - Filtered out
//!
//! # Examples
//!
//! ```
//! use spiderman::core::link_extractor::extract_links;
//!
//! let html = r#"
//!     <html>
//!         <a href="/about">About</a>
//!         <a href="https://external.com">External</a>
//!         <a href="#top">Top</a>
//!     </html>
//! "#;
//!
//! let base_url = "http://example.com";
//! let links = extract_links(html, base_url);
//!
//! // Result: ["http://example.com/about", "https://external.com"]
//! ```

use std::collections::HashSet;

/// Extracts all valid links from HTML content and normalizes them to absolute URLs
///
/// This is the main entry point for link extraction. It takes raw HTML and a base URL,
/// then returns a list of unique, normalized, absolute URLs found in the HTML.
///
/// # Arguments
///
/// * `html` - The HTML content to extract links from
/// * `base_url` - The base URL used to resolve relative links
///
/// # Returns
///
/// A `Vec<String>` containing unique, normalized absolute URLs
///
/// # Process Flow
///
/// 1. Parse HTML to find all `<a href="...">` tags using regex
/// 2. Extract the href attribute value from each tag
/// 3. Filter out invalid URLs (anchors, javascript:, mailto:, etc.)
/// 4. Normalize relative URLs to absolute URLs using the base URL
/// 5. Deduplicate URLs using a HashSet
/// 6. Return the final list of unique URLs
///
/// # Examples
///
/// ```
/// use spiderman::core::link_extractor::extract_links;
///
/// let html = r#"<a href="/page1">Page 1</a><a href="/page2">Page 2</a>"#;
/// let links = extract_links(html, "http://example.com");
///
/// assert_eq!(links.len(), 2);
/// assert!(links.contains(&"http://example.com/page1".to_string()));
/// ```
pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let mut unique_links = HashSet::new();

    // Find all <a> tags with href attributes using regex
    // Pattern matches: <a ...href="..." ...> or <a ...href='...' ...>
    let re = regex::Regex::new(r#"<a\s+[^>]*href\s*=\s*["']([^"']+)["'][^>]*>"#).unwrap();

    for cap in re.captures_iter(html) {
        if let Some(href) = cap.get(1) {
            let url = href.as_str();

            // Filter out invalid URLs
            if !is_valid_url(url) {
                continue;
            }

            // Normalize the URL to absolute
            if let Some(absolute_url) = normalize_url(url, base_url) {
                unique_links.insert(absolute_url);
            }
        }
    }

    // Convert HashSet to Vec and return
    unique_links.into_iter().collect()
}

/// Checks if a URL is valid for crawling
///
/// This function filters out URLs that should not be followed by the crawler:
/// - Fragment-only URLs (e.g., `#section`) - These are same-page anchors
/// - JavaScript URLs (e.g., `javascript:void(0)`)
/// - Mailto links (e.g., `mailto:email@example.com`)
/// - Telephone links (e.g., `tel:+1234567890`)
/// - Data URLs (e.g., `data:image/png;base64,...`)
/// - Empty URLs
///
/// # Arguments
///
/// * `url` - The URL string to validate
///
/// # Returns
///
/// `true` if the URL is valid for crawling, `false` otherwise
///
/// # Examples
///
/// ```
/// use spiderman::core::link_extractor::is_valid_url;
///
/// assert!(is_valid_url("http://example.com"));
/// assert!(is_valid_url("/about"));
/// assert!(is_valid_url("../page.html"));
///
/// assert!(!is_valid_url("#section"));
/// assert!(!is_valid_url("javascript:void(0)"));
/// assert!(!is_valid_url("mailto:test@example.com"));
/// ```
pub fn is_valid_url(url: &str) -> bool {
    let url = url.trim();

    // Empty URLs are invalid
    if url.is_empty() {
        return false;
    }

    // Filter out fragment-only URLs (anchors)
    if url.starts_with('#') {
        return false;
    }

    // Filter out javascript: URLs
    if url.starts_with("javascript:") {
        return false;
    }

    // Filter out mailto: URLs
    if url.starts_with("mailto:") {
        return false;
    }

    // Filter out tel: URLs
    if url.starts_with("tel:") {
        return false;
    }

    // Filter out data: URLs
    if url.starts_with("data:") {
        return false;
    }

    true
}

/// Normalizes a URL to an absolute URL using a base URL
///
/// This function handles various URL formats and converts them to absolute URLs:
///
/// # URL Format Handling
///
/// | Input Format | Base URL | Output |
/// |--------------|----------|--------|
/// | `http://example.com/page` | (any) | `http://example.com/page` |
/// | `/about` | `http://example.com` | `http://example.com/about` |
/// | `about` | `http://example.com/blog/` | `http://example.com/blog/about` |
/// | `../page` | `http://example.com/a/b/` | `http://example.com/a/page` |
/// | `//cdn.com/file` | `http://example.com` | `http://cdn.com/file` |
///
/// # Arguments
///
/// * `url` - The URL to normalize (can be relative or absolute)
/// * `base_url` - The base URL to resolve relative URLs against
///
/// # Returns
///
/// `Some(String)` with the normalized absolute URL, or `None` if normalization fails
///
/// # Examples
///
/// ```
/// use spiderman::core::link_extractor::normalize_url;
///
/// let base = "http://example.com/page/";
///
/// assert_eq!(
///     normalize_url("/about", base),
///     Some("http://example.com/about".to_string())
/// );
///
/// assert_eq!(
///     normalize_url("contact.html", base),
///     Some("http://example.com/page/contact.html".to_string())
/// );
/// ```
pub fn normalize_url(url: &str, base_url: &str) -> Option<String> {
    let url = url.trim();
    let base_url = base_url.trim();

    // If URL is already absolute (has protocol), return as-is
    if url.starts_with("http://") || url.starts_with("https://") {
        return Some(clean_url(url));
    }

    // Handle protocol-relative URLs (//example.com/path)
    if url.starts_with("//") {
        // Extract protocol from base_url
        let protocol = if base_url.starts_with("https://") {
            "https:"
        } else {
            "http:"
        };
        return Some(clean_url(&format!("{}{}", protocol, url)));
    }

    // Parse base URL to extract components
    let (base_protocol, base_host, base_path) = parse_base_url(base_url)?;

    // Handle absolute paths (start with /)
    if url.starts_with('/') {
        return Some(clean_url(&format!(
            "{}://{}{}",
            base_protocol, base_host, url
        )));
    }

    // Handle relative paths
    // Remove filename from base path if present
    let base_dir = if base_path.ends_with('/') {
        base_path.to_string()
    } else {
        // Remove last component (filename)
        let parts: Vec<&str> = base_path.rsplitn(2, '/').collect();
        if parts.len() > 1 {
            format!("{}/", parts[1])
        } else {
            "/".to_string()
        }
    };

    // Combine base directory with relative URL
    let combined = format!("{}://{}{}{}", base_protocol, base_host, base_dir, url);

    // Resolve .. and . in the path
    Some(clean_url(&resolve_path(&combined)))
}

/// Parses a base URL into its components
///
/// Extracts protocol, host, and path from a URL.
///
/// # Arguments
///
/// * `base_url` - The URL to parse
///
/// # Returns
///
/// `Some((protocol, host, path))` if parsing succeeds, `None` otherwise
///
/// # Examples
///
/// ```
/// use spiderman::core::link_extractor::parse_base_url;
///
/// let (protocol, host, path) = parse_base_url("http://example.com/path/page.html").unwrap();
/// assert_eq!(protocol, "http");
/// assert_eq!(host, "example.com");
/// assert_eq!(path, "/path/page.html");
/// ```
pub fn parse_base_url(base_url: &str) -> Option<(String, String, String)> {
    // Extract protocol
    let (protocol, rest) = if let Some(pos) = base_url.find("://") {
        (&base_url[..pos], &base_url[pos + 3..])
    } else {
        return None;
    };

    // Extract host and path
    let (host, path) = if let Some(pos) = rest.find('/') {
        (&rest[..pos], &rest[pos..])
    } else {
        (rest, "/")
    };

    Some((protocol.to_string(), host.to_string(), path.to_string()))
}

/// Resolves relative path components (. and ..) in a URL
///
/// This function normalizes paths by resolving:
/// - `.` (current directory) - removed
/// - `..` (parent directory) - moves up one level
///
/// # Arguments
///
/// * `url` - The URL with potentially relative path components
///
/// # Returns
///
/// A String with resolved path
///
/// # Examples
///
/// ```
/// use spiderman::core::link_extractor::resolve_path;
///
/// assert_eq!(
///     resolve_path("http://example.com/a/b/../c"),
///     "http://example.com/a/c"
/// );
///
/// assert_eq!(
///     resolve_path("http://example.com/a/./b"),
///     "http://example.com/a/b"
/// );
/// ```
pub fn resolve_path(url: &str) -> String {
    // Split URL into base (protocol + host) and path
    let (base, path) = if let Some(pos) = url.find("://") {
        if let Some(slash_pos) = url[pos + 3..].find('/') {
            let split_pos = pos + 3 + slash_pos;
            (&url[..split_pos], &url[split_pos..])
        } else {
            return url.to_string();
        }
    } else {
        return url.to_string();
    };

    // Split path into components
    let parts: Vec<&str> = path.split('/').collect();
    let mut resolved: Vec<&str> = Vec::new();

    for part in parts.iter() {
        match *part {
            "." | "" => {
                // Skip current directory markers and empty parts (except first)
                if resolved.is_empty() {
                    resolved.push("");
                }
            }
            ".." => {
                // Go up one directory (remove last component)
                if resolved.len() > 1 {
                    resolved.pop();
                }
            }
            _ => {
                // Regular path component
                resolved.push(part);
            }
        }
    }

    // Reconstruct URL
    format!("{}{}", base, resolved.join("/"))
}

/// Cleans a URL by removing fragments and normalizing format
///
/// This function:
/// - Removes URL fragments (everything after `#`)
/// - Trims whitespace
/// - Normalizes the URL format
///
/// # Arguments
///
/// * `url` - The URL to clean
///
/// # Returns
///
/// A cleaned URL string
///
/// # Examples
///
/// ```
/// use spiderman::core::link_extractor::clean_url;
///
/// assert_eq!(
///     clean_url("http://example.com/page#section"),
///     "http://example.com/page"
/// );
///
/// assert_eq!(
///     clean_url("  http://example.com/  "),
///     "http://example.com/"
/// );
/// ```
pub fn clean_url(url: &str) -> String {
    let url = url.trim();

    // Remove fragment (everything after #)
    let url = if let Some(pos) = url.find('#') {
        &url[..pos]
    } else {
        url
    };

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== URL Validation Tests =====

    #[test]
    fn test_is_valid_url_with_valid_urls() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("/about"));
        assert!(is_valid_url("../page.html"));
        assert!(is_valid_url("page.html"));
    }

    #[test]
    fn test_is_valid_url_with_invalid_urls() {
        assert!(!is_valid_url("#section"));
        assert!(!is_valid_url("javascript:void(0)"));
        assert!(!is_valid_url("mailto:test@example.com"));
        assert!(!is_valid_url("tel:+1234567890"));
        assert!(!is_valid_url("data:image/png;base64,123"));
        assert!(!is_valid_url(""));
        assert!(!is_valid_url("   "));
    }

    // ===== URL Normalization Tests =====

    #[test]
    fn test_normalize_url_absolute_urls() {
        let base = "http://example.com";

        assert_eq!(
            normalize_url("http://other.com/page", base),
            Some("http://other.com/page".to_string())
        );

        assert_eq!(
            normalize_url("https://secure.com/page", base),
            Some("https://secure.com/page".to_string())
        );
    }

    #[test]
    fn test_normalize_url_absolute_paths() {
        let base = "http://example.com/some/path";

        assert_eq!(
            normalize_url("/about", base),
            Some("http://example.com/about".to_string())
        );

        assert_eq!(
            normalize_url("/contact/us", base),
            Some("http://example.com/contact/us".to_string())
        );
    }

    #[test]
    fn test_normalize_url_relative_paths() {
        let base = "http://example.com/blog/";

        assert_eq!(
            normalize_url("post.html", base),
            Some("http://example.com/blog/post.html".to_string())
        );

        let base_with_file = "http://example.com/blog/index.html";
        assert_eq!(
            normalize_url("post.html", base_with_file),
            Some("http://example.com/blog/post.html".to_string())
        );
    }

    #[test]
    fn test_normalize_url_protocol_relative() {
        assert_eq!(
            normalize_url("//cdn.example.com/file.js", "http://example.com"),
            Some("http://cdn.example.com/file.js".to_string())
        );

        assert_eq!(
            normalize_url("//cdn.example.com/file.js", "https://example.com"),
            Some("https://cdn.example.com/file.js".to_string())
        );
    }

    #[test]
    fn test_normalize_url_with_parent_directory() {
        let base = "http://example.com/a/b/c/";

        assert_eq!(
            normalize_url("../page.html", base),
            Some("http://example.com/a/b/page.html".to_string())
        );

        assert_eq!(
            normalize_url("../../page.html", base),
            Some("http://example.com/a/page.html".to_string())
        );
    }

    // ===== URL Parsing Tests =====

    #[test]
    fn test_parse_base_url() {
        let (protocol, host, path) =
            parse_base_url("http://example.com/path/to/page.html").unwrap();
        assert_eq!(protocol, "http");
        assert_eq!(host, "example.com");
        assert_eq!(path, "/path/to/page.html");
    }

    #[test]
    fn test_parse_base_url_no_path() {
        let (protocol, host, path) = parse_base_url("http://example.com").unwrap();
        assert_eq!(protocol, "http");
        assert_eq!(host, "example.com");
        assert_eq!(path, "/");
    }

    #[test]
    fn test_parse_base_url_with_port() {
        let (protocol, host, path) = parse_base_url("http://example.com:8080/page").unwrap();
        assert_eq!(protocol, "http");
        assert_eq!(host, "example.com:8080");
        assert_eq!(path, "/page");
    }

    // ===== Path Resolution Tests =====

    #[test]
    fn test_resolve_path_with_parent_directory() {
        assert_eq!(
            resolve_path("http://example.com/a/b/../c"),
            "http://example.com/a/c"
        );

        assert_eq!(
            resolve_path("http://example.com/a/b/../../c"),
            "http://example.com/c"
        );
    }

    #[test]
    fn test_resolve_path_with_current_directory() {
        assert_eq!(
            resolve_path("http://example.com/a/./b"),
            "http://example.com/a/b"
        );

        assert_eq!(
            resolve_path("http://example.com/./a/b"),
            "http://example.com/a/b"
        );
    }

    #[test]
    fn test_resolve_path_mixed() {
        assert_eq!(
            resolve_path("http://example.com/a/b/../c/./d"),
            "http://example.com/a/c/d"
        );
    }

    // ===== URL Cleaning Tests =====

    #[test]
    fn test_clean_url_removes_fragment() {
        assert_eq!(
            clean_url("http://example.com/page#section"),
            "http://example.com/page"
        );

        assert_eq!(
            clean_url("http://example.com/page#section1#section2"),
            "http://example.com/page"
        );
    }

    #[test]
    fn test_clean_url_trims_whitespace() {
        assert_eq!(
            clean_url("  http://example.com/page  "),
            "http://example.com/page"
        );
    }

    // ===== Link Extraction Tests =====

    #[test]
    fn test_extract_links_basic() {
        let html = r#"
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
        "#;

        let links = extract_links(html, "http://example.com");

        assert_eq!(links.len(), 2);
        assert!(links.contains(&"http://example.com/about".to_string()));
        assert!(links.contains(&"http://example.com/contact".to_string()));
    }

    #[test]
    fn test_extract_links_filters_invalid() {
        let html = r##"
            <a href="/valid">Valid</a>
            <a href="#section">Anchor</a>
            <a href="javascript:void(0)">JS</a>
            <a href="mailto:test@example.com">Email</a>
        "##;

        let links = extract_links(html, "http://example.com");

        assert_eq!(links.len(), 1);
        assert!(links.contains(&"http://example.com/valid".to_string()));
    }

    #[test]
    fn test_extract_links_deduplicates() {
        let html = r##"
            <a href="/page">Link 1</a>
            <a href="/page">Link 2</a>
            <a href="/page">Link 3</a>
        "##;

        let links = extract_links(html, "http://example.com");

        assert_eq!(links.len(), 1);
        assert!(links.contains(&"http://example.com/page".to_string()));
    }

    #[test]
    fn test_extract_links_mixed_formats() {
        let html = r##"
            <a href="http://external.com">External</a>
            <a href="/about">About</a>
            <a href="contact.html">Contact</a>
        "##;

        let links = extract_links(html, "http://example.com/");

        assert_eq!(links.len(), 3);
        assert!(links.contains(&"http://external.com".to_string()));
        assert!(links.contains(&"http://example.com/about".to_string()));
        assert!(links.contains(&"http://example.com/contact.html".to_string()));
    }

    #[test]
    fn test_extract_links_single_quotes() {
        let html = r##"<a href='/page1'>Page 1</a><a href='/page2'>Page 2</a>"##;

        let links = extract_links(html, "http://example.com");

        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_extract_links_empty_html() {
        let links = extract_links("", "http://example.com");
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_extract_links_no_links() {
        let html = "<p>No links here</p>";
        let links = extract_links(html, "http://example.com");
        assert_eq!(links.len(), 0);
    }
}
