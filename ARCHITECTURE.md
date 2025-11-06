# Spiderman Web Crawler - Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      SPIDERMAN WEB CRAWLER                      │
└─────────────────────────────────────────────────────────────────┘

                              ┌────────────┐
                              │   Seed URL │
                              └──────┬─────┘
                                     │
                                     ▼
┌────────────────────────────────────────────────────────────────┐
│                        URL MANAGER                             │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ to_visit Queue:  [url1] → [url2] → [url3]              │ │
│  └──────────────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ visited Set:     {url1, url2, url3, url4, url5}         │ │
│  └──────────────────────────────────────────────────────────┘ │
└───────────────────┬────────────────────────────────────────────┘
                    │ get_next()
                    ▼
            ┌───────────────┐
            │  Current URL  │
            └───────┬───────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────────────┐
│                          WEBSHOOTER                            │
│                      (HTTP Fetcher)                            │
│                                                                │
│  • Opens TCP connection                                        │
│  • Sends HTTP GET request                                      │
│  • Receives HTML response                                      │
└───────────────────┬────────────────────────────────────────────┘
                    │
                    ▼
            ┌───────────────┐
            │   HTML Content│
            └───────┬───────┘
                    │
         ┌──────────┴──────────┐
         │                     │
         ▼                     ▼
┌──────────────────┐  ┌──────────────────┐
│  LINK EXTRACTOR  │  │   HTML TO MD     │
│                  │  │    PARSER        │
│ • Find <a> tags  │  │                  │
│ • Extract hrefs  │  │ • Convert HTML   │
│ • Normalize URLs │  │ • Clean format   │
│ • Filter invalid │  │ • Extract text   │
└────────┬─────────┘  └────────┬─────────┘
         │                     │
         ▼                     ▼
  ┌─────────────┐      ┌─────────────┐
  │  New URLs   │      │  Markdown   │
  │  [url4,     │      │  Content    │
  │   url5,     │      │             │
  │   url6]     │      │             │
  └──────┬──────┘      └──────┬──────┘
         │                     │
         │                     ▼
         │           ┌──────────────────┐
         │           │    DOCUMENT      │
         │           │     MODEL        │
         │           │  • url           │
         │           │  • title         │
         └──────────▶│  • content       │◀───┐
     add_url()       │  • timestamp     │    │
                     │  • links         │    │
                     └─────────┬────────┘    │
                               │             │
                               ▼             │
                     ┌──────────────────┐    │
                     │  EXPORT SYSTEM   │    │
                     │                  │    │
                     │ • Save to file   │    │
                     │ • Save to DB     │    │
                     │ • JSON/JSONL     │    │
                     └─────────┬────────┘    │
                               │             │
                               ▼             │
                     ┌──────────────────┐    │
                     │  SEARCH ENGINE   │    │
                     │   (Your System)  │    │
                     └──────────────────┘    │
                                             │
                     ┌───────────────────────┘
                     │  Loop continues
                     │  until queue empty
                     └───────────────────────┐
                                             │
                                             ▼
                                    ┌────────────────┐
                                    │  CRAWL DONE    │
                                    └────────────────┘
```

## Component Details

### 1. URL Manager (✅ Built)
- **Purpose**: Orchestrates the crawl
- **Input**: URLs to add
- **Output**: Next URL to crawl
- **Features**:
  - FIFO queue
  - Deduplication
  - Domain filtering
  - Max pages limit

### 2. Webshooter (✅ Built)
- **Purpose**: Fetches web pages
- **Input**: URL string
- **Output**: HTML content
- **Features**:
  - Raw TCP connection
  - HTTP/1.1 protocol
  - User-Agent header

### 3. Link Extractor (✅ Built)
- **Purpose**: Discovers new URLs
- **Input**: HTML + base URL
- **Output**: List of URLs
- **Features**:
  - Regex-based extraction
  - URL normalization
  - Invalid link filtering

### 4. HTML to MD Parser (✅ Built)
- **Purpose**: Converts HTML to text
- **Input**: HTML string
- **Output**: Markdown string
- **Features**:
  - Clean text extraction
  - Format preservation
  - Whitespace normalization

### 5. Document Model (❌ TODO)
- **Purpose**: Structured data format
- **Fields**:
  - url: String
  - title: String
  - content: String (markdown)
  - crawled_at: DateTime
  - links: Vec<String>

### 6. Export System (❌ TODO)
- **Purpose**: Save crawled data
- **Formats**:
  - JSONL (recommended)
  - JSON
  - CSV
- **Destinations**:
  - File system
  - Database

## Data Flow Example

```
1. Start: seed_url = "http://example.com"
   └─> URL Manager queue: [example.com]

2. Fetch: example.com
   └─> Webshooter returns HTML

3. Process HTML:
   ├─> Link Extractor finds: [/about, /contact, /blog]
   └─> HTML to MD produces: "# Example Domain\n\nThis is..."

4. Store:
   ├─> Create Document(url, title, content, timestamp, links)
   └─> Export to file/database

5. Queue new URLs:
   └─> URL Manager queue: [example.com/about, example.com/contact, example.com/blog]

6. Repeat from step 2 until queue empty
```

## Module Interaction

```
main.rs
  │
  └─> Spiderman::crawl()
        │
        ├─> UrlManager::new(seed_url)
        │     │
        │     └─> Manages: to_visit queue, visited set
        │
        └─> Loop:
              │
              ├─> UrlManager::get_next() → current_url
              │
              ├─> Spiderman::fetch(current_url)
              │     │
              │     └─> webshooter opens TCP, gets HTML
              │
              ├─> extract_links(html, current_url) → new_urls
              │     │
              │     └─> UrlManager::add_url(each new_url)
              │
              ├─> parser(html) → markdown
              │
              ├─> Create Document
              │
              └─> Export Document → file/db
```

## Crawl State Transitions

```
┌──────────┐
│  Start   │
└────┬─────┘
     │
     ▼
┌─────────────────┐
│ Initialize      │
│ URL Manager     │
│ with seed       │
└────┬────────────┘
     │
     ▼
┌─────────────────┐       ┌──────────────┐
│ Has next URL?   ├──No──→│ Crawl Done   │
└────┬────────────┘       └──────────────┘
     │ Yes
     ▼
┌─────────────────┐
│ Get next URL    │
└────┬────────────┘
     │
     ▼
┌─────────────────┐       ┌──────────────┐
│ Fetch HTML      ├──Err─→│ Log error,   │
└────┬────────────┘       │ continue     │
     │ Ok                 └──────────────┘
     ▼
┌─────────────────┐
│ Extract links   │
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ Add to queue    │
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ Parse to MD     │
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ Create Document │
└────┬────────────┘
     │
     ▼
┌─────────────────┐
│ Export Document │
└────┬────────────┘
     │
     └──────────────┐
                    │ Loop back
                    ▼
```

## File Structure

```
spiderman/
├── src/
│   ├── core/
│   │   ├── webshooter/          ✅ HTTP fetching
│   │   │   └── mod.rs
│   │   ├── html_to_md/          ✅ HTML to Markdown
│   │   │   └── mod.rs
│   │   ├── link_extractor/      ✅ Link discovery
│   │   │   └── mod.rs
│   │   ├── url_manager/         ✅ Queue management
│   │   │   └── mod.rs
│   │   ├── document/            ❌ TODO
│   │   │   └── mod.rs
│   │   ├── export/              ❌ TODO
│   │   │   └── mod.rs
│   │   ├── crawl.rs             ✅ Main crawl logic
│   │   └── mod.rs               ✅ Module exports
│   └── main.rs                  ✅ Entry point
├── Cargo.toml                   ✅ Dependencies
├── MODULES_GUIDE.md             ✅ Detailed guide
├── IMPLEMENTATION_SUMMARY.md    ✅ Quick summary
└── ARCHITECTURE.md              ✅ This file
```

## Performance Characteristics

### Time Complexity
- URL Manager operations: O(1)
- Link extraction: O(n) where n = HTML size
- Deduplication: O(1) lookups

### Space Complexity
- URLs stored: O(m) where m = unique URLs found
- No HTML stored in memory (streaming)

### Scalability
- Can handle millions of URLs (limited by RAM for HashSet)
- Efficient queue operations
- No recursive stack (iterative)

## Next Steps for MVP

1. **Document Model** (10 min)
   - Create struct with url, title, content, timestamp, links
   
2. **Export System** (15 min)
   - Implement JSONL writer
   - Save documents to file
   
3. **Integration** (20 min)
   - Wire everything in crawl()
   - Add error handling
   
4. **Testing** (15 min)
   - End-to-end test
   - Verify output format

Total: ~60 minutes to MVP! 🚀
