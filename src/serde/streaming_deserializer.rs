//! Streaming deserializer for parsing JSON objects from text streams.
//!
//! This module provides the `StreamingDeserializer` which can process
//! chunks of text containing mixed content, extracting and deserializing
//! JSON objects as they become available.

use std::io::Cursor;
use std::marker::PhantomData;

use anyhow::Error;
use serde::de::DeserializeOwned;

use crate::JSONParser;
use crate::serde::deserializer::DeserializeError;

/// A deserializer for processing streams of text containing JSON.
///
/// The `StreamingDeserializer` can process chunks of text data incrementally,
/// extracting JSON objects and deserializing them into Rust types.
/// It automatically handles partial JSON objects split across multiple chunks.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "serde")]
/// # {
/// use serde::Deserialize;
/// use surfing::serde::StreamingDeserializer;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// // Create a deserializer for the User type
/// let mut deserializer = StreamingDeserializer::<User>::new();
///
/// // Process stream in chunks
/// let chunks = [
///     "Text before {\"id\":",
///     "42,\"name\":\"Alice",
///     "\"} more text"
/// ];
///
/// // Add each chunk and check for completed objects
/// for chunk in &chunks[0..2] {
///     let result = deserializer.process_chunk(chunk);
///     assert!(result.is_none()); // Not complete yet
/// }
///
/// // The final chunk should complete the JSON object
/// let result = deserializer.process_chunk(chunks[2]);
/// assert!(result.is_some());
///
/// let user = result.unwrap();
/// assert_eq!(user.id, 42);
/// assert_eq!(user.name, "Alice");
/// # }
/// ```
pub struct StreamingDeserializer<T>
where
    T: DeserializeOwned,
{
    parser: JSONParser,
    accumulated_json: String,
    _phantom: PhantomData<T>,
}

impl<T> StreamingDeserializer<T>
where
    T: DeserializeOwned,
{
    /// Creates a new streaming deserializer.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use serde::Deserialize;
    /// use surfing::serde::StreamingDeserializer;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct LogEntry {
    ///     level: String,
    ///     message: String,
    /// }
    ///
    /// let deserializer = StreamingDeserializer::<LogEntry>::new();
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            parser: JSONParser::new(),
            accumulated_json: String::new(),
            _phantom: PhantomData,
        }
    }

    /// Process a chunk of text data and attempt to extract and deserialize JSON.
    ///
    /// This method processes the given chunk of text, extracting any JSON content
    /// and accumulating it. If a complete JSON object is found, it deserializes
    /// it into the target type `T`.
    ///
    /// # Arguments
    ///
    /// * `chunk` - A string slice containing text data, potentially with embedded JSON.
    ///
    /// # Returns
    ///
    /// * `Some(T)` - If a complete JSON object was found and successfully deserialized.
    /// * `None` - If the JSON is still incomplete or no JSON was found.
    ///
    /// # Errors
    ///
    /// The method will return `None` if:
    /// - The chunk contains no JSON
    /// - The JSON object is still incomplete
    /// - There was an error deserializing the JSON
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use serde::Deserialize;
    /// use surfing::serde::StreamingDeserializer;
    ///
    /// #[derive(Debug, Deserialize, PartialEq)]
    /// struct Config {
    ///     debug: bool,
    ///     port: u16,
    /// }
    ///
    /// let mut deserializer = StreamingDeserializer::<Config>::new();
    ///
    /// // Process incomplete JSON
    /// let result = deserializer.process_chunk("{\"debug\":true,");
    /// assert!(result.is_none()); // Still incomplete
    ///
    /// // Complete the JSON
    /// let result = deserializer.process_chunk("\"port\":8080}");
    /// assert!(result.is_some());
    ///
    /// let config = result.unwrap();
    /// assert_eq!(config.debug, true);
    /// assert_eq!(config.port, 8080);
    /// # }
    /// ```
    pub fn process_chunk(&mut self, chunk: &str) -> Option<T> {
        // Extract JSON from this chunk
        let mut buffer = Vec::new();
        {
            let mut writer = Cursor::new(&mut buffer);
            if self.parser.extract_json_from_stream(&mut writer, chunk).is_err() {
                return None;
            }
        }

        // Add this chunk's extracted JSON to our accumulation
        if let Ok(chunk_json) = String::from_utf8(buffer) {
            self.accumulated_json.push_str(&chunk_json);
        } else {
            return None;
        }

        // If we've completed a JSON object, try to deserialize it
        if !self.parser.is_in_json() && !self.accumulated_json.is_empty() {
            let accumulated_json = self.accumulated_json.clone();
            // Reset the accumulated JSON for the next object
            self.accumulated_json.clear();
            
            match serde_json::from_str::<T>(&accumulated_json) {
                Ok(value) => {
                    Some(value)
                }
                Err(_) => {
                    None
                },
            }
        } else {
            // Still waiting for more JSON
            None
        }
    }

    /// Returns whether the parser is currently in the middle of processing a JSON object.
    ///
    /// # Returns
    ///
    /// * `true` - If the parser is currently processing an incomplete JSON object.
    /// * `false` - If the parser is not in the middle of a JSON object.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use serde::Deserialize;
    /// use surfing::serde::StreamingDeserializer;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Data {
    ///     value: i32,
    /// }
    ///
    /// let mut deserializer = StreamingDeserializer::<Data>::new();
    /// assert!(!deserializer.is_in_json()); // Not processing anything yet
    ///
    /// deserializer.process_chunk("{\"value\":");
    /// assert!(deserializer.is_in_json()); // In the middle of JSON
    ///
    /// deserializer.process_chunk("42}");
    /// assert!(!deserializer.is_in_json()); // JSON complete
    /// # }
    /// ```
    pub fn is_in_json(&self) -> bool {
        self.parser.is_in_json()
    }

    /// Returns the currently accumulated partial JSON string.
    ///
    /// This can be useful for debugging or logging purposes.
    ///
    /// # Returns
    ///
    /// A string slice containing the currently accumulated partial JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use serde::Deserialize;
    /// use surfing::serde::StreamingDeserializer;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Data {
    ///     value: i32,
    /// }
    ///
    /// let mut deserializer = StreamingDeserializer::<Data>::new();
    /// deserializer.process_chunk("{\"value\":");
    ///
    /// assert_eq!(deserializer.accumulated_json(), "{\"value\":");
    /// # }
    /// ```
    pub fn accumulated_json(&self) -> &str {
        &self.accumulated_json
    }

    /// Resets the deserializer state.
    ///
    /// This clears any accumulated JSON and resets the parser,
    /// allowing you to start processing a new stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use serde::Deserialize;
    /// use surfing::serde::StreamingDeserializer;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Data {
    ///     value: i32,
    /// }
    ///
    /// let mut deserializer = StreamingDeserializer::<Data>::new();
    /// deserializer.process_chunk("{\"value\":");  // Partial JSON
    ///
    /// // Reset the deserializer
    /// deserializer.reset();
    /// assert!(!deserializer.is_in_json());
    /// assert!(deserializer.accumulated_json().is_empty());
    /// # }
    /// ```
    pub fn reset(&mut self) {
        self.parser = JSONParser::new();
        self.accumulated_json.clear();
    }

    /// Attempts to finalize and deserialize any accumulated JSON.
    ///
    /// This method should be called when no more chunks are expected,
    /// to handle cases where the JSON might be valid despite the parser
    /// still expecting more input.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<T>)` - `Some(T)` if a complete object was deserialized, `None` if no valid JSON is available
    /// * `Err(DeserializeError)` - If there was an error deserializing the JSON
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use serde::Deserialize;
    /// use surfing::serde::StreamingDeserializer;
    ///
    /// #[derive(Debug, Deserialize, PartialEq)]
    /// struct Data {
    ///     value: i32,
    /// }
    ///
    /// let mut deserializer = StreamingDeserializer::<Data>::new();
    ///
    /// // The stream ends, try to finalize
    /// let result = deserializer.finalize();
    /// assert!(result.is_ok());
    /// assert!(result.unwrap().is_none()); // No JSON was accumulated
    ///
    /// // Process complete JSON
    /// deserializer.process_chunk("{\"value\":42}");
    ///
    /// // Should be able to finalize
    /// let result = deserializer.finalize();
    /// assert!(result.is_ok());
    /// assert!(result.unwrap().is_some());
    /// # }
    /// ```
    pub fn finalize(&mut self) -> Result<Option<T>, DeserializeError> {
        if self.accumulated_json.is_empty() {
            return Ok(None);
        }

        match serde_json::from_str::<T>(&self.accumulated_json) {
            Ok(value) => {
                self.reset();
                Ok(Some(value))
            }
            Err(e) => Err(DeserializeError::Deserialization(e)),
        }
    }
}

impl<T> Default for StreamingDeserializer<T>
where
    T: DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestData {
        id: u64,
        name: String,
    }

    #[test]
    fn test_complete_json_in_one_chunk() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        
        let result = deserializer.process_chunk("{\"id\":1,\"name\":\"test\"}");
        assert!(result.is_some());
        
        let data = result.unwrap();
        assert_eq!(data.id, 1);
        assert_eq!(data.name, "test");
    }

    #[test]
    fn test_partial_json_across_multiple_chunks() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        
        // First chunk - no complete JSON yet
        let result = deserializer.process_chunk("{\"id\":2,");
        assert!(result.is_none());
        
        // Second chunk - still incomplete
        let result = deserializer.process_chunk("\"name\":\"");
        assert!(result.is_none());
        
        // Third chunk - completes the JSON
        let result = deserializer.process_chunk("streaming\"}");
        assert!(result.is_some());
        
        let data = result.unwrap();
        assert_eq!(data.id, 2);
        assert_eq!(data.name, "streaming");
    }

    #[test]
    fn test_mixed_text_with_json() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        
        let result = deserializer.process_chunk("Log entry: {\"id\":3,\"name\":\"mixed\"} End of log");
        assert!(result.is_some());
        
        let data = result.unwrap();
        assert_eq!(data.id, 3);
        assert_eq!(data.name, "mixed");
    }

    #[test]
    fn test_reset_deserializer() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        
        // Start processing some JSON
        deserializer.process_chunk("{\"id\":4,");
        assert!(deserializer.is_in_json());
        
        // Reset the deserializer
        deserializer.reset();
        assert!(!deserializer.is_in_json());
        assert_eq!(deserializer.accumulated_json(), "");
        
        // Start fresh
        let result = deserializer.process_chunk("{\"id\":4,\"name\":\"reset\"}");
        assert!(result.is_some());
    }

    #[test]
    fn test_finalize_with_complete_json() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        deserializer.process_chunk("{\"id\":5,\"name\":\"finalize\"}");
        
        let result = deserializer.finalize();
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(data.is_some());
        assert_eq!(data.unwrap().name, "finalize");
    }

    #[test]
    fn test_finalize_with_incomplete_json() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        deserializer.process_chunk("{\"id\":6,\"name\":");
        
        let result = deserializer.finalize();
        assert!(result.is_err());
    }

    #[test]
    fn test_no_json_returns_none() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        
        let result = deserializer.process_chunk("This text contains no JSON objects");
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_json_objects() {
        let mut deserializer = StreamingDeserializer::<TestData>::new();
        
        // Process a chunk with two complete JSON objects
        let chunk = "{\"id\":7,\"name\":\"first\"}{\"id\":8,\"name\":\"second\"}";
        
        // Should get the first object
        let result1 = deserializer.process_chunk(chunk);
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().id, 7);
        
        // The second object should be ignored (current implementation limitation)
        // A more advanced implementation could handle this by tracking partial objects
    }
}