//! Utility functions for extracting JSON to strings.

use std::io::Cursor;

use crate::JSONParser;

/// Extracts JSON from a string and returns the result as a String.
///
/// This is a convenience wrapper around `JSONParser::extract_json_from_stream`
/// that handles buffer management and returns a String directly.
///
/// # Arguments
///
/// * `input` - The string slice containing mixed text and JSON.
///
/// # Returns
///
/// * `Ok(String)` - The extracted JSON as a String.
/// * `Err(Error)` - If an error occurred during parsing or UTF-8 conversion.
///
/// # Examples
///
/// ```
/// use surfing::utils::extract_json_to_string;
///
/// let input = "Log message: {\"level\":\"info\",\"msg\":\"Hello\"} More text";
/// let json = extract_json_to_string(input).unwrap();
/// assert_eq!(json, "{\"level\":\"info\",\"msg\":\"Hello\"}");
/// ```
///
/// ```
/// use surfing::utils::extract_json_to_string;
///
/// // Works with multiple JSON objects
/// let input = "First: {\"id\":1} Second: {\"id\":2}";
/// let json = extract_json_to_string(input).unwrap();
/// assert_eq!(json, "{\"id\":1}{\"id\":2}");
/// ```
pub fn extract_json_to_string(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut parser = JSONParser::new();
    let mut buffer = Vec::new();
    
    {
        let mut writer = Cursor::new(&mut buffer);
        parser.extract_json_from_stream(&mut writer, input)?;
    }
    
    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_to_string_single() {
        let input = "Before {\"key\":\"value\"} After";
        let result = extract_json_to_string(input).unwrap();
        assert_eq!(result, "{\"key\":\"value\"}");
    }

    #[test]
    fn test_extract_json_to_string_multiple() {
        let input = "Start {\"a\":1}{\"b\":2} End";
        let result = extract_json_to_string(input).unwrap();
        assert_eq!(result, "{\"a\":1}{\"b\":2}");
    }

    #[test]
    fn test_extract_json_to_string_nested() {
        let input = "Data: {\"outer\":{\"inner\":true}} Text";
        let result = extract_json_to_string(input).unwrap();
        assert_eq!(result, "{\"outer\":{\"inner\":true}}");
    }

    #[test]
    fn test_extract_json_to_string_array() {
        let input = "Array: [1,2,3] End";
        let result = extract_json_to_string(input).unwrap();
        assert_eq!(result, "[1,2,3]");
    }
}