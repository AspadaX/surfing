//! Deserializer implementation for Serde integration.

use std::io::Cursor;

#[cfg(feature = "serde")]
use {
    serde::de::DeserializeOwned,
    serde_json::Error as SerdeJsonError,
};

use crate::JSONParser;
use crate::utils::extract_json_to_string;

/// Error type for deserialization failures.
#[derive(Debug)]
#[cfg(feature = "serde")]
pub enum DeserializeError {
    /// Error extracting JSON from text
    Extraction(String),
    /// Error deserializing the extracted JSON
    Deserialization(SerdeJsonError),
}

#[cfg(feature = "serde")]
impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::Extraction(e) => write!(f, "JSON extraction error: {}", e),
            DeserializeError::Deserialization(e) => write!(f, "JSON deserialization error: {}", e),
        }
    }
}

#[cfg(feature = "serde")]
impl std::error::Error for DeserializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DeserializeError::Extraction(_) => None,
            DeserializeError::Deserialization(e) => Some(e),
        }
    }
}

/// Deserializes a value from mixed text containing JSON.
///
/// This function extracts JSON from the given text and then deserializes
/// it into the specified type using Serde.
///
/// # Arguments
///
/// * `input` - A string slice containing mixed text with embedded JSON.
///
/// # Returns
///
/// * `Ok(T)` - The successfully deserialized value.
/// * `Err(DeserializeError)` - If extraction or deserialization fails.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "serde")]
/// # {
/// use serde::Deserialize;
/// use surfing::serde::from_mixed_text;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct User {
///     id: u64,
///     name: String,
///     active: bool,
/// }
///
/// let text = "User info: {\"id\":42,\"name\":\"Alice\",\"active\":true} (created today)";
/// let user: User = from_mixed_text(text).unwrap();
///
/// assert_eq!(user.id, 42);
/// assert_eq!(user.name, "Alice");
/// assert_eq!(user.active, true);
/// # }
/// ```
#[cfg(feature = "serde")]
pub fn from_mixed_text<T>(input: &str) -> Result<T, DeserializeError>
where
    T: DeserializeOwned,
{
    // First, extract the JSON from the mixed text
    let json = match extract_json_to_string(input) {
        Ok(json) => json,
        Err(e) => return Err(DeserializeError::Extraction(e.to_string())),
    };

    // Then deserialize it using serde
    serde_json::from_str(&json).map_err(DeserializeError::Deserialization)
}

/// Deserializes a value from mixed text using an existing JSONParser.
///
/// This function allows you to reuse a parser instance, which is useful
/// when processing multiple chunks of data or when you need to maintain
/// parser state across calls.
///
/// # Arguments
///
/// * `parser` - A mutable reference to a JSONParser instance.
/// * `input` - A string slice containing mixed text with embedded JSON.
///
/// # Returns
///
/// * `Ok(T)` - The successfully deserialized value.
/// * `Err(DeserializeError)` - If extraction or deserialization fails.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "serde")]
/// # {
/// use serde::Deserialize;
/// use surfing::JSONParser;
/// use surfing::serde::from_mixed_text_with_parser;
/// use std::io::Cursor;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Metrics {
///     cpu: f64,
///     memory: u64,
/// }
///
/// let mut parser = JSONParser::new();
///
/// // Process first chunk
/// let text1 = "System metrics: {\"cpu\":";
/// let text2 = "3.5,\"memory\":1024}";
///
/// // Process the first chunk (this will be partial JSON)
/// {
///     let mut buffer = Vec::new();
///     let mut writer = Cursor::new(&mut buffer);
///     parser.extract_json_from_stream(&mut writer, text1).unwrap();
/// }
///
/// // We can check if we're in the middle of parsing JSON
/// assert!(parser.is_in_json());
///
/// // Process the second chunk and deserialize the complete JSON
/// let metrics: Metrics = from_mixed_text_with_parser(&mut parser, text2).unwrap();
///
/// assert_eq!(metrics.cpu, 3.5);
/// assert_eq!(metrics.memory, 1024);
/// # }
/// ```
#[cfg(feature = "serde")]
pub fn from_mixed_text_with_parser<T>(
    parser: &mut JSONParser,
    input: &str,
) -> Result<T, DeserializeError>
where
    T: DeserializeOwned,
{
    // Set up a buffer to collect the JSON
    let mut buffer = Vec::new();
    
    // Extract JSON from the input text
    {
        let mut writer = Cursor::new(&mut buffer);
        if let Err(e) = parser.extract_json_from_stream(&mut writer, input) {
            return Err(DeserializeError::Extraction(e.to_string()));
        }
    }
    
    // Convert buffer to string
    let json = String::from_utf8(buffer)
        .map_err(|e| DeserializeError::Extraction(e.to_string()))?;
    
    // Get any previously extracted JSON that might still be in the buffer
    // If we received empty input but parser has finished JSON processing, use whatever is in the buffer
    if json.is_empty() && !parser.is_in_json() {
        // We need to get the complete JSON from the parser's internal state
        let mut buffer = Vec::new();
        {
            let mut writer = Cursor::new(&mut buffer);
            // Write an empty string to trigger the buffer flush
            if let Err(e) = parser.extract_json_from_stream(&mut writer, "") {
                return Err(DeserializeError::Extraction(e.to_string()));
            }
        }
        
        let complete_json = String::from_utf8(buffer)
            .map_err(|e| DeserializeError::Extraction(e.to_string()))?;
            
        if !complete_json.is_empty() {
            return serde_json::from_str(&complete_json).map_err(DeserializeError::Deserialization);
        }
    }
    
    // Normal case: attempt deserialization if we have a complete JSON object
    if !parser.is_in_json() && !json.is_empty() {
        serde_json::from_str(&json).map_err(DeserializeError::Deserialization)
    } else {
        // Return an error if we don't have complete JSON
        Err(DeserializeError::Extraction(
            "Incomplete JSON: parser is still expecting more input".to_string()
        ))
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Deserialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct NestedStruct {
        id: u64,
        data: TestStruct,
    }

    #[test]
    fn test_deserialize_simple_struct() {
        let input = "Text before {\"name\":\"test\",\"value\":42} text after";
        let result: TestStruct = from_mixed_text(input).unwrap();
        
        assert_eq!(
            result,
            TestStruct {
                name: "test".to_string(),
                value: 42,
            }
        );
    }

    #[test]
    fn test_deserialize_nested_struct() {
        let input = "Data: {\"id\":123,\"data\":{\"name\":\"nested\",\"value\":99}}";
        let result: NestedStruct = from_mixed_text(input).unwrap();
        
        assert_eq!(
            result,
            NestedStruct {
                id: 123,
                data: TestStruct {
                    name: "nested".to_string(),
                    value: 99,
                },
            }
        );
    }

    #[test]
    fn test_deserialize_with_parser() {
        // Create a simple parser for complete JSON
        let mut parser = JSONParser::new();
        
        let json = "{\"name\":\"parser_test\",\"value\":42}";
        let result: TestStruct = from_mixed_text_with_parser(&mut parser, json).unwrap();
        
        assert_eq!(
            result,
            TestStruct {
                name: "parser_test".to_string(),
                value: 42,
            }
        );
    }

    #[test]
    fn test_error_on_invalid_json() {
        let input = "Invalid: {\"name\":\"test\",\"value\":\"not a number\"}";
        let result: Result<TestStruct, _> = from_mixed_text(input);
        
        assert!(result.is_err());
        if let Err(DeserializeError::Deserialization(_)) = result {
            // Expected error type
        } else {
            panic!("Expected deserialization error");
        }
    }

    #[test]
    fn test_error_on_incomplete_json() {
        let mut parser = JSONParser::new();
        let input = "{\"name\":\"incomplete\"";
        
        let result: Result<TestStruct, _> = from_mixed_text_with_parser(&mut parser, input);
        assert!(result.is_err());
        
        // Make sure the parser is still in-progress
        assert!(parser.is_in_json());
    }
}