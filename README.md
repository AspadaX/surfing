# Surfing 🏄

A Rust library for parsing JSON objects from text streams.

[![Crates.io](https://img.shields.io/crates/v/surfing.svg)](https://crates.io/crates/surfing)
[![Documentation](https://docs.rs/surfing/badge.svg)](https://docs.rs/surfing)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Surfing provides utilities to extract JSON objects from text streams, making it particularly useful for:

- Processing log files containing JSON entries mixed with plain text
- Extracting JSON objects from console output
- Handling streaming JSON data that might arrive in chunks
- Filtering JSON content from mixed data sources

## Features

- Extract JSON objects and arrays from mixed text content
- Support for processing partial JSON (streaming)
- Simple API that works with any writer implementing the `Write` trait
- High-level utility functions for common use cases
- Serde integration for direct deserialization (optional feature)
- Streaming deserializer for handling JSON in data streams
- Zero dependencies (aside from `anyhow` for error handling)

## Installation

Add this to your `Cargo.toml`:

```toml
# Basic functionality
[dependencies]
surfing = "0.1.0"

# Or with Serde support
[dependencies]
surfing = { version = "0.1.0", features = ["serde"] }
```

## Usage

### Basic Extraction

Extract JSON objects from text containing non-JSON content:

```rust
use std::io::BufWriter;
use surfing::JSONParser;

let mut parser = JSONParser::new();
let mut buffer = Vec::new();

{
    let mut writer = BufWriter::new(&mut buffer);
    parser.extract_json_from_stream(
        &mut writer, 
        "Log entry: {\"level\":\"info\",\"message\":\"Server started\"} End of line"
    ).unwrap();
}

let json = String::from_utf8(buffer).unwrap();
assert_eq!(json, "{\"level\":\"info\",\"message\":\"Server started\"}");
```

### Simple Utility Function

For simple use cases, use the high-level utility function:

```rust
use surfing::extract_json_to_string;

let input = "Log entry: {\"level\":\"info\",\"message\":\"Server started\"} End of line";
let json = extract_json_to_string(input).unwrap();
assert_eq!(json, "{\"level\":\"info\",\"message\":\"Server started\"}");
```

### Processing Streaming Data

Handle JSON that might arrive in chunks:

```rust
use std::io::BufWriter;
use surfing::JSONParser;

let mut parser = JSONParser::new();
let mut buffer = Vec::new();

{
    let mut writer = BufWriter::new(&mut buffer);
    
    // First chunk with partial JSON
    parser.extract_json_from_stream(&mut writer, "Starting {\"status\":").unwrap();
    
    // Second chunk completing the JSON
    parser.extract_json_from_stream(&mut writer, "\"running\",\"uptime\":42}").unwrap();
}

let json = String::from_utf8(buffer).unwrap();
assert_eq!(json, "{\"status\":\"running\",\"uptime\":42}");
```

### Using with Standard Output

Process JSON and write directly to stdout:

```rust
use std::io::stdout;
use surfing::JSONParser;

let mut parser = JSONParser::new();

// Lock stdout for better performance with multiple writes
let stdout = stdout();
let mut handle = stdout.lock();

// This would print only the JSON part to the console
parser.extract_json_from_stream(
    &mut handle, 
    "Config: {\"debug\":true,\"port\":8080}"
).unwrap();
```

## Performance Considerations

### Buffering

For optimal performance when processing large files or streams:

- Use `BufWriter` or `BufReader` to reduce the number of system calls
- Process data in chunks of appropriate size (typically 4-8KB) 
- Reuse parser instances when processing multiple chunks to maintain state

### Memory Usage

The parser stores minimal state:

- Current JSON nesting level
- A small buffer for tracking markers

This makes it suitable for processing large streams with minimal memory overhead.

## Serde Integration

When enabled with the `serde` feature, you can deserialize directly from mixed text:

```rust
use serde::Deserialize;
use surfing::serde::from_mixed_text;

#[derive(Debug, Deserialize)]
struct LogEntry {
    level: String,
    message: String,
}

// Text with embedded JSON
let input = "Log entry: {\"level\":\"info\",\"message\":\"Started server\"} End of line";

// Directly deserialize the JSON part into a struct
let entry: LogEntry = from_mixed_text(input).unwrap();
assert_eq!(entry.level, "info");
assert_eq!(entry.message, "Started server");
```

### Streaming Deserialization

Process and deserialize streaming data in two ways:

#### High-level StreamingDeserializer

For a more convenient API, use the `StreamingDeserializer`:

```rust
use serde::Deserialize;
use surfing::serde::StreamingDeserializer;

#[derive(Debug, Deserialize)]
struct User {
    id: u64,
    name: String,
}

// Create a deserializer for User structs
let mut deserializer = StreamingDeserializer::<User>::new();

// Process chunks as they arrive
let chunks = [
    "Log line {\"id\":",
    "42,\"name\":\"Alice\"}",
    " more text"
];

// First chunk - incomplete JSON
let result = deserializer.process_chunk(chunks[0]);
assert!(result.is_none());

// Second chunk - completes the JSON
let result = deserializer.process_chunk(chunks[1]);
assert!(result.is_some());
let user = result.unwrap();
assert_eq!(user.id, 42);

// Third chunk - no more JSON to extract
let result = deserializer.process_chunk(chunks[2]);
assert!(result.is_none());
```

#### Low-level API

```rust
use serde::Deserialize;
use surfing::JSONParser;
use surfing::serde::from_mixed_text_with_parser;

#[derive(Debug, Deserialize)]
struct Config {
    name: String,
    port: u16,
}

let mut parser = JSONParser::new();

// Process the chunks as they arrive
let chunk1 = "Config: {\"name\":\"";
let chunk2 = "api-server\",\"port\":8080}";

// First chunk (incomplete)
match from_mixed_text_with_parser::<Config>(&mut parser, chunk1) {
    Ok(_) => println!("Complete"),
    Err(_) => println!("Incomplete, waiting for more data"),
}

// Second chunk completes the JSON
let config: Config = from_mixed_text_with_parser(&mut parser, chunk2).unwrap();
assert_eq!(config.name, "api-server");
assert_eq!(config.port, 8080);
```

## Examples

Check the [examples](https://github.com/surfing/surfing/tree/main/examples) directory for more detailed usage scenarios:

- `basic.rs` - Simple extraction from mixed text
- `streaming.rs` - Processing data in chunks
- `stdout.rs` - Filtering JSON to standard output
- `simple.rs` - Using the high-level utility functions
- `serde_integration.rs` - Using Serde to deserialize extracted JSON
- `streaming_serde.rs` - Using StreamingDeserializer for stream processing

## License

This project is licensed under the MIT License - see the LICENSE file for details.
