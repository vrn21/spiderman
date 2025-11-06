pub mod crawl;
pub mod html_to_md;
pub mod webshooter;

#[derive(Debug, Default)]
pub struct Spiderman<'a> {
    url: &'a str,
    html: Option<String>,
    markdown: Option<String>,
}

impl<'a> Spiderman<'a> {
    pub fn new(url: &'a str) -> Self {
        Self {
            url,
            html: None,
            markdown: None,
        }
    }

    /// Get the parsed markdown content
    pub fn get_markdown(&self) -> Option<&String> {
        self.markdown.as_ref()
    }
}
