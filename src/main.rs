use core::Spiderman;

pub mod core;
fn main() {
    async_std::task::block_on(async {
        let mut page = Spiderman::new("https://example.com");
        page.crawl().await.unwrap();
    });
}
