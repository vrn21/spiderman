# Spiderman Web Crawler - Modules Guide

This guide provides a comprehensive explanation of the Link Extractor and URL Manager modules, which are the core components that enable the web crawler to discover and manage URLs.

---

## Table of Contents

1. [Link Extractor Module](#link-extractor-module)
2. [URL Manager Module](#url-manager-module)
3. [How They Work Together](#how-they-work-together)
4. [Complete Example](#complete-example)

---

## Link Extractor Module

**Location:** `src/core/link_extractor/mod.rs`

### Purpose

The Link Extractor is responsible for discovering new URLs from HTML content. It parses HTML, finds all links, and converts them into absolute URLs that can be crawled.

### Key Functions

#### 1. `extract_links(html: &str, base_url: &str) -> Vec<String>`

The main entry point that extracts all valid, unique links from HTML.

**Example:**
```rust
use spiderman::core::link_extractor::extract_links;

let html = r##"
    <a href="/about">About</a>
    <a href="https://external.com">External</a>
    <a href="#section">Jump</a>
"##;

let links = extract_links(html, "http://example.com");
// Result: ["http://example.com/about", "https://external.com"]
```

**Process Flow:**
```
HTML Input
    ↓
Find all <a href="..."> tags (regex)
    ↓
Extract href attributes
    ↓
Filter invalid URLs (anchors, javascript:, mailto:)
    ↓
Normalize to absolute URLs
    ↓
Deduplicate (HashSet)
    ↓
Return Vec<String>
```

#### 2. `is_valid_url(url: &str) -> bool`

Validates whether a URL should be crawled.

**Filters out:**
- Fragment-only URLs: `#section`
- JavaScript: `javascript:void(0)`
- Mailto links: `mailto:email@example.com`
- Telephone: `tel:+1234567890`
- Data URLs: `data:image/png;base64,...`
- Empty strings

**Example:**
```rust
assert!(is_valid_url("http://example.com"));      // ✓
assert!(is_valid_url("/about"));                   // ✓
assert!(!is_valid_url("#section"));                // ✗ anchor
assert!(!is_valid_url("javascript:void(0)"));      // ✗ javascript
assert!(!is_valid_url("mailto:test@example.com")); // ✗ mailto
```

#### 3. `normalize_url(url: &str, base_url: &str) -> Option<String>`

Converts relative URLs to absolute URLs.

**Handles different URL formats:**

| Input URL | Base URL | Output |
|-----------|----------|--------|
| `http://example.com/page` | (any) | `http://example.com/page` |
| `/about` | `http://example.com` | `http://example.com/about` |
| `contact.html` | `http://example.com/blog/` | `http://example.com/blog/contact.html` |
| `../page` | `http://example.com/a/b/` | `http://example.com/a/page` |
| `//cdn.com/file` | `http://example.com` | `http://cdn.com/file` |

**Example:**
```rust
use spiderman::core::link_extractor::normalize_url;

let base = "http://example.com/blog/";

// Relative path
assert_eq!(
    normalize_url("post.html", base),
    Some("http://example.com/blog/post.html".to_string())
);

// Absolute path
assert_eq!(
    normalize_url("/about", base),
    Some("http://example.com/about".to_string())
);

// Parent directory
assert_eq!(
    normalize_url("../contact", base),
    Some("http://example.com/contact".to_string())
);
```

#### 4. `resolve_path(url: &str) -> String`

Resolves relative path components (`.` and `..`) in URLs.

**Example:**
```rust
use spiderman::core::link_extractor::resolve_path;

// Resolves parent directory
assert_eq!(
    resolve_path("http://example.com/a/b/../c"),
    "http://example.com/a/c"
);

// Resolves current directory
assert_eq!(
    resolve_path("http://example.com/a/./b"),
    "http://example.com/a/b"
);
```

#### 5. `clean_url(url: &str) -> String`

Cleans URLs by removing fragments and normalizing format.

**Example:**
```rust
use spiderman::core::link_extractor::clean_url;

assert_eq!(
    clean_url("http://example.com/page#section"),
    "http://example.com/page"
);
```

### How Link Extraction Works

Let's walk through a complete example:

```rust
let html = r##"
<html>
    <body>
        <nav>
            <a href="/home">Home</a>
            <a href="/about">About</a>
            <a href="contact.html">Contact</a>
        </nav>
        <main>
            <a href="http://external.com">External Link</a>
            <a href="#top">Back to Top</a>
            <a href="javascript:alert('hi')">Click</a>
            <a href="mailto:info@example.com">Email Us</a>
        </main>
        <footer>
            <a href="../legal/terms.html">Terms</a>
        </footer>
    </body>
</html>
"##;

let base_url = "http://example.com/blog/post/";
let links = extract_links(html, base_url);

// Results:
// [
//     "http://example.com/home",           // absolute path
//     "http://example.com/about",          // absolute path
//     "http://example.com/blog/post/contact.html",  // relative
//     "http://external.com",               // external absolute
//     "http://example.com/blog/legal/terms.html"    // parent dir
// ]
```

**Step-by-Step Process:**

1. **Extraction:** Regex finds all `<a href="...">` tags
2. **Validation:** Filters out `#top`, `javascript:`, `mailto:`
3. **Normalization:**
   - `/home` → `http://example.com/home`
   - `contact.html` → `http://example.com/blog/post/contact.html`
   - `../legal/terms.html` → `http://example.com/blog/legal/terms.html`
4. **Deduplication:** Returns unique URLs only

---

## URL Manager Module

**Location:** `src/core/url_manager/mod.rs`

### Purpose

The URL Manager orchestrates the crawling process by maintaining a queue of URLs to visit and tracking which URLs have already been crawled to prevent duplicates.

### Architecture

```
┌─────────────────────────────────────┐
│         URL Manager                 │
├─────────────────────────────────────┤
│                                     │
│  to_visit (Queue - VecDeque)       │
│  ┌──────────────────────────────┐  │
│  │ Front → url2 → url3 → Back   │  │
│  └──────────────────────────────┘  │
│          ↓ pop_front()             │
│                                     │
│  visited (HashSet)                 │
│  ┌──────────────────────────────┐  │
│  │ {url1, url2, url3, url4...}  │  │
│  └──────────────────────────────┘  │
│                                     │
│  Config:                           │
│  - max_pages: Option<usize>        │
│  - allowed_domains: Vec<String>    │
│                                     │
└─────────────────────────────────────┘
```

### Core Struct

```rust
pub struct UrlManager {
    to_visit: VecDeque<String>,      // Queue of URLs waiting to be crawled
    visited: HashSet<String>,        // URLs already seen (queued or crawled)
    max_pages: Option<usize>,        // Optional limit on pages to crawl
    allowed_domains: Option<Vec<String>>, // Optional domain restrictions
}
```

### Key Methods

#### 1. `new(seed_url: &str) -> Self`

Creates a new URL Manager with a seed URL (the starting point).

**Example:**
```rust
use spiderman::core::url_manager::UrlManager;

let manager = UrlManager::new("http://example.com");
// Queue: [example.com]
// Visited: {example.com}
```

#### 2. `add_url(&mut self, url: &str) -> bool`

Adds a URL to the crawl queue with validation and deduplication.

**Checks performed:**
1. **Normalization:** Converts URL to standard form
2. **Duplicate check:** Rejects if already visited
3. **Domain filter:** Rejects if not in allowed domains (if configured)
4. **Max pages:** Rejects if limit reached

**Returns:** `true` if added, `false` if rejected

**Example:**
```rust
let mut manager = UrlManager::new("http://example.com");

assert!(manager.add_url("http://example.com/about"));     // ✓ Added
assert!(!manager.add_url("http://example.com/about"));    // ✗ Duplicate
```

#### 3. `get_next(&mut self) -> Option<String>`

Gets the next URL to crawl from the queue (FIFO - First In, First Out).

**Returns:** 
- `Some(url)` if there's a URL to crawl
- `None` if queue is empty or max pages limit reached

**Example:**
```rust
let mut manager = UrlManager::new("http://example.com");
manager.add_url("http://example.com/page1");
manager.add_url("http://example.com/page2");

// Get URLs in order
assert_eq!(manager.get_next(), Some("http://example.com".to_string()));
assert_eq!(manager.get_next(), Some("http://example.com/page1".to_string()));
assert_eq!(manager.get_next(), Some("http://example.com/page2".to_string()));
assert_eq!(manager.get_next(), None); // Queue empty
```

#### 4. `has_next(&self) -> bool`

Checks if there are more URLs in the queue.

**Example:**
```rust
let mut manager = UrlManager::new("http://example.com");
assert!(manager.has_next());  // true

manager.get_next();
assert!(!manager.has_next()); // false (queue empty)
```

#### 5. `is_visited(&self, url: &str) -> bool`

Checks if a URL has already been visited (crawled or queued).

**Example:**
```rust
let mut manager = UrlManager::new("http://example.com");
assert!(manager.is_visited("http://example.com"));      // true (seed URL)
assert!(!manager.is_visited("http://example.com/new")); // false
```

#### 6. `set_max_pages(&mut self, max: usize)`

Sets a limit on the total number of pages to crawl.

**Example:**
```rust
let mut manager = UrlManager::new("http://example.com");
manager.set_max_pages(10); // Only crawl 10 pages total

// After 10 pages, get_next() returns None
```

#### 7. `set_allowed_domains(&mut self, domains: Vec<String>)`

Restricts crawling to specific domains only.

**Example:**
```rust
let mut manager = UrlManager::new("http://example.com");
manager.set_allowed_domains(vec![
    "example.com".to_string(),
    "www.example.com".to_string()
]);

// Only URLs from these domains will be added
assert!(manager.add_url("http://example.com/page"));      // ✓
assert!(manager.add_url("http://www.example.com/page"));  // ✓
assert!(!manager.add_url("http://external.com/page"));    // ✗
```

#### 8. `stats(&self) -> (usize, usize, usize)`

Returns crawl statistics: (total visited, queued, processed).

**Example:**
```rust
let mut manager = UrlManager::new("http://example.com");
manager.add_url("http://example.com/page1");
manager.add_url("http://example.com/page2");

let (total, queued, processed) = manager.stats();
// total = 3 (all URLs seen)
// queued = 3 (waiting in queue)
// processed = 0 (none crawled yet)

manager.get_next(); // Crawl first URL

let (total, queued, processed) = manager.stats();
// total = 3
// queued = 2
// processed = 1
```

### URL Normalization

URLs are normalized before storage to ensure proper deduplication:

```rust
use spiderman::core::url_manager::normalize_url_for_storage;

// Lowercase
assert_eq!(
    normalize_url_for_storage("HTTP://EXAMPLE.COM"),
    "http://example.com"
);

// Remove trailing slash
assert_eq!(
    normalize_url_for_storage("http://example.com/page/"),
    "http://example.com/page"
);

// Remove default ports
assert_eq!(
    normalize_url_for_storage("http://example.com:80/page"),
    "http://example.com/page"
);

// Remove fragments
assert_eq!(
    normalize_url_for_storage("http://example.com/page#section"),
    "http://example.com/page"
);
```

This ensures:
- `http://example.com/page` and `http://example.com/page/` are treated as the same URL
- `HTTP://EXAMPLE.COM` and `http://example.com` are treated as the same URL

### Domain Extraction

```rust
use spiderman::core::url_manager::extract_domain;

assert_eq!(
    extract_domain("http://example.com/page"),
    Some("example.com".to_string())
);

assert_eq!(
    extract_domain("http://www.example.com:8080/page"),
    Some("www.example.com".to_string())
);
```

---

## How They Work Together

The Link Extractor and URL Manager work together to enable recursive web crawling:

### Crawl Flow

```
┌──────────────────────────────────────────────────────────┐
│ 1. Initialize URL Manager with seed URL                 │
│    manager = UrlManager::new("http://example.com")      │
│    Queue: [example.com]                                  │
└──────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────┐
│ 2. Get next URL to crawl                                 │
│    url = manager.get_next()                              │
│    Current: "http://example.com"                         │
│    Queue: []                                             │
└──────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────┐
│ 3. Fetch HTML from URL                                   │
│    html = fetch(url)                                     │
└──────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────┐
│ 4. Extract links from HTML                               │
│    links = extract_links(html, url)                      │
│    Found: ["/about", "/contact", "/blog"]                │
└──────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────┐
│ 5. Add discovered links to queue                         │
│    for link in links:                                    │
│        manager.add_url(link)                             │
│    Queue: [example.com/about, example.com/contact, ...]  │
└──────────────────────────────────────────────────────────┘
                           ↓
┌──────────────────────────────────────────────────────────┐
│ 6. Repeat from step 2 until queue is empty               │
└──────────────────────────────────────────────────────────┘
```

### Detailed Example

```rust
use spiderman::core::link_extractor::extract_links;
use spiderman::core::url_manager::UrlManager;

// Step 1: Initialize with seed URL
let mut manager = UrlManager::new("http://example.com");

// Step 2: Crawl loop
while let Some(url) = manager.get_next() {
    println!("Crawling: {}", url);
    
    // Step 3: Fetch HTML (simplified)
    let html = fetch_html(&url); // Your fetch function
    
    // Step 4: Extract links
    let links = extract_links(&html, &url);
    println!("Found {} links", links.len());
    
    // Step 5: Add links to queue
    for link in links {
        if manager.add_url(&link) {
            println!("  Queued: {}", link);
        }
    }
    
    // Optional: Check stats
    let (total, queued, processed) = manager.stats();
    println!("Progress: {}/{} processed, {} queued", processed, total, queued);
}

println!("Crawl complete!");
```

### Example Output

```
Crawling: http://example.com
Found 3 links
  Queued: http://example.com/about
  Queued: http://example.com/contact
  Queued: http://example.com/blog
Progress: 1/4 processed, 3 queued

Crawling: http://example.com/about
Found 2 links
  Queued: http://example.com/team
Progress: 2/5 processed, 3 queued

Crawling: http://example.com/contact
Found 0 links
Progress: 3/5 processed, 2 queued

Crawling: http://example.com/blog
Found 5 links
  Queued: http://example.com/blog/post1
  Queued: http://example.com/blog/post2
Progress: 4/7 processed, 3 queued

... continues until queue is empty ...

Crawl complete!
```

---

## Complete Example

Here's a complete, working example that demonstrates both modules:

```rust
use spiderman::core::link_extractor::extract_links;
use spiderman::core::url_manager::UrlManager;

fn main() {
    // Create URL manager with seed
    let mut manager = UrlManager::new("http://example.com");
    
    // Optional: Configure limits
    manager.set_max_pages(50); // Limit to 50 pages
    manager.set_allowed_domains(vec!["example.com".to_string()]);
    
    // Track crawled pages
    let mut crawled_pages = Vec::new();
    
    // Crawl loop
    while let Some(url) = manager.get_next() {
        println!("\n[Crawling] {}", url);
        
        // Fetch HTML (you'd implement this)
        match fetch_html(&url) {
            Ok(html) => {
                // Extract links
                let links = extract_links(&html, &url);
                println!("  Found {} links", links.len());
                
                // Add to queue
                let mut added = 0;
                for link in links {
                    if manager.add_url(&link) {
                        added += 1;
                    }
                }
                println!("  Added {} new URLs to queue", added);
                
                // Save crawled page
                crawled_pages.push(url.clone());
                
                // Show progress
                let (total, queued, processed) = manager.stats();
                println!("  Progress: {}/{} pages processed", processed, total);
            }
            Err(e) => {
                eprintln!("  Error: {}", e);
            }
        }
    }
    
    println!("\n✓ Crawl complete!");
    println!("Total pages crawled: {}", crawled_pages.len());
}

// Placeholder fetch function
fn fetch_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Your HTTP fetching logic here
    // Return HTML content
    Ok("<html>...</html>".to_string())
}
```

---

## Best Practices

### 1. Always Set Domain Restrictions

```rust
let mut manager = UrlManager::new("http://example.com");
manager.set_allowed_domains(vec!["example.com".to_string()]);
```

This prevents your crawler from following external links and crawling the entire internet.

### 2. Set Reasonable Limits

```rust
manager.set_max_pages(1000); // Reasonable limit
```

This prevents runaway crawls on large sites.

### 3. Handle Errors Gracefully

```rust
while let Some(url) = manager.get_next() {
    match fetch_and_parse(&url) {
        Ok(links) => {
            for link in links {
                manager.add_url(&link);
            }
        }
        Err(e) => {
            eprintln!("Error crawling {}: {}", url, e);
            // Continue with next URL
        }
    }
}
```

Don't let one failed page stop your entire crawl.

### 4. Monitor Progress

```rust
let (total, queued, processed) = manager.stats();
if processed % 10 == 0 {
    println!("Progress: {}/{} pages", processed, total);
}
```

### 5. Respect Websites

- Add delays between requests (not implemented yet, but recommended)
- Respect robots.txt (not implemented yet, but recommended)
- Use a descriptive User-Agent (already implemented in webshooter)

---

## Summary

### Link Extractor
- **Purpose:** Discovers URLs in HTML
- **Input:** HTML string + base URL
- **Output:** List of absolute URLs
- **Key Features:** Validation, normalization, deduplication

### URL Manager
- **Purpose:** Organizes crawling workflow
- **Input:** URLs to add to queue
- **Output:** Next URL to crawl
- **Key Features:** Queue management, deduplication, domain filtering, limits

### Together
They enable the crawler to:
1. Start with one URL (seed)
2. Discover links on that page
3. Add them to a queue
4. Crawl each URL in the queue
5. Repeat until done

This is the foundation of web crawling! 🕷️