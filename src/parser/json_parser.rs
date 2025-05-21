//! JSON Parser module for extracting JSON from text streams.

use std::io::Write;

use crate::constants::PAIRED_MARKERS;
use crate::parser::marker::Marker;

/// A parser that extracts JSON objects and arrays from a stream of text.
///
/// `JSONParser` can process text that contains both JSON and non-JSON content,
/// and will extract only the JSON parts. It handles both complete and partial 
/// JSON documents, allowing for incremental parsing of streaming data.
///
/// # Examples
///
/// ```
/// use std::io::BufWriter;
/// use surfing::parser::json_parser::JSONParser;
///
/// // Create a new parser
/// let mut parser = JSONParser::new();
/// let mut buffer = Vec::new();
/// 
/// // Process text with embedded JSON
/// {
///     let mut writer = BufWriter::new(&mut buffer);
///     
///     // Process text with JSON content
///     parser.extract_json_from_stream(&mut writer, "Text before {\"id\": 123} text after").unwrap();
/// }
///
/// // Get the extracted JSON
/// let json_only = String::from_utf8(buffer).unwrap();
/// assert_eq!(json_only, "{\"id\": 123}");
/// ```
pub struct JSONParser {
    buffer: String,
    markers: Vec<Marker>,
}

impl JSONParser {
    /// Creates a new JSONParser instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use surfing::parser::json_parser::JSONParser;
    ///
    /// let parser = JSONParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            markers: Vec::new(),
        }
    }
    
    /// Checks if the parser is currently processing a JSON structure.
    ///
    /// This method returns `true` when the parser is in the middle of processing
    /// a JSON object or array and is expecting more input to complete it.
    ///
    /// # Returns
    ///
    /// * `true` - If the parser is currently inside a JSON object or array.
    /// * `false` - If the parser is not inside a JSON structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use surfing::JSONParser;
    ///
    /// let mut parser = JSONParser::new();
    /// assert!(!parser.is_in_json()); // Not parsing anything yet
    ///
    /// // Process partial JSON
    /// let mut buffer = Vec::new();
    /// parser.extract_json_from_stream(&mut buffer, "{\"partial").unwrap();
    /// assert!(parser.is_in_json()); // Still expecting more JSON
    ///
    /// // Complete the JSON
    /// parser.extract_json_from_stream(&mut buffer, "\":true}").unwrap();
    /// assert!(!parser.is_in_json()); // JSON is complete
    /// ```
    pub fn is_in_json(&self) -> bool {
        !self.markers.is_empty()
    }
    
    /// Removes the marker pair when a closing marker is found.
    ///
    /// # Arguments
    ///
    /// * `item` - The character to check as a potential closing marker.
    fn remove_markers_pair(&mut self, item: &char) {
        // Create reversed markers for finding the ending marker
        let mut markers_to_reverse: Vec<Marker> = self.markers.clone();
        markers_to_reverse.reverse();
        
        // Look for a start marker
        for marker in markers_to_reverse.iter() {
            // If we find a start marker, we remove the marker from the buffer
            if marker.is_counter_part(item) {
                self.markers.pop();
                return;
            } 
        }
    }
    
    /// Updates the internal markers state based on the current character.
    ///
    /// # Arguments
    ///
    /// * `item` - The character to process.
    fn update_markers(&mut self, item: &char) {
        // Store the valid start marker. 
        // We only check the end marker.
        if let Some(marker) = Marker::new(item) {
            self.markers.push(marker);
            return;
        }
        
        self.remove_markers_pair(item);
        
        // If we have no markers left, return
        if self.markers.is_empty() { 
            self.buffer.clear();
            return;
        }
    }
    
    /// Extracts JSON content from a string and writes it to the provided writer.
    ///
    /// This method processes each character in the input string and:
    /// - Extracts only valid JSON objects and arrays
    /// - Ignores text outside of JSON structures
    /// - Maintains state across multiple calls for incremental processing
    ///
    /// # Arguments
    ///
    /// * `writer` - A mutable reference to an object implementing the `Write` trait.
    /// * `json_object` - The string slice to process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If processing completed successfully.
    /// * `Err(Error)` - If there was an error writing to the writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{BufWriter, stdout, StdoutLock};
    /// use surfing::parser::json_parser::JSONParser;
    ///
    /// // Example with BufWriter
    /// let mut parser = JSONParser::new();
    /// let mut buffer = Vec::new();
    /// {
    ///     let mut writer = BufWriter::new(&mut buffer);
    ///     parser.extract_json_from_stream(&mut writer, "Text {\"id\": 123}").unwrap();
    /// }
    ///
    /// // Example with StdoutLock
    /// let mut parser = JSONParser::new();
    /// let stdout = stdout();
    /// {
    ///     let mut handle = stdout.lock();
    ///     // In a real program, you would handle the result
    ///     let _ = parser.extract_json_from_stream(&mut handle, "{\"key\": \"value\"}");
    /// }
    /// ```
    pub fn extract_json_from_stream<W: Write>(&mut self, writer: &mut W, json_object: &str) -> Result<(), Box<dyn std::error::Error>> {
        for item in json_object.chars() {
            if self.is_in_json() {
                self.buffer.push(item);
                self.update_markers(&item);
                write!(writer, "{}", item)?;
                continue;
            }

            if PAIRED_MARKERS.contains(&item) {
                self.buffer.push(item);
                self.update_markers(&item);
                write!(writer, "{}", item)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;

    #[test]
    fn test_json_parser_empty() {
        let parser = JSONParser::new();
        assert!(!parser.is_in_json());
        assert!(parser.markers.is_empty());
    }

    #[test]
    fn test_json_parser_extract_simple_json() {
        let mut parser = JSONParser::new();
        let mut buffer = Vec::new();

        {
            let mut writer = BufWriter::new(&mut buffer);
            parser.extract_json_from_stream(&mut writer, "{}").unwrap();
            // Flush the writer to ensure all data is written to the underlying buffer
            writer.flush().unwrap();
        }

        assert!(!parser.is_in_json());

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "{}");
    }

    #[test]
    fn test_json_parser_extract_nested_json() {
        let mut parser = JSONParser::new();
        let mut buffer = Vec::new();

        {
            let mut writer = BufWriter::new(&mut buffer);
            parser.extract_json_from_stream(&mut writer, "{\"key\": [1, 2, 3]}").unwrap();
            assert!(!parser.is_in_json());
        }

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "{\"key\": [1, 2, 3]}");
    }

    #[test]
    fn test_json_parser_extract_partial_json() {
        let mut parser = JSONParser::new();
        let mut buffer = Vec::new();

        {
            let mut writer = BufWriter::new(&mut buffer);
            parser.extract_json_from_stream(&mut writer, "{\"key").unwrap();
            assert!(parser.is_in_json());
    
            parser.extract_json_from_stream(&mut writer, "\": [1, 2, 3]}").unwrap();
            assert!(!parser.is_in_json());
        }

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "{\"key\": [1, 2, 3]}");
    }
    
    #[test]
    fn test_json_parser_extract_json_mixed_with_text() {
        let mut parser = JSONParser::new();
        let mut buffer = Vec::new();

        {
            let mut writer = BufWriter::new(&mut buffer);

            // Plain text followed by JSON
            parser.extract_json_from_stream(&mut writer, "Some plain text {\"id\": 123, \"data\": ").unwrap();
            assert!(parser.is_in_json());

            // More JSON with nested structure
            parser.extract_json_from_stream(&mut writer, "{\"nested\": [1, 2, {\"deep\": true}]}}").unwrap();
            assert!(!parser.is_in_json());

            // JSON followed by plain text
            parser.extract_json_from_stream(&mut writer, " followed by more text").unwrap();
            assert!(!parser.is_in_json());

            // Another JSON object
            parser.extract_json_from_stream(&mut writer, " and another {\"array\": [4, 5, 6]}").unwrap();
            assert!(!parser.is_in_json());
        }

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, 
            "{\"id\": 123, \"data\": {\"nested\": [1, 2, {\"deep\": true}]}}{\"array\": [4, 5, 6]}");
    }
}