//! Serde integration for Surfing.
//!
//! This module provides integration with the Serde ecosystem, allowing 
//! direct deserialization of JSON extracted from mixed text content.
//!
//! # Feature Flag
//!
//! This module is only available when the `serde` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! surfing = { version = "0.1.0", features = ["serde"] }
//! ```
//!
//! # Single-Step Deserialization
//!
//! ```
//! # #[cfg(feature = "serde")]
//! # {
//! use serde::Deserialize;
//! use surfing::serde::from_mixed_text;
//!
//! #[derive(Debug, Deserialize, PartialEq)]
//! struct LogEntry {
//!     level: String,
//!     message: String,
//! }
//!
//! let text = "Log entry: {\"level\":\"info\",\"message\":\"Server started\"} End of line";
//! let entry: LogEntry = from_mixed_text(text).unwrap();
//!
//! assert_eq!(entry, LogEntry {
//!     level: "info".to_string(),
//!     message: "Server started".to_string(),
//! });
//! # }
//! ```
//!
//! # Streaming Deserialization
//!
//! For streaming use cases, the `StreamingDeserializer` provides a higher-level API:
//!
//! ```
//! # #[cfg(feature = "serde")]
//! # {
//! use serde::Deserialize;
//! use surfing::serde::StreamingDeserializer;
//!
//! #[derive(Debug, Deserialize, PartialEq)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//!
//! // Create a deserializer for the User type
//! let mut deserializer = StreamingDeserializer::<User>::new();
//!
//! // Process chunks as they arrive
//! let chunks = [
//!     "Text {\"id\":",
//!     "42,\"name\":\"Alice\"}",
//! ];
//!
//! let result = deserializer.process_chunk(chunks[0]);
//! assert!(result.is_none()); // Incomplete JSON
//!
//! let result = deserializer.process_chunk(chunks[1]);
//! assert!(result.is_some());
//!
//! let user = result.unwrap();
//! assert_eq!(user.id, 42);
//! # }
//! ```

mod deserializer;
mod streaming_deserializer;

#[doc(inline)]
pub use deserializer::from_mixed_text;
pub use deserializer::from_mixed_text_with_parser;
pub use deserializer::DeserializeError;
pub use streaming_deserializer::StreamingDeserializer;