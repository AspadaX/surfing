//! Serde integration example
//!
//! This example demonstrates how to use the Serde integration to directly
//! deserialize structured data from mixed text content.
//!
//! Run with: cargo run --example serde_integration --features serde

use serde::Deserialize;
use surfing::serde::from_mixed_text;
use surfing::JSONParser;

// Define some data structures to deserialize into
#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: u64,
    level: String,
    msg: String,
}

#[derive(Debug, Deserialize)]
struct Metrics {
    cpu: f64,
    memory: f64,
    disk: f64,
}

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

fn main() {
    println!("=== Serde Integration Example ===\n");

    // === Basic deserialization from mixed text ===
    println!("1. Single-step deserialization");
    
    let log_text = r#"this is some mixed text{"timestamp":1686839400,"level":"info","msg":"Application started successfully"}"#;
    
    // Extract and deserialize in one step
    match from_mixed_text::<LogEntry>(log_text) {
        Ok(log_entry) => {
            println!("  Log entry: {:#?}", log_entry);
        },
        Err(e) => {
            println!("  Error: {}", e);
        }
    }
    println!();

    // === Multiple JSON objects in the same text ===
    println!("2. Extracting multiple JSON objects");
    
    let metrics_text = r#"{"cpu":12.5,"memory":45.2,"disk":78.9}"#;
    let user_text = r#"{"id":42,"name":"Alice Smith","email":"alice@example.com","active":true}"#;
    
    // Process each JSON separately
    
    // First JSON object - metrics
    println!("  First JSON object (metrics):");
    match from_mixed_text::<Metrics>(metrics_text) {
        Ok(metrics) => {
            println!("  {:#?}", metrics);
        },
        Err(e) => {
            println!("  Error: {}", e);
        }
    }
    println!();
    
    // Second JSON object - user
    println!("  Second JSON object (user):");
    match from_mixed_text::<User>(user_text) {
        Ok(user) => {
            println!("  {:#?}", user);
        },
        Err(e) => {
            println!("  Error: {}", e);
        }
    }
    println!();

    // === Streaming JSON processing ===
    println!("3. Streaming JSON processing");
    
    // Simulating a stream that arrives in multiple chunks
    let chunks = [
        "{\"id\":",
        "1001,\"name\":\"Stream",
        "ing User\",\"email\":\"streaming@example",
        ".com\",\"active\":false}"
    ];
    
    let mut parser = JSONParser::new();
    
    // Process each chunk as it arrives
    println!("  Processing chunks:");
    
    // Use a separate buffer to accumulate the complete JSON
    let mut complete_json = String::new();
    
    for (i, chunk) in chunks.iter().enumerate() {
        println!("    Chunk {}: {}", i + 1, chunk);
        
        // Process this chunk
        let mut buffer = Vec::new();
        {
            let mut writer = std::io::Cursor::new(&mut buffer);
            parser.extract_json_from_stream(&mut writer, chunk).unwrap();
        }
        
        // Add this chunk's extracted JSON to our accumulation
        let chunk_json = String::from_utf8(buffer).unwrap();
        complete_json.push_str(&chunk_json);
        
        // Check if we have completed JSON now
        println!("    Status: {}",
            if parser.is_in_json() {
                format!("Incomplete (still parsing JSON)")
            } else {
                format!("Complete (JSON parsing finished)")
            }
        );
        
        // If we've finished parsing JSON, we can try to deserialize
        if !parser.is_in_json() {
            println!("    Extracted complete JSON: {}", complete_json);
            
            match serde_json::from_str::<User>(&complete_json) {
                Ok(user) => {
                    println!("\n  Successfully deserialized user:");
                    println!("  {:#?}", user);
                },
                Err(e) => {
                    println!("    Deserialization error: {}", e);
                }
            }
            
            break;
        }
    }

    println!("\nSerde integration example completed successfully!");
}