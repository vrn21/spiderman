use super::Spiderman;

impl<'a> Spiderman<'a> {
    pub async fn crawl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.fetch().await?;

        // Process the fetched HTML here using Apache tika

        //extract data from html

        //store data into a db/fs

        Ok(())
    }
}
