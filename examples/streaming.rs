//! Streaming example for processing JSON in chunks
//!
//! This example demonstrates how to extract JSON from a stream
//! that arrives in multiple chunks, as might happen when reading
//! from network sources or large files.

use std::io::{BufWriter, Write};
use surfing::JSONParser;

fn main() -> Result<(), anyhow::Error> {
    // Create a new parser
    let mut parser = JSONParser::new();
    
    // Simulate chunks of data arriving from a stream
    let chunks = [
        "Server starting...\n",
        "Loading config {\"name\":\"streaming-server\",",
        "\"version\":\"1.0.0\",\"settings\":{",
        "\"timeout\":5000,\"retry\":true}}",
        "\nInitialization complete.",
        "\nReceived request: {\"method\":\"GET\",",
        "\"path\":\"/api/data\"}",
    ];
    
    // Set up a buffer to store the extracted JSON
    let mut buffer = Vec::new();
    
    // Process each chunk as it arrives
    println!("Processing {} chunks of streaming data...", chunks.len());
    
    {
        // Create a writer that we'll use within this scope
        let mut writer = BufWriter::new(&mut buffer);
        
        for (i, chunk) in chunks.iter().enumerate() {
            // In a real application, this would come from stdin, network, etc.
            println!("Chunk {}: {}", i + 1, chunk.trim());
            
            // Extract JSON from this chunk
            parser.extract_json_from_stream(&mut writer, chunk)?;
        }
        
        // Ensure everything is written to the buffer
        writer.flush()?;
    } // Writer is dropped here, releasing the borrow on buffer
    
    // Convert the buffer to a string
    let json_only = String::from_utf8(buffer)?;
    
    // Print the extracted JSON
    println!("\nExtracted JSON from stream:");
    println!("{}", json_only);
    
    // The result should be just the JSON parts concatenated:
    // {"name":"streaming-server","version":"1.0.0","settings":{"timeout":5000,"retry":true}}{"method":"GET","path":"/api/data"}
    
    Ok(())
}