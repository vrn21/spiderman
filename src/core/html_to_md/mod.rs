/// Converts HTML content to Markdown format
///
/// This function takes raw HTML content and converts it to clean Markdown text,
/// removing unnecessary HTML tags and formatting while preserving the content structure.
///
/// # Arguments
///
/// * `html` - A String containing the HTML content to be converted
///
/// # Returns
///
/// * A String containing the Markdown-formatted content
///
/// # Example
///
/// ```
/// let html = String::from("<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>");
/// let markdown = parser(html);
/// // Returns: "# Hello World\n\nThis is a **test**.\n\n"
/// ```
pub(crate) fn parser(html: String) -> String {
    // Configure html2text with appropriate settings for web crawling
    // Using RichDecorator for better Markdown-like formatting
    let markdown = html2text::from_read(
        html.as_bytes(),
        usize::MAX, // No line wrapping - preserve content width
    );

    // Clean up the markdown output
    clean_markdown(markdown)
}

/// Cleans up the generated markdown by removing excessive whitespace
/// and normalizing formatting
fn clean_markdown(markdown: String) -> String {
    // Remove excessive blank lines (more than 2 consecutive newlines)
    let cleaned = markdown.lines().collect::<Vec<&str>>().join("\n");

    // Normalize multiple consecutive blank lines to at most 2
    let mut result = String::new();
    let mut blank_count = 0;

    for line in cleaned.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                result.push('\n');
            }
        } else {
            blank_count = 0;
            result.push_str(line);
            result.push('\n');
        }
    }

    // Trim leading and trailing whitespace
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_html_to_markdown() {
        let html = String::from("<h1>Hello World</h1><p>This is a test.</p>");
        let result = parser(html);
        assert!(result.contains("Hello World"));
        assert!(result.contains("This is a test"));
    }

    #[test]
    fn test_html_with_links() {
        let html = String::from(r#"<a href="https://example.com">Example Link</a>"#);
        let result = parser(html);
        assert!(result.contains("Example Link"));
    }

    #[test]
    fn test_html_with_lists() {
        let html = String::from("<ul><li>Item 1</li><li>Item 2</li></ul>");
        let result = parser(html);
        assert!(result.contains("Item 1"));
        assert!(result.contains("Item 2"));
    }

    #[test]
    fn test_empty_html() {
        let html = String::from("");
        let result = parser(html);
        assert_eq!(result, "");
    }

    #[test]
    fn test_clean_markdown_removes_excessive_blank_lines() {
        let markdown = String::from("Line 1\n\n\n\n\nLine 2");
        let result = clean_markdown(markdown);
        assert!(!result.contains("\n\n\n\n"));
    }
}
