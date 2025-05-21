//! Stdout example
//!
//! This example demonstrates how to use the JSONParser with stdout,
//! which is useful for filtering JSON from a stream in real-time.

use std::io::stdout;
use surfing::JSONParser;

fn main() -> Result<(), anyhow::Error> {
    // Create a new parser
    let mut parser = JSONParser::new();
    
    // Input containing mixed text and JSON
    let input = [
        "Starting application\n",
        "Debug data: {\"timestamp\":1623766800,\"level\":\"info\",",
        "\"message\":\"Application started successfully\"}\n",
        "Processing request from 192.168.1.1\n",
        "Request payload: {\"id\":42,\"action\":\"get\",",
        "\"params\":{\"filter\":\"active\"}}\n"
    ];
    
    // Get a lock on stdout for more efficient writing
    let stdout = stdout();
    let mut handle = stdout.lock();
    
    println!("Original input (simulated stream):");
    for chunk in &input {
        print!("{}", chunk);
    }
    
    println!("\n\nExtracted JSON (sent to stdout):");
    
    // Process each chunk and write directly to stdout
    for chunk in &input {
        // In a real application, these chunks might come from stdin or a network stream
        parser.extract_json_from_stream(&mut handle, chunk)?;
    }
    
    // Add a newline at the end for better formatting
    println!("\n");
    
    // The output should contain only the JSON parts:
    // {"timestamp":1623766800,"level":"info","message":"Application started successfully"}
    // {"id":42,"action":"get","params":{"filter":"active"}}
    
    Ok(())
}