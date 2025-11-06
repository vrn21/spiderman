use async_std::{
    io::{BufReader, ReadExt},
    net::TcpStream,
};

#[derive(Debug, Default)]
struct WebPage<'a> {
    url: &'a str,
    html: Option<String>,
}

impl WebPage<'_> {
    pub fn new(url: &str) -> Self {
        Self::default()
    }

    pub async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = TcpStream::connect(self.url).await?;
        let mut reader = BufReader::new(socket);
        let mut html = String::new();

        match reader.read_to_string(&mut html).await {
            Ok(_) => {
                self.html = Some(html);
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn crawl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.fetch().await?;

        // Process the fetched HTML here using Apache tika

        //extract data from html

        //store data into a db/fs

        Ok(())
    }
}

fn main() {
    async_std::task::block_on(async {
        let mut page = WebPage::new("https://example.com");
        page.crawl().await.unwrap();
    });
}
