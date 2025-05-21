//! Basic example of extracting JSON from mixed text
//!
//! This example demonstrates how to extract JSON objects from text
//! that contains both JSON and non-JSON content.

use std::io::BufWriter;
use surfing::JSONParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new parser
    let mut parser = JSONParser::new();
    
    // Input text containing both JSON and plain text
    let input = r#"
    System starting at 2023-06-15T14:30:00Z
    Configuration loaded: {"debug":true,"port":8080,"timeout":30}
    Log level set to INFO
    Server listening on {"address":"0.0.0.0","port":8080}
    "#;
    
    // Set up a buffer to store the extracted JSON
    let mut buffer = Vec::new();
    
    // Extract only the JSON content
    {
        let mut writer = BufWriter::new(&mut buffer);
        parser.extract_json_from_stream(&mut writer, input)?;
    }
    
    // Convert the buffer to a string
    let json_only = String::from_utf8(buffer)?;
    
    // Print the original input
    println!("Original input:\n{}", input);
    
    // Print the extracted JSON
    println!("\nExtracted JSON:\n{}", json_only);
    
    // The result should be just the JSON parts concatenated:
    // {"debug":true,"port":8080,"timeout":30}{"address":"0.0.0.0","port":8080}
    
    Ok(())
}