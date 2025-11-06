pub mod crawl;
pub mod webshooter;

#[derive(Debug, Default)]
pub struct Spiderman<'a> {
    url: &'a str,
    html: Option<String>,
}

impl<'a> Spiderman<'a> {
    pub fn new(url: &'a str) -> Self {
        Self { url, html: None }
    }
}
