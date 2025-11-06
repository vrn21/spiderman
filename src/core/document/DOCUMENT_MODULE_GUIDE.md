# Document Module - Comprehensive Guide

## Overview

The **Document Module** is the core data structure for representing crawled web pages in the Spiderman crawler. It stores the URL, content, metadata, links, and timestamps for each crawled page in a structured, serializable format.

---

## Table of Contents

1. [Purpose](#purpose)
2. [Architecture](#architecture)
3. [The Document Struct](#the-document-struct)
4. [Metadata Extraction](#metadata-extraction)
5. [Usage Examples](#usage-examples)
6. [Serialization](#serialization)
7. [API Reference](#api-reference)
8. [Testing](#testing)

---

## Purpose

The Document module serves as:

1. **Data Model**: Standardized representation of crawled pages
2. **Storage Format**: JSON-serializable for persistence
3. **Metadata Container**: Extracts and stores page metadata (title, description, etc.)
4. **Search Engine Interface**: Provides structured data for full-text search indexing

### Why You Need It

Without a Document model, your crawler just has raw strings. With it, you have:
- Structured data for your search engine
- Metadata for better search relevance
- Timestamps for freshness tracking
- Links for graph analysis
- Serialization for storage/transmission

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      DOCUMENT MODULE                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │              Document Struct                          │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │ • url: String                                         │ │
│  │ • title: String                                       │ │
│  │ • description: Option<String>                         │ │
│  │ • content: String (Markdown)                          │ │
│  │ • raw_html: Option<String>                            │ │
│  │ • links: Vec<String>                                  │ │
│  │ • crawled_at: DateTime<Utc>                           │ │
│  │ • metadata: HashMap<String, String>                   │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │          Metadata Extraction Functions                │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │ • extract_metadata(html) → Metadata                   │ │
│  │ • extract_title(html) → Option<String>                │ │
│  │ • extract_meta_tags(html, metadata)                   │ │
│  │ • decode_html_entities(text) → String                 │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │           Serialization (JSON)                        │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │ • to_json() → Result<String>                          │ │
│  │ • to_json_pretty() → Result<String>                   │ │
│  │ • from_json(json) → Result<Document>                  │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## The Document Struct

### Fields

```rust
pub struct Document {
    url: String,                      // Page URL
    title: String,                    // Page title
    description: Option<String>,      // Meta description
    content: String,                  // Markdown content
    raw_html: Option<String>,         // Original HTML (optional)
    links: Vec<String>,               // Outbound links
    crawled_at: DateTime<Utc>,        // Timestamp
    metadata: HashMap<String, String>, // Additional metadata
}
```

### Field Descriptions

#### `url: String`
- The full URL of the crawled page
- Example: `"http://example.com/about"`
- **Required**: Always present

#### `title: String`
- Page title extracted from `<title>` tag
- Falls back to empty string if not found
- **Required**: Always present (but may be empty)

#### `description: Option<String>`
- Meta description from `<meta name="description">`
- `None` if not present
- **Optional**: May be absent

#### `content: String`
- The main page content in Markdown format
- Generated by the `html_to_md` parser
- **Required**: Always present

#### `raw_html: Option<String>`
- The original HTML source (optional)
- Useful for debugging or re-processing
- **Optional**: Only stored if explicitly set

#### `links: Vec<String>`
- List of outbound URLs found on the page
- Extracted by the link extractor
- **Required**: Always present (but may be empty)

#### `crawled_at: DateTime<Utc>`
- UTC timestamp of when the page was crawled
- Automatically set to current time on creation
- **Required**: Always present

#### `metadata: HashMap<String, String>`
- Additional key-value metadata
- Examples: author, keywords, custom fields
- **Optional**: May be empty

---

## Metadata Extraction

### The `Metadata` Struct

```rust
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub author: Option<String>,
    pub other: HashMap<String, String>,
}
```

### Extraction Function

```rust
pub fn extract_metadata(html: &str) -> Metadata
```

Extracts metadata from HTML `<head>` section:

#### What It Extracts

1. **Title** - From `<title>` tag
   ```html
   <title>Page Title</title>
   ```

2. **Description** - From meta tag
   ```html
   <meta name="description" content="Page description">
   ```

3. **Keywords** - From meta tag
   ```html
   <meta name="keywords" content="keyword1, keyword2">
   ```

4. **Author** - From meta tag
   ```html
   <meta name="author" content="John Doe">
   ```

5. **Custom Meta Tags** - Any other meta tags
   ```html
   <meta name="custom-field" content="custom value">
   ```

### How It Works

```
HTML Input
    ↓
Extract <title> tag using regex
    ↓
Find all <meta> tags using regex
    ↓
Parse name and content attributes
    ↓
Decode HTML entities (&amp;, &lt;, etc.)
    ↓
Return Metadata struct
```

---

## Usage Examples

### Example 1: Basic Document Creation

```rust
use spiderman::core::document::Document;

// Create a simple document
let doc = Document::new(
    "http://example.com",
    "# Example Page\n\nThis is the content".to_string(),
    vec!["http://example.com/about".to_string()]
);

println!("URL: {}", doc.url());
println!("Content: {}", doc.content());
println!("Links: {}", doc.link_count());
```

### Example 2: Using Builder Pattern

```rust
use spiderman::core::document::Document;

let doc = Document::new(
    "http://example.com",
    "# Content".to_string(),
    vec![]
)
.with_title("Example Page".to_string())
.with_description(Some("A test page".to_string()))
.with_metadata("author", "John Doe")
.with_metadata("keywords", "example, test");

assert_eq!(doc.title(), "Example Page");
assert_eq!(doc.get_metadata("author"), Some("John Doe"));
```

### Example 3: Extracting Metadata from HTML

```rust
use spiderman::core::document::{Document, extract_metadata};

let html = r#"
    <html>
        <head>
            <title>My Blog Post</title>
            <meta name="description" content="An interesting article">
            <meta name="author" content="Jane Doe">
        </head>
        <body>
            <h1>Welcome</h1>
        </body>
    </html>
"#;

// Extract metadata
let metadata = extract_metadata(html);

// Create document with metadata
let doc = Document::new(
    "http://blog.example.com/post",
    "# Welcome\n\n...".to_string(),
    vec![]
)
.with_title(metadata.title.unwrap_or_default())
.with_description(metadata.description)
.with_metadata("author", &metadata.author.unwrap_or_default());

println!("Title: {}", doc.title());
println!("Description: {:?}", doc.description());
```

### Example 4: Complete Crawl Pipeline

```rust
use spiderman::core::document::{Document, extract_metadata};
use spiderman::core::link_extractor::extract_links;
use spiderman::core::html_to_md::parser;

async fn create_document_from_crawl(url: &str, html: String) -> Document {
    // 1. Extract links
    let links = extract_links(&html, url);
    
    // 2. Convert HTML to Markdown
    let markdown = parser(html.clone());
    
    // 3. Extract metadata
    let metadata = extract_metadata(&html);
    
    // 4. Create document
    Document::new(url, markdown, links)
        .with_title(metadata.title.unwrap_or_else(|| "Untitled".to_string()))
        .with_description(metadata.description)
        .with_raw_html(html)
}
```

### Example 5: Saving and Loading

```rust
use spiderman::core::document::Document;
use std::fs;

// Create document
let doc = Document::new(
    "http://example.com",
    "# Content".to_string(),
    vec![]
)
.with_title("Example".to_string());

// Save to JSON file
let json = doc.to_json_pretty().unwrap();
fs::write("document.json", json).unwrap();

// Load from JSON file
let loaded_json = fs::read_to_string("document.json").unwrap();
let loaded_doc = Document::from_json(&loaded_json).unwrap();

assert_eq!(loaded_doc.url(), "http://example.com");
```

---

## Serialization

### JSON Format

Documents are serialized to JSON for storage and transmission.

#### Example JSON Output

```json
{
  "url": "http://example.com",
  "title": "Example Domain",
  "description": "This domain is for examples",
  "content": "# Example Domain\n\nThis domain is for use in examples...",
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

#### Optional Fields

Fields that are `None` or empty are omitted from JSON:

```json
{
  "url": "http://example.com",
  "title": "Example",
  "content": "...",
  "links": [],
  "crawled_at": "2024-01-15T10:30:00Z"
}
```
(No `description`, `raw_html`, or `metadata` fields)

### Serialization Methods

#### `to_json() -> Result<String>`
Compact JSON (one line, no formatting)

```rust
let doc = Document::new("http://example.com", "content".to_string(), vec![]);
let json = doc.to_json().unwrap();
// Output: {"url":"http://example.com",...}
```

#### `to_json_pretty() -> Result<String>`
Pretty-printed JSON (multi-line, indented)

```rust
let doc = Document::new("http://example.com", "content".to_string(), vec![]);
let json = doc.to_json_pretty().unwrap();
// Output:
// {
//   "url": "http://example.com",
//   ...
// }
```

#### `from_json(json: &str) -> Result<Document>`
Parse JSON back into a Document

```rust
let json = r#"{"url":"http://example.com","title":"","content":"test","links":[],"crawled_at":"2024-01-01T00:00:00Z"}"#;
let doc = Document::from_json(json).unwrap();
```

### JSONL Format (Recommended for Storage)

For storing many documents, use JSONL (JSON Lines) format:
- One JSON document per line
- Easy to append
- Stream-processable

```
{"url":"http://example.com/page1",...}
{"url":"http://example.com/page2",...}
{"url":"http://example.com/page3",...}
```

Example:
```rust
use std::fs::OpenOptions;
use std::io::Write;

let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("crawled_documents.jsonl")
    .unwrap();

for doc in documents {
    let json = doc.to_json().unwrap();
    writeln!(file, "{}", json).unwrap();
}
```

---

## API Reference

### Constructors

#### `Document::new(url: &str, content: String, links: Vec<String>) -> Self`
Creates a new document with current timestamp.

**Parameters:**
- `url`: The page URL
- `content`: Markdown content
- `links`: List of outbound links

**Returns:** New `Document` instance

---

### Builder Methods (Chainable)

#### `with_title(self, title: String) -> Self`
Sets the title.

#### `with_description(self, description: Option<String>) -> Self`
Sets the description.

#### `with_raw_html(self, html: String) -> Self`
Stores the original HTML.

#### `with_metadata(self, key: &str, value: &str) -> Self`
Adds a metadata key-value pair.

#### `with_timestamp(self, timestamp: DateTime<Utc>) -> Self`
Sets the crawled timestamp (useful for testing).

---

### Getters

#### `url(&self) -> &str`
Returns the URL.

#### `title(&self) -> &str`
Returns the title.

#### `description(&self) -> Option<&str>`
Returns the description if present.

#### `content(&self) -> &str`
Returns the content (Markdown).

#### `raw_html(&self) -> Option<&str>`
Returns the raw HTML if stored.

#### `links(&self) -> &[String]`
Returns the list of links.

#### `crawled_at(&self) -> DateTime<Utc>`
Returns the crawled timestamp.

#### `metadata(&self) -> &HashMap<String, String>`
Returns the entire metadata map.

#### `get_metadata(&self, key: &str) -> Option<&str>`
Returns a specific metadata value.

#### `link_count(&self) -> usize`
Returns the number of links.

#### `content_length(&self) -> usize`
Returns the content length in bytes.

---

### Serialization Methods

#### `to_json(&self) -> Result<String, serde_json::Error>`
Converts to compact JSON string.

#### `to_json_pretty(&self) -> Result<String, serde_json::Error>`
Converts to pretty-printed JSON string.

#### `from_json(json: &str) -> Result<Self, serde_json::Error>`
Creates a Document from JSON string.

---

### Metadata Extraction Functions

#### `extract_metadata(html: &str) -> Metadata`
Extracts all metadata from HTML.

**Returns:** `Metadata` struct with extracted values

**Example:**
```rust
let html = r#"<html><head><title>Test</title></head></html>"#;
let metadata = extract_metadata(html);
assert_eq!(metadata.title, Some("Test".to_string()));
```

---

## Testing

The Document module has **21 comprehensive tests** covering:

### Test Categories

1. **Document Creation** (3 tests)
   - Basic creation
   - Builder pattern
   - With raw HTML

2. **Getters** (2 tests)
   - All getter methods
   - Metadata access

3. **Serialization** (3 tests)
   - to_json()
   - from_json()
   - to_json_pretty()

4. **Metadata Extraction** (8 tests)
   - Title extraction
   - Complete metadata
   - Partial metadata
   - Custom meta tags
   - HTML entity decoding
   - Attribute extraction

5. **Edge Cases** (5 tests)
   - Empty links
   - Many links
   - Empty HTML
   - Timestamp handling

### Running Tests

```bash
# Run all document tests
cargo test document::tests

# Run specific test
cargo test document::tests::test_document_builder_pattern

# Run with output
cargo test document::tests -- --nocapture
```

### Test Coverage

✅ **100% of public API tested**
✅ **Edge cases covered**
✅ **Serialization/deserialization verified**
✅ **Metadata extraction validated**

---

## Integration with Other Modules

### With HTML to MD Parser

```rust
use spiderman::core::html_to_md::parser;
use spiderman::core::document::Document;

let html = "<html><body><h1>Title</h1></body></html>";
let markdown = parser(html.to_string());
let doc = Document::new("http://example.com", markdown, vec![]);
```

### With Link Extractor

```rust
use spiderman::core::link_extractor::extract_links;
use spiderman::core::document::Document;

let html = r#"<a href="/page1">Link 1</a>"#;
let links = extract_links(html, "http://example.com");
let doc = Document::new("http://example.com", "content".to_string(), links);
```

### With URL Manager

```rust
use spiderman::core::url_manager::UrlManager;
use spiderman::core::document::Document;

let mut manager = UrlManager::new("http://example.com");
let mut documents = Vec::new();

while let Some(url) = manager.get_next() {
    // Fetch and process...
    let doc = Document::new(&url, "content".to_string(), vec![]);
    documents.push(doc);
}
```

---

## Best Practices

### 1. Always Extract Metadata

```rust
// Good
let metadata = extract_metadata(&html);
let doc = Document::new(url, content, links)
    .with_title(metadata.title.unwrap_or_default())
    .with_description(metadata.description);

// Bad
let doc = Document::new(url, content, links);
// Missing title and description
```

### 2. Use Builder Pattern for Clarity

```rust
// Good
let doc = Document::new(url, content, links)
    .with_title(title)
    .with_description(desc)
    .with_metadata("author", author);

// Less clear
let mut doc = Document::new(url, content, links);
// Separate mutations
```

### 3. Store Raw HTML When Needed

```rust
// For debugging or reprocessing
let doc = Document::new(url, content, links)
    .with_raw_html(html);
```

### 4. Use JSONL for Batch Storage

```rust
// One document per line
for doc in documents {
    writeln!(file, "{}", doc.to_json()?)?;
}
```

### 5. Handle Serialization Errors

```rust
match doc.to_json() {
    Ok(json) => save_to_file(json),
    Err(e) => eprintln!("Serialization error: {}", e),
}
```

---

## Summary

### What the Document Module Provides

✅ **Structured Data Model** - Clean representation of crawled pages
✅ **Metadata Extraction** - Automatic extraction from HTML
✅ **JSON Serialization** - Easy storage and transmission
✅ **Builder Pattern** - Flexible, readable construction
✅ **Type Safety** - Compile-time guarantees
✅ **Well-Tested** - 21 comprehensive tests
✅ **Well-Documented** - Examples for every use case

### What You Can Do With It

- Feed data to full-text search engines
- Store crawled pages in databases
- Analyze crawl results
- Export data in standard formats
- Track metadata for ranking
- Monitor crawl timestamps

### Next Steps

Now that you have the Document module, you can:

1. **Integrate it into `crawl()`** - Use it in your main crawl function
2. **Create export system** - Save documents to files/database
3. **Build search indexer** - Feed documents to your search engine

The Document module is **production-ready** and forms the foundation for structured web crawling! 🕷️📄