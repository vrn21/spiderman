use core::Spiderman;

pub mod core;

fn main() {
    // Run the async crawler
    async_std::task::block_on(async {
        println!("╔══════════════════════════════════════════════════╗");
        println!("║        🕷️  Spiderman Web Crawler v0.1.0         ║");
        println!("╚══════════════════════════════════════════════════╝");
        println!();

        // Configure the crawler
        let config = core::CrawlConfig::default()
            .with_max_pages(10) // Limit to 10 pages for demo
            .with_output_dir("crawled_data")
            .with_output_file("example.jsonl")
            .with_verbose(true);

        // Optional: Set allowed domains to stay on one site
        // .with_allowed_domains(vec!["example.com".to_string()]);

        // Create crawler with seed URL
        let mut spider = Spiderman::new("example.com");

        // Start crawling
        match spider.crawl(config).await {
            Ok(result) => {
                println!("🎉 Success!");
                println!();
                println!("📊 Final Results:");
                println!("   ✓ {} pages crawled successfully", result.pages_crawled);
                println!("   ✗ {} pages failed", result.pages_failed);
                println!("   🔗 {} unique URLs discovered", result.urls_discovered);
                println!();
                println!(
                    "📁 Output saved to: crawled_data/example.jsonl ({} documents)",
                    result.documents.len()
                );
                println!();

                // Show sample of first document
                if let Some(first_doc) = result.documents.first() {
                    println!("📄 Sample Document:");
                    println!("   URL: {}", first_doc.url());
                    println!("   Title: {}", first_doc.title());
                    println!("   Links found: {}", first_doc.link_count());
                    println!("   Content length: {} bytes", first_doc.content_length());
                    println!();
                }

                println!("💡 Tip: View the output with:");
                println!("   cat crawled_data/example.jsonl | jq .");
                println!();
            }
            Err(e) => {
                eprintln!("❌ Crawl failed: {}", e);
                std::process::exit(1);
            }
        }
    });
}
