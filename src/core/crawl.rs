use super::Spiderman;
use super::html_to_md::parser;

impl<'a> Spiderman<'a> {
    pub async fn crawl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch the HTML content from the URL
        self.fetch().await?;

        // Convert HTML to Markdown
        if let Some(html_content) = &self.html {
            let markdown_content = parser(html_content.clone());
            self.markdown = Some(markdown_content);

            // Debug: Print the markdown (you can remove this later)
            println!("=== Markdown Content ===");
            if let Some(md) = &self.markdown {
                println!("{}", md);
            }
        }

        // TODO: Store data into a db/fs

        Ok(())
    }
}
