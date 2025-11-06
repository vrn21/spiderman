use super::Spiderman;
use async_std::{
    io::{BufReader, ReadExt},
    net::TcpStream,
};

impl<'a> Spiderman<'a> {
    pub(crate) async fn fetch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
}
