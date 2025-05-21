//! Surfing is a Rust library for parsing JSON from text streams.
//!
//! This library allows you to extract JSON objects and arrays from mixed text content,
//! making it ideal for processing logs, console output, or any text that may contain
//! embedded JSON structures.
//!
//! # Features
//!
//! - **Core functionality**: Extract JSON from mixed text content
//! - **Streaming support**: Process data in chunks
//! - **Utility functions**: Convenient high-level API
//! - **Serde integration**: Deserialize JSON directly from mixed text (requires the `serde` feature)
//!
//! # Examples
//!
//! Using the low-level API with a writer:
//!
//! ```
//! use std::io::BufWriter;
//! use surfing::JSONParser;
//!
//! // Create a parser
//! let mut parser = JSONParser::new();
//! let mut buffer = Vec::new();
//!
//! // Extract JSON from mixed content
//! {
//!     let mut writer = BufWriter::new(&mut buffer);
//!     parser.extract_json_from_stream(&mut writer, "Log message: {\"level\":\"info\",\"msg\":\"started\"} more text").unwrap();
//! }
//!
//! // Get the result (only the JSON part)
//! let json = String::from_utf8(buffer).unwrap();
//! assert_eq!(json, "{\"level\":\"info\",\"msg\":\"started\"}");
//! ```
//!
//! Using the convenient utility function:
//!
//! ```
//! use surfing::extract_json_to_string;
//!
//! // Extract JSON directly to a string
//! let input = "Log message: {\"level\":\"info\",\"msg\":\"started\"} more text";
//! let json = extract_json_to_string(input).unwrap();
//! assert_eq!(json, "{\"level\":\"info\",\"msg\":\"started\"}");
//! ```
//!
//! # Serde Integration
//!
//! When enabled with the `serde` feature, you can deserialize directly from mixed text:
//!
//! ```
//! # #[cfg(feature = "serde")]
//! # {
//! use serde::Deserialize;
//! use surfing::serde::from_mixed_text;
//!
//! #[derive(Debug, Deserialize)]
//! struct LogEntry {
//!     level: String,
//!     msg: String,
//! }
//!
//! let input = "Log message: {\"level\":\"info\",\"msg\":\"started\"} more text";
//! let entry: LogEntry = from_mixed_text(input).unwrap();
//! assert_eq!(entry.level, "info");
//! assert_eq!(entry.msg, "started");
//! # }
//! ```
//!
//! # Streaming Support
//!
//! The parser supports incrementally processing streams of data, maintaining state
//! between calls to handle partial JSON objects:
//!
//! ```
//! use std::io::BufWriter;
//! use surfing::JSONParser;
//!
//! let mut parser = JSONParser::new();
//! let mut buffer = Vec::new();
//!
//! {
//!     let mut writer = BufWriter::new(&mut buffer);
//!     // First part of a JSON object
//!     parser.extract_json_from_stream(&mut writer, "{\"partial").unwrap();
//!     // Rest of the JSON object
//!     parser.extract_json_from_stream(&mut writer, "\":true}").unwrap();
//! }
//!
//! let json = String::from_utf8(buffer).unwrap();
//! assert_eq!(json, "{\"partial\":true}");
//! ```

pub mod constants;
pub mod parser;
pub mod utils;

#[cfg(feature = "serde")]
pub mod serde;

// Re-export the main types and functions for convenience
pub use parser::json_parser::JSONParser;
pub use utils::string_extract::extract_json_to_string;