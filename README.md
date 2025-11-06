# 🕷️ Spiderman Web Crawler

A fast, simple, and production-ready web crawler written in Rust. Perfect for feeding data to full-text search engines.

[![Tests](https://img.shields.io/badge/tests-104%20passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-2021-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

## Features

✨ **Simple & Clean API** - Easy to use, well-documented  
🚀 **Fast & Efficient** - Async I/O, minimal memory footprint  
🔍 **Smart Link Discovery** - Automatic URL extraction and normalization  
📊 **Metadata Extraction** - Extracts titles, descriptions, keywords  
💾 **Multiple Export Formats** - JSONL, JSON for easy integration  
🎯 **Configurable** - Max pages, domain filtering, custom output  
🧪 **Well-Tested** - 104 tests, production-ready  
📝 **Excellent Documentation** - Comprehensive guides and examples  

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Modules](#modules)
- [Output Format](#output-format)
- [Examples](#examples)
- [Testing](#testing)
- [Documentation](#documentation)
- [Contributing](#contributing)

---

## Quick Start

```rust
use spiderman::core::{Spiderman, CrawlConfig};

async_std::task::block_on(async {
    // Configure crawler
    let config = CrawlConfig::default()
        .with_max_pages(50)
        .with_output_dir("output")
        .with_allowed_domains(vec!["example.com".to_string()]);

    // Create crawler and start crawling
    let mut spider = Spiderman::new("example.com");
    let result = spider.crawl(config).await.unwrap();

    println!("✅ Crawled {} pages!", result.pages_crawled);
});
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
spiderman = "0.1.0"
async-std = "1.13"
```

Or run the standalone crawler:

```bash
cargo build --release
./target/release/spiderman
```

## Usage

### Basic Crawl

```rust
use spiderman::core::{Spiderman, CrawlConfig};

async_std::task::block_on(async {
    let mut spider = Spiderman::new("example.com");
    let config = CrawlConfig::default();
    let result = spider.crawl(config).await.unwrap();

    println!("Pages crawled: {}", result.pages_crawled);
    println!("URLs discovered: {}", result.urls_discovered);
});
```

### With Custom Configuration

```rust
let config = CrawlConfig::new()
    .with_max_pages(100)                    // Limit to 100 pages
    .with_allowed_domains(vec![             // Stay on these domains
        "example.com".to_string(),
        "www.example.com".to_string()
    ])
    .with_output_dir("crawled_data")        // Output directory
    .with_output_file("results.jsonl")      // Output filename
    .with_raw_html(true)                    // Store raw HTML
    .with_verbose(true);                    // Print progress

let mut spider = Spiderman::new("example.com");
let result = spider.crawl(config).await.unwrap();
```

### Process Results

```rust
let result = spider.crawl(config).await.unwrap();

for doc in result.documents {
    println!("URL: {}", doc.url());
    println!("Title: {}", doc.title());
    println!("Links: {}", doc.link_count());
    println!("Content: {} bytes", doc.content_length());
    println!();
}
```

## Configuration

### CrawlConfig Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_pages` | `Option<usize>` | `Some(50)` | Maximum pages to crawl |
| `allowed_domains` | `Option<Vec<String>>` | `None` | Restrict to specific domains |
| `output_dir` | `String` | `"output"` | Output directory path |
| `output_file` | `String` | `"crawl.jsonl"` | Output filename |
| `store_raw_html` | `bool` | `false` | Store original HTML |
| `verbose` | `bool` | `true` | Print progress logs |

### Example Configurations

**Small Site Crawl (< 100 pages)**
```rust
CrawlConfig::default()
    .with_max_pages(100)
    .with_allowed_domains(vec!["example.com".to_string()])
```

**Documentation Site**
```rust
CrawlConfig::default()
    .with_max_pages(500)
    .with_output_dir("docs_crawl")
    .with_raw_html(false)
```

**Development/Testing**
```rust
CrawlConfig::default()
    .with_max_pages(5)
    .with_verbose(true)
    .with_raw_html(true)
```

## Architecture

```
┌─────────────────────────────────────────────┐
│           Spiderman Crawler                 │
├─────────────────────────────────────────────┤
│                                             │
│  1. URL Manager (Queue + Dedup)            │
│     ↓                                       │
│  2. Webshooter (HTTP Fetch)                │
│     ↓                                       │
│  3. Link Extractor (Find URLs)             │
│     ↓                                       │
│  4. HTML to MD Parser (Convert)            │
│     ↓                                       │
│  5. Document Model (Structure)             │
│     ↓                                       │
│  6. Export System (Save)                   │
│                                             │
└─────────────────────────────────────────────┘
```

### Data Flow

```
Seed URL → URL Queue → Fetch HTML → Extract Links → Add to Queue
                           ↓
                    Parse to Markdown
                           ↓
                    Extract Metadata
                           ↓
                     Create Document
                           ↓
                    Export to JSONL
```

## Modules

### 1. **Webshooter** - HTTP Fetching
Fetches HTML content via raw TCP connections.

```rust
let mut spider = Spiderman::new("example.com");
// Fetching happens automatically in crawl()
```

### 2. **HTML to MD** - Content Conversion
Converts HTML to clean Markdown text.

```rust
use spiderman::core::html_to_md::parser;

let html = "<h1>Title</h1><p>Content</p>";
let markdown = parser(html.to_string());
// Output: "# Title\n\nContent"
```

### 3. **Link Extractor** - URL Discovery
Finds and normalizes URLs in HTML.

```rust
use spiderman::core::link_extractor::extract_links;

let html = r#"<a href="/about">About</a>"#;
let links = extract_links(html, "http://example.com");
// Output: ["http://example.com/about"]
```

### 4. **URL Manager** - Queue Management
Manages crawl queue and prevents duplicates.

```rust
use spiderman::core::url_manager::UrlManager;

let mut manager = UrlManager::new("http://example.com");
manager.set_max_pages(100);
manager.add_url("http://example.com/page1");
```

### 5. **Document** - Data Model
Structured representation of crawled pages.

```rust
use spiderman::core::document::Document;

let doc = Document::new(url, markdown, links)
    .with_title("Page Title")
    .with_description(Some("Description".to_string()));
```

### 6. **Export** - Save Results
Exports documents to files.

```rust
use spiderman::core::export::Exporter;

let exporter = Exporter::new("output");
exporter.export_document(&doc, "crawl.jsonl").unwrap();
```

## Output Format

### JSONL (Recommended)

Each line is a complete JSON document:

```jsonl
{"url":"http://example.com","title":"Example Domain","content":"# Example...","links":["http://example.com/about"],"crawled_at":"2024-01-15T10:30:00Z"}
{"url":"http://example.com/about","title":"About","content":"# About...","links":[],"crawled_at":"2024-01-15T10:30:01Z"}
```

**Benefits:**
- ✅ Stream-processable
- ✅ Easy to append
- ✅ One bad line doesn't corrupt file
- ✅ Perfect for large datasets

### JSON Document Structure

```json
{
  "url": "http://example.com",
  "title": "Example Domain",
  "description": "This domain is for examples",
  "content": "# Example Domain\n\nThis is the content...",
  "links": [
    "http://example.com/about",
    "http://example.com/contact"
  ],
  "crawled_at": "2024-01-15T10:30:00Z",
  "metadata": {
    "author": "IANA",
    "keywords": "example, domain"
  }
}
```

### Reading Output

**Using jq:**
```bash
cat output/crawl.jsonl | jq .
cat output/crawl.jsonl | jq '.url'
cat output/crawl.jsonl | jq 'select(.title | contains("Example"))'
```

**In Rust:**
```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use spiderman::core::Document;

let file = File::open("output/crawl.jsonl")?;
let reader = BufReader::new(file);

for line in reader.lines() {
    let doc: Document = serde_json::from_str(&line?)?;
    println!("{}: {}", doc.url(), doc.title());
}
```

## Examples

### Example 1: Simple Crawl

```rust
use spiderman::core::{Spiderman, CrawlConfig};

async_std::task::block_on(async {
    let config = CrawlConfig::default().with_max_pages(10);
    let mut spider = Spiderman::new("example.com");
    
    match spider.crawl(config).await {
        Ok(result) => println!("✅ Crawled {} pages", result.pages_crawled),
        Err(e) => eprintln!("❌ Error: {}", e),
    }
});
```

### Example 2: Domain-Restricted Crawl

```rust
let config = CrawlConfig::default()
    .with_max_pages(100)
    .with_allowed_domains(vec!["docs.example.com".to_string()]);

let mut spider = Spiderman::new("docs.example.com");
let result = spider.crawl(config).await?;
```

### Example 3: Export to Multiple Formats

```rust
use spiderman::core::export::Exporter;

let result = spider.crawl(config).await?;
let exporter = Exporter::new("output");

// Export as JSONL
exporter.export_batch(&result.documents, "crawl.jsonl")?;

// Export as JSON array
exporter.export_json_array(&result.documents, "crawl.json")?;
```

### Example 4: Custom Processing

```rust
let result = spider.crawl(config).await?;

for doc in result.documents {
    // Filter by title
    if doc.title().contains("API") {
        println!("API Doc found: {}", doc.url());
        
        // Extract specific metadata
        if let Some(author) = doc.get_metadata("author") {
            println!("  Author: {}", author);
        }
        
        // Save to custom location
        let json = doc.to_json_pretty()?;
        std::fs::write(format!("api_docs/{}.json", doc.title()), json)?;
    }
}
```

## Testing

Run all tests:

```bash
cargo test
```

Run specific module tests:

```bash
cargo test link_extractor::tests
cargo test url_manager::tests
cargo test document::tests
cargo test export::tests
```

Run with output:

```bash
cargo test -- --nocapture
```

### Test Coverage

- ✅ 104 tests total
- ✅ URL extraction and normalization (22 tests)
- ✅ URL queue management (23 tests)
- ✅ Document creation and metadata (21 tests)
- ✅ Export functionality (14 tests)
- ✅ HTML parsing (11 tests)
- ✅ Configuration (4 tests)
- ✅ Integration (9 tests)

## Documentation

### Generated Docs

View full API documentation:

```bash
cargo doc --open
```

### Guides

Comprehensive guides are available in the repository:

- **MODULES_GUIDE.md** - Detailed explanation of Link Extractor and URL Manager
- **DOCUMENT_MODULE_GUIDE.md** - Complete Document module reference
- **ARCHITECTURE.md** - System architecture and data flow
- **IMPLEMENTATION_SUMMARY.md** - Quick implementation reference

### Module Documentation

Each module has extensive inline documentation with examples:

```rust
// Example: Link Extractor documentation
use spiderman::core::link_extractor::extract_links;

/// Extracts all valid links from HTML content
/// See module docs for details: cargo doc --open
let links = extract_links(html, base_url);
```

## Performance

### Benchmarks (Approximate)

- **Speed**: ~10-50 pages/second (network dependent)
- **Memory**: ~50MB for 10K URLs in queue
- **Disk**: ~1KB per document (JSONL, no raw HTML)

### Optimizations

- ✅ Efficient data structures (VecDeque, HashSet)
- ✅ No recursive algorithms
- ✅ Minimal memory allocations
- ✅ Stream-based processing
- ✅ Async I/O

## Limitations

Current limitations (may be addressed in future versions):

- ❌ No HTTPS support (HTTP only via port 80)
- ❌ No robots.txt parsing (yet)
- ❌ No rate limiting (yet)
- ❌ No JavaScript rendering
- ❌ No concurrent requests (single-threaded crawling)

## Use Cases

Perfect for:

✅ Building search engine indices  
✅ Documentation aggregation  
✅ Content archiving  
✅ Link analysis  
✅ SEO auditing  
✅ Data mining  
✅ Website monitoring  

## Roadmap

- [ ] HTTPS support
- [ ] robots.txt parsing
- [ ] Rate limiting
- [ ] Concurrent crawling
- [ ] Sitemap.xml support
- [ ] Better error recovery
- [ ] CSV export format
- [ ] Database integration

## Contributing

Contributions are welcome! Areas for improvement:

1. Add HTTPS support (TLS library)
2. Implement robots.txt parser
3. Add rate limiting
4. Improve error handling
5. Add more export formats

## License

MIT License - See LICENSE file for details

## Author

Built with ❤️ using Rust

## Acknowledgments

- `html2text` crate for HTML parsing
- `regex` crate for URL extraction
- `serde` and `serde_json` for serialization
- `chrono` for timestamps

---

## Quick Reference

### Common Commands

```bash
# Build
cargo build --release

# Run
cargo run

# Test
cargo test

# Documentation
cargo doc --open

# Format
cargo fmt

# Lint
cargo clippy
```

### Import Everything

```rust
use spiderman::core::{
    Spiderman,
    CrawlConfig,
    CrawlResult,
    Document,
    Exporter,
};
```

---

**Happy Crawling! 🕷️**

For questions or issues, please open an issue on GitHub.