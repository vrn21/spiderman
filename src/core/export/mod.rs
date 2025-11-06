//! Export Module
//!
//! This module provides functionality to export crawled documents to various formats.
//! It's responsible for persisting crawler results to the file system.
//!
//! # Overview
//!
//! The Export module supports:
//! 1. **JSONL Export** - One JSON document per line (recommended for large datasets)
//! 2. **JSON Export** - Single JSON array (good for small datasets)
//! 3. **Batch Operations** - Efficient bulk export
//! 4. **Error Handling** - Robust error reporting
//!
//! # JSONL Format
//!
//! JSONL (JSON Lines) is the recommended format because:
//! - Each line is a complete JSON document
//! - Easy to append without re-parsing entire file
//! - Stream-processable (doesn't require loading entire file)
//! - Simple error recovery (one bad line doesn't corrupt entire file)
//!
//! Example JSONL file:
//! ```text
//! {"url":"http://example.com/page1",...}
//! {"url":"http://example.com/page2",...}
//! {"url":"http://example.com/page3",...}
//! ```
//!
//! # Examples
//!
//! ## Export Single Document
//!
//! ```no_run
//! use spiderman::core::export::Exporter;
//! use spiderman::core::document::Document;
//!
//! let exporter = Exporter::new("output");
//! let doc = Document::new("http://example.com", "content".to_string(), vec![]);
//!
//! exporter.export_document(&doc, "crawl.jsonl").unwrap();
//! ```
//!
//! ## Batch Export
//!
//! ```no_run
//! use spiderman::core::export::Exporter;
//!
//! let exporter = Exporter::new("output");
//! exporter.export_batch(&documents, "crawl.jsonl").unwrap();
//! ```

use crate::core::document::Document;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Exporter for saving crawled documents to files
///
/// This struct handles exporting documents to various formats with
/// proper error handling and directory management.
///
/// # Fields
///
/// * `output_dir` - The directory where exported files will be saved
///
/// # Examples
///
/// ```no_run
/// use spiderman::core::export::Exporter;
///
/// let exporter = Exporter::new("output");
/// // or with custom path
/// let exporter = Exporter::new("/path/to/output");
/// ```
#[derive(Debug, Clone)]
pub struct Exporter {
    /// Output directory path
    output_dir: PathBuf,
}

impl Exporter {
    /// Creates a new Exporter with the specified output directory
    ///
    /// The directory will be created if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Path to the output directory
    ///
    /// # Returns
    ///
    /// A new `Exporter` instance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderman::core::export::Exporter;
    ///
    /// let exporter = Exporter::new("crawled_data");
    /// ```
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Ensures the output directory exists, creating it if necessary
    ///
    /// # Returns
    ///
    /// `Ok(())` if directory exists or was created successfully
    /// `Err` if directory creation fails
    fn ensure_output_dir(&self) -> io::Result<()> {
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)?;
        }
        Ok(())
    }

    /// Gets the full path for an output file
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the output file
    ///
    /// # Returns
    ///
    /// Full path to the output file
    fn get_output_path(&self, filename: &str) -> PathBuf {
        self.output_dir.join(filename)
    }

    /// Exports a single document to a JSONL file
    ///
    /// Appends the document as a new line to the specified file.
    /// Creates the file if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `document` - The document to export
    /// * `filename` - Name of the output file (e.g., "crawl.jsonl")
    ///
    /// # Returns
    ///
    /// `Ok(())` if export succeeds
    /// `Err` if serialization or file write fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderman::core::export::Exporter;
    /// use spiderman::core::document::Document;
    ///
    /// let exporter = Exporter::new("output");
    /// let doc = Document::new("http://example.com", "content".to_string(), vec![]);
    ///
    /// exporter.export_document(&doc, "crawl.jsonl").unwrap();
    /// ```
    pub fn export_document(&self, document: &Document, filename: &str) -> io::Result<()> {
        self.ensure_output_dir()?;

        let file_path = self.get_output_path(filename);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        let json = document
            .to_json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        writeln!(file, "{}", json)?;
        Ok(())
    }

    /// Exports multiple documents to a JSONL file in batch
    ///
    /// More efficient than calling `export_document` repeatedly
    /// because it opens the file once and writes all documents.
    ///
    /// # Arguments
    ///
    /// * `documents` - Slice of documents to export
    /// * `filename` - Name of the output file
    ///
    /// # Returns
    ///
    /// `Ok(())` if all documents exported successfully
    /// `Err` on first error encountered
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderman::core::export::Exporter;
    ///
    /// let exporter = Exporter::new("output");
    /// let documents = vec![/* ... */];
    ///
    /// exporter.export_batch(&documents, "crawl.jsonl").unwrap();
    /// ```
    pub fn export_batch(&self, documents: &[Document], filename: &str) -> io::Result<()> {
        self.ensure_output_dir()?;

        let file_path = self.get_output_path(filename);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        for doc in documents {
            let json = doc
                .to_json()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writeln!(file, "{}", json)?;
        }

        Ok(())
    }

    /// Exports documents to a single JSON array file
    ///
    /// Creates a JSON file with all documents in an array.
    /// Use this for small datasets only - large datasets should use JSONL.
    ///
    /// # Arguments
    ///
    /// * `documents` - Slice of documents to export
    /// * `filename` - Name of the output file
    ///
    /// # Returns
    ///
    /// `Ok(())` if export succeeds
    /// `Err` if serialization or file write fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderman::core::export::Exporter;
    ///
    /// let exporter = Exporter::new("output");
    /// let documents = vec![/* ... */];
    ///
    /// exporter.export_json_array(&documents, "crawl.json").unwrap();
    /// ```
    pub fn export_json_array(&self, documents: &[Document], filename: &str) -> io::Result<()> {
        self.ensure_output_dir()?;

        let file_path = self.get_output_path(filename);
        let json = serde_json::to_string_pretty(documents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(file_path, json)?;
        Ok(())
    }

    /// Returns the output directory path
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderman::core::export::Exporter;
    ///
    /// let exporter = Exporter::new("output");
    /// println!("Output directory: {:?}", exporter.output_dir());
    /// ```
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Checks if the output directory exists
    ///
    /// # Returns
    ///
    /// `true` if the directory exists, `false` otherwise
    pub fn dir_exists(&self) -> bool {
        self.output_dir.exists()
    }

    /// Clears all files in the output directory
    ///
    /// **Warning**: This deletes all files in the output directory!
    ///
    /// # Returns
    ///
    /// `Ok(())` if clearing succeeds
    /// `Err` if directory read or file deletion fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spiderman::core::export::Exporter;
    ///
    /// let exporter = Exporter::new("output");
    /// exporter.clear_output_dir().unwrap();
    /// ```
    pub fn clear_output_dir(&self) -> io::Result<()> {
        if self.output_dir.exists() {
            for entry in fs::read_dir(&self.output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }
}

/// Default exporter instance using "output" directory
impl Default for Exporter {
    fn default() -> Self {
        Self::new("output")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::Document;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_document(url: &str) -> Document {
        Document::new(url, "# Test Content".to_string(), vec![])
            .with_title("Test Title".to_string())
    }

    #[test]
    fn test_exporter_new() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        assert_eq!(exporter.output_dir(), temp_dir.path());
    }

    #[test]
    fn test_exporter_default() {
        let exporter = Exporter::default();
        assert_eq!(exporter.output_dir(), Path::new("output"));
    }

    #[test]
    fn test_ensure_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("new_dir");
        let exporter = Exporter::new(&output_path);

        assert!(!output_path.exists());
        exporter.ensure_output_dir().unwrap();
        assert!(output_path.exists());
    }

    #[test]
    fn test_export_single_document() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());
        let doc = create_test_document("http://example.com");

        exporter.export_document(&doc, "test.jsonl").unwrap();

        let file_path = temp_dir.path().join("test.jsonl");
        assert!(file_path.exists());

        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("http://example.com"));
        assert!(content.contains("Test Title"));
    }

    #[test]
    fn test_export_multiple_documents() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        let doc1 = create_test_document("http://example.com/page1");
        let doc2 = create_test_document("http://example.com/page2");

        exporter.export_document(&doc1, "test.jsonl").unwrap();
        exporter.export_document(&doc2, "test.jsonl").unwrap();

        let file_path = temp_dir.path().join("test.jsonl");
        let content = fs::read_to_string(file_path).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(content.contains("page1"));
        assert!(content.contains("page2"));
    }

    #[test]
    fn test_export_batch() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        let documents = vec![
            create_test_document("http://example.com/1"),
            create_test_document("http://example.com/2"),
            create_test_document("http://example.com/3"),
        ];

        exporter.export_batch(&documents, "batch.jsonl").unwrap();

        let file_path = temp_dir.path().join("batch.jsonl");
        let content = fs::read_to_string(file_path).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_export_json_array() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        let documents = vec![
            create_test_document("http://example.com/1"),
            create_test_document("http://example.com/2"),
        ];

        exporter
            .export_json_array(&documents, "array.json")
            .unwrap();

        let file_path = temp_dir.path().join("array.json");
        let content = fs::read_to_string(file_path).unwrap();

        assert!(content.starts_with('['));
        assert!(content.ends_with(']') || content.ends_with("]\n"));
        assert!(content.contains("http://example.com/1"));
        assert!(content.contains("http://example.com/2"));
    }

    #[test]
    fn test_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        assert!(exporter.dir_exists());

        let non_existent = temp_dir.path().join("non_existent");
        let exporter2 = Exporter::new(non_existent);
        assert!(!exporter2.dir_exists());
    }

    #[test]
    fn test_clear_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        // Create some files
        let doc = create_test_document("http://example.com");
        exporter.export_document(&doc, "file1.jsonl").unwrap();
        exporter.export_document(&doc, "file2.jsonl").unwrap();

        // Verify files exist
        assert!(temp_dir.path().join("file1.jsonl").exists());
        assert!(temp_dir.path().join("file2.jsonl").exists());

        // Clear directory
        exporter.clear_output_dir().unwrap();

        // Verify files are deleted
        assert!(!temp_dir.path().join("file1.jsonl").exists());
        assert!(!temp_dir.path().join("file2.jsonl").exists());
    }

    #[test]
    fn test_jsonl_format() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        let doc = create_test_document("http://example.com");
        exporter.export_document(&doc, "test.jsonl").unwrap();

        let file_path = temp_dir.path().join("test.jsonl");
        let content = fs::read_to_string(file_path).unwrap();

        // Verify it's valid JSON
        let line = content.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["url"], "http://example.com");
        assert_eq!(parsed["title"], "Test Title");
    }

    #[test]
    fn test_append_behavior() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = Exporter::new(temp_dir.path());

        let doc1 = create_test_document("http://example.com/1");
        let doc2 = create_test_document("http://example.com/2");

        // First export
        exporter.export_document(&doc1, "append.jsonl").unwrap();

        // Second export to same file (should append)
        exporter.export_document(&doc2, "append.jsonl").unwrap();

        let file_path = temp_dir.path().join("append.jsonl");
        let content = fs::read_to_string(file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 2);
    }
}
