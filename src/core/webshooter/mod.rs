use super::Spiderman;
use async_std::{
    io::{BufReader, ReadExt, WriteExt},
    net::TcpStream,
};

/// Fetches HTML content from a given URL using raw TCP connections
///
/// This module provides web fetching functionality for the Spiderman web crawler.
/// It handles HTTP requests by manually implementing the HTTP protocol over TCP.
///
/// # Implementation Details
///
/// The fetching is done using raw TCP sockets with manual HTTP request construction:
/// - Establishes TCP connection to the host
/// - Sends HTTP GET request with proper headers
/// - Parses HTTP response headers
/// - Extracts and returns the response body
///
/// # Limitations
///
/// - Only supports HTTP (port 80), not HTTPS
/// - Does not follow redirects automatically
/// - Basic HTTP/1.1 implementation
/// - No support for chunked transfer encoding (uses Connection: close)
///
/// # Errors
///
/// This function will return an error if:
/// - The URL format is invalid (missing host or path)
/// - DNS resolution fails
/// - TCP connection cannot be established
/// - HTTP request/response parsing fails
/// - Network I/O errors occur

impl<'a> Spiderman<'a> {
    /// Fetches HTML content from the URL and stores it in the struct
    ///
    /// This method performs an HTTP GET request to the configured URL using
    /// raw TCP sockets and stores the retrieved HTML content in the `html` field.
    ///
    /// # How it works
    ///
    /// 1. Parses the URL to extract host and path
    /// 2. Establishes TCP connection on port 80 (HTTP)
    /// 3. Sends HTTP GET request with proper headers
    /// 4. Reads the response and extracts the body
    /// 5. Stores the HTML content in the struct
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Mutable reference to modify the html field
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the fetch was successful and HTML was stored
    /// * `Err(Box<dyn std::error::Error>)` - If any error occurred during fetching
    ///
    /// # Example
    ///
    /// ```no_run
    /// use spiderman::core::Spiderman;
    ///
    /// async_std::task::block_on(async {
    ///     let mut spider = Spiderman::new("example.com");
    ///     match spider.fetch().await {
    ///         Ok(_) => println!("Successfully fetched HTML"),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// });
    /// ```
    pub(crate) async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Parse the URL to extract host and path
        let (host, path) = parse_url(self.url)?;

        // Connect to the host on port 80 (HTTP)
        let address = format!("{}:80", host);
        let mut stream = TcpStream::connect(&address).await?;

        // Build HTTP GET request
        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             User-Agent: Spiderman/0.1.0 (Rust Web Crawler)\r\n\
             Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
             Connection: close\r\n\
             \r\n",
            path, host
        );

        // Send the HTTP request
        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        // Read the response
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_to_string(&mut response).await?;

        // Extract the body from the HTTP response
        let html = extract_body(&response)?;

        // Store the fetched HTML
        self.html = Some(html);
        Ok(())
    }
}

/// Parses a URL string to extract host and path components
///
/// Supports URLs in the following formats:
/// - `http://example.com/path`
/// - `example.com/path`
/// - `example.com`
///
/// # Arguments
///
/// * `url` - The URL string to parse
///
/// # Returns
///
/// * `Ok((host, path))` - Tuple containing the host and path
/// * `Err` - If the URL format is invalid
///
/// # Example
///
/// ```
/// let (host, path) = parse_url("http://example.com/test")?;
/// assert_eq!(host, "example.com");
/// assert_eq!(path, "/test");
/// ```
fn parse_url(url: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Remove protocol if present
    let url = url
        .trim_start_matches("http://")
        .trim_start_matches("https://");

    // Split host and path
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

/// Extracts the HTML body from an HTTP response
///
/// Parses the HTTP response and extracts the content after the headers.
/// The body starts after the first empty line (`\r\n\r\n` or `\n\n`).
///
/// # Arguments
///
/// * `response` - The complete HTTP response string
///
/// # Returns
///
/// * `Ok(body)` - The extracted body content
/// * `Err` - If the response format is invalid
///
/// # Example
///
/// ```
/// let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html>...</html>";
/// let body = extract_body(response)?;
/// assert_eq!(body, "<html>...</html>");
/// ```
fn extract_body(response: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Find the separator between headers and body
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
    fn test_parse_url_with_http_protocol() {
        let (host, path) = parse_url("http://example.com/test").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/test");
    }

    #[test]
    fn test_parse_url_with_https_protocol() {
        let (host, path) = parse_url("https://example.com/page").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/page");
    }

    #[test]
    fn test_parse_url_without_protocol() {
        let (host, path) = parse_url("example.com/about").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/about");
    }

    #[test]
    fn test_parse_url_without_path() {
        let (host, path) = parse_url("example.com").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/");
    }

    #[test]
    fn test_parse_url_with_subdomain() {
        let (host, path) = parse_url("http://www.example.com/page").unwrap();
        assert_eq!(host, "www.example.com");
        assert_eq!(path, "/page");
    }

    #[test]
    fn test_parse_url_with_deep_path() {
        let (host, path) = parse_url("example.com/path/to/resource").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/path/to/resource");
    }

    #[test]
    fn test_parse_url_empty_string() {
        let result = parse_url("");
        assert!(result.is_err(), "Empty URL should return error");
    }

    #[test]
    fn test_extract_body_with_crlf() {
        let response =
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body>Test</body></html>";
        let body = extract_body(response).unwrap();
        assert_eq!(body, "<html><body>Test</body></html>");
    }

    #[test]
    fn test_extract_body_with_lf() {
        let response = "HTTP/1.1 200 OK\nContent-Type: text/html\n\n<html><body>Test</body></html>";
        let body = extract_body(response).unwrap();
        assert_eq!(body, "<html><body>Test</body></html>");
    }

    #[test]
    fn test_extract_body_with_multiple_headers() {
        let response = "HTTP/1.1 200 OK\r\n\
                       Content-Type: text/html\r\n\
                       Content-Length: 100\r\n\
                       Server: TestServer\r\n\
                       \r\n\
                       <html>Content</html>";
        let body = extract_body(response).unwrap();
        assert_eq!(body, "<html>Content</html>");
    }

    #[test]
    fn test_extract_body_no_separator() {
        let response = "HTTP/1.1 200 OK";
        let result = extract_body(response);
        assert!(result.is_err(), "Response without separator should error");
    }

    #[test]
    fn test_extract_body_empty_body() {
        let response = "HTTP/1.1 200 OK\r\n\r\n";
        let body = extract_body(response).unwrap();
        assert_eq!(body, "");
    }

    #[test]
    fn test_fetch_real_website() {
        async_std::task::block_on(async {
            let mut spider = Spiderman::new("example.com");
            let result = spider.fetch().await;

            assert!(result.is_ok(), "Fetching example.com should succeed");
            assert!(spider.html.is_some(), "HTML should be stored after fetch");

            let html = spider.html.unwrap();
            assert!(!html.is_empty(), "HTML content should not be empty");
            assert!(
                html.to_lowercase().contains("example"),
                "Should contain example.com content"
            );
        });
    }

    #[test]
    fn test_fetch_with_path() {
        async_std::task::block_on(async {
            let mut spider = Spiderman::new("example.com/");
            let result = spider.fetch().await;

            assert!(result.is_ok(), "Fetching with path should succeed");
            assert!(spider.html.is_some(), "HTML should be stored");
        });
    }

    #[test]
    fn test_fetch_invalid_host() {
        async_std::task::block_on(async {
            let mut spider = Spiderman::new("this-domain-absolutely-does-not-exist-12345.com");
            let result = spider.fetch().await;

            assert!(result.is_err(), "Fetching invalid host should return error");
            assert!(spider.html.is_none(), "HTML should not be stored on error");
        });
    }

    #[test]
    fn test_fetch_updates_html_field() {
        async_std::task::block_on(async {
            let mut spider = Spiderman::new("example.com");

            // Initially, html should be None
            assert!(spider.html.is_none(), "HTML should be None before fetch");

            // Fetch the content
            spider.fetch().await.unwrap();

            // After fetch, html should be Some
            assert!(spider.html.is_some(), "HTML should be Some after fetch");

            // Verify content is not empty
            let html = spider.html.as_ref().unwrap();
            assert!(!html.is_empty(), "HTML content should not be empty");
        });
    }

    #[test]
    fn test_multiple_fetches() {
        async_std::task::block_on(async {
            let mut spider = Spiderman::new("example.com");

            // First fetch
            let result1 = spider.fetch().await;
            assert!(result1.is_ok(), "First fetch should succeed");
            let html1 = spider.html.clone();

            // Second fetch (should overwrite)
            let result2 = spider.fetch().await;
            assert!(result2.is_ok(), "Second fetch should succeed");
            let html2 = spider.html.clone();

            // Both should have content
            assert!(html1.is_some() && html2.is_some());

            // Content should be similar (may vary slightly due to server changes)
            assert_eq!(html1.unwrap().len(), html2.unwrap().len());
        });
    }

    #[test]
    fn test_fetch_with_http_prefix() {
        async_std::task::block_on(async {
            let mut spider = Spiderman::new("http://example.com");
            let result = spider.fetch().await;

            assert!(result.is_ok(), "Fetching with http:// prefix should work");
            assert!(spider.html.is_some(), "HTML should be stored");
        });
    }
}
