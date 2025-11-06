# 🚀 Quick Start Guide

Get started with Spiderman Web Crawler in 5 minutes!

## Step 1: Clone or Create Project

```bash
# If you have the project
cd spiderman

# Or create new project
cargo new my-crawler
cd my-crawler
```

## Step 2: Build

```bash
cargo build --release
```

## Step 3: Run

```bash
cargo run
```

That's it! The crawler will:
- Start crawling from `example.com`
- Crawl up to 10 pages (configurable)
- Save results to `crawled_data/example.jsonl`
- Display progress in real-time

## Step 4: View Results

```bash
# View output file
cat crawled_data/example.jsonl

# Pretty print with jq (if installed)
cat crawled_data/example.jsonl | jq .

# Count crawled pages
wc -l crawled_data/example.jsonl
```

## Customize Configuration

Edit `src/main.rs`:

```rust
let config = core::CrawlConfig::default()
    .with_max_pages(100)                    // Change max pages
    .with_output_dir("my_output")           // Change output directory
    .with_allowed_domains(vec![             // Restrict domains
        "example.com".to_string()
    ]);

let mut spider = Spiderman::new("your-site.com");  // Change seed URL
```

## Common Configurations

### Small Site (< 100 pages)
```rust
CrawlConfig::default()
    .with_max_pages(100)
    .with_allowed_domains(vec!["site.com".to_string()])
```

### Documentation Site
```rust
CrawlConfig::default()
    .with_max_pages(500)
    .with_output_dir("docs")
```

### Testing/Development
```rust
CrawlConfig::default()
    .with_max_pages(5)
    .with_verbose(true)
```

## Use as Library

Add to `Cargo.toml`:
```toml
[dependencies]
spiderman = { path = "../spiderman" }
async-std = "1.13"
```

Use in your code:
```rust
use spiderman::core::{Spiderman, CrawlConfig};

async_std::task::block_on(async {
    let config = CrawlConfig::default().with_max_pages(50);
    let mut spider = Spiderman::new("example.com");
    let result = spider.crawl(config).await.unwrap();
    
    println!("Crawled {} pages!", result.pages_crawled);
});
```

## Troubleshooting

**Build errors?**
```bash
cargo clean
cargo build
```

**Tests failing?**
```bash
cargo test
```

**Need help?**
- Check `README.md` for full documentation
- Check `MODULES_GUIDE.md` for detailed explanations
- Run `cargo doc --open` for API docs

## What's Next?

1. **Read the full README.md** - Complete usage guide
2. **Explore the modules** - See MODULES_GUIDE.md
3. **Customize for your needs** - Modify CrawlConfig
4. **Process the output** - Feed to your search engine
5. **Contribute** - Add features or improvements

## Quick Command Reference

```bash
cargo build --release    # Build optimized binary
cargo run                # Run the crawler
cargo test               # Run all tests
cargo doc --open         # View documentation
cargo fmt                # Format code
cargo clippy             # Lint code
```

Happy Crawling! 🕷️

For full documentation, see README.md
