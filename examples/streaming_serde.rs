//! Streaming Serde example
//!
//! This example demonstrates how to use the StreamingDeserializer to
//! process and deserialize JSON objects embedded in a stream of mixed text.
//!
//! Run with: cargo run --example streaming_serde --features serde

use serde::Deserialize;
use std::thread;
use std::time::Duration;
use surfing::serde::StreamingDeserializer;

// Define a data structure to deserialize into
#[derive(Debug, Deserialize)]
struct Message {
    id: u64,
    timestamp: u64,
    content: String,
    user: User,
}

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
}

fn main() {
    println!("=== Streaming Serde Example ===\n");

    // Create a deserializer for our Message type
    let mut deserializer = StreamingDeserializer::<Message>::new();

    // Simulate a stream of data coming in chunks
    // In a real-world scenario, this could be network data, log entries, etc.
    let stream_chunks = [
        "Log entry [2023-06-15 14:30:00]: Starting stream processing...\n",
        "Received data chunk: {\"id\":101,\"timestamp\":1623766800,\"content\":\"Hello, ",
        "this is a test message\",\"user\":{\"id\":1,\"name\":\"System",
        "\"}}",
        "\nStatus: Message processed successfully\n",
        "Log entry [2023-06-15 14:30:05]: Processing next message\n",
        "Received data: {\"id\":102,\"timestamp\":1623766805,\"content\":\"Another test ",
        "message with streaming JSON\",\"user\":{\"id\":2,\"name\":\"Alice\"}}",
        "\nStatus: Message processed successfully\n",
        "Log entry [2023-06-15 14:30:10]: Continuing stream...\n",
        "Processing batch: {\"id\":103,\"timestamp\":1623766810,\"content\":\"This JSON is split ",
        "across multiple chunks to demonstrate ",
        "buffering capabilities\",\"user\":{\"id\":3,\"name\":\"Bob\"}}",
        "\nLog entry [2023-06-15 14:30:15]: Final message incoming\n",
        "Last message: {\"id\":104,\"timestamp\":1623766815,\"content\":\"Goodbye!\",",
        "\"user\":{\"id\":1,\"name\":\"System\"}}",
        "\nStatus: Stream processing complete.\n",
    ];

    println!(
        "Simulating stream processing with {} chunks:",
        stream_chunks.len()
    );
    println!("(Processing one chunk every 500ms)\n");

    // Process each chunk as it arrives
    for (i, chunk) in stream_chunks.iter().enumerate() {
        println!("Chunk {}: {}", i + 1, chunk.trim());

        // In a real application, this would be where you receive data from a socket,
        // read from a file incrementally, etc.

        // Add a small delay to simulate real-time processing
        thread::sleep(Duration::from_millis(500));

        // Process the chunk with our deserializer
        match deserializer.process_chunk(chunk) {
            Some(message) => {
                // Successfully extracted and deserialized a complete Message object
                println!("\n✅ Successfully extracted and deserialized a Message:");
                println!("   ID: {}", message.id);
                println!("   Timestamp: {}", message.timestamp);
                println!("   Content: {}", message.content);
                println!("   User: {} (ID: {})", message.user.name, message.user.id);
                println!(
                    "\n   JSONParser status: {}\n",
                    if deserializer.is_in_json() {
                        "In JSON"
                    } else {
                        "Not in JSON"
                    }
                );
            }
            None => {
                // No complete JSON object available yet
                if deserializer.is_in_json() {
                    println!(
                        "   Continuing to collect JSON (Accumulated so far: {})",
                        if deserializer.accumulated_json().len() > 30 {
                            format!("{}...", &deserializer.accumulated_json()[..30])
                        } else {
                            deserializer.accumulated_json().to_string()
                        }
                    );
                } else {
                    println!("   No JSON content in this chunk");
                }
                println!("");
            }
        }
    }

    println!("Stream processing complete.");
    println!("The StreamingDeserializer automatically handles:");
    println!(" - Mixed text with embedded JSON");
    println!(" - JSON objects split across multiple chunks");
    println!(" - Automatic deserialization when a complete object is available");
}
