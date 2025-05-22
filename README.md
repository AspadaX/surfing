# Surfing 🏄

A Rust library for parsing JSON objects from text streams.

[![Crates.io](https://img.shields.io/crates/v/surfing.svg)](https://crates.io/crates/surfing)
[![Documentation](https://docs.rs/surfing/badge.svg)](https://docs.rs/surfing)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What is Surfing?

Surfing is a lightweight Rust library that extracts JSON objects from mixed text content. 

## Real-world Use Cases

### Extracting Structured Data from LLM Outputs

Large Language Models often output JSON mixed with explanatory text, and reasoning models may output jsons with their reasoning steps. Surfing makes it easy to extract and use this structured data:

```rust
use std::io::stdout;
use surfing::JSONParser;

// Process LLM response containing JSON
let llm_response = "Here's the user profile: {\"id\":123,\"name\":\"Alice\",\"role\":\"admin\"} Let me know if you need more info.";

let mut parser = JSONParser::new();
let mut lock = stdout().lock();

// Extract only the JSON part
parser.extract_json_from_stream(&mut lock, llm_response).unwrap();
// Output: {"id":123,"name":"Alice","role":"admin"}
```

### Processing Streaming LLM Responses

When working with streaming API responses, JSON often arrives in chunks. Surfing handles this seamlessly:

```rust
use std::io::stdout;
use surfing::JSONParser;

// Initialize the parser
let mut json_parser = JSONParser::new();
let mut lock = stdout().lock();

// Process each chunk as it arrives from a streaming API
let chunks = [
    "The weather forecast is {\"location\":\"New York\",",
    "\"temperature\":72,\"conditions\":\"sunny\"}",
    " Hope that helps!"
];

for chunk in chunks {
    json_parser.extract_json_from_stream(&mut lock, chunk).unwrap();
}
// Outputs: {"location":"New York","temperature":72,"conditions":"sunny"}
```

### Deserializing JSON Directly into Rust Structs

With the `serde` feature enabled, you can directly extract and deserialize JSON into your data structures:

```rust
use serde::Deserialize;
use surfing::serde::StreamingDeserializer;

#[derive(Debug, Deserialize)]
struct Weather {
    location: String,
    temperature: i32,
    conditions: String,
}

// Create a deserializer for Weather structs
let mut deserializer = StreamingDeserializer::<Weather>::new();

// Process chunks as they arrive
let chunks = [
    "The weather forecast is {\"location\":\"New York\",",
    "\"temperature\":72,\"conditions\":\"sunny\"} Hope that helps!"
];

for chunk in chunks {
    let result = deserializer.process_chunk(chunk).unwrap();
    if let Some(weather) = result {
        println!("Weather in {}: {}°F, {}", 
            weather.location, 
            weather.temperature,
            weather.conditions);
    }
}

// Output: Weather in New York: 72°F, sunny
```

### Processing Log Files with Embedded JSON

Many modern logging systems emit JSON data. Surfing helps extract and analyze this data:

```rust
use std::io::BufWriter;
use surfing::JSONParser;

// Log entries with embedded JSON
let log_entries = r#"
[2023-06-15 14:30:00] INFO: System starting
[2023-06-15 14:30:01] DEBUG: {"component":"database","status":"connected","latency_ms":45}
[2023-06-15 14:30:05] ERROR: {"error":"connection_timeout","service":"auth","attempts":3}
[2023-06-15 14:30:10] INFO: System ready
"#;

let mut parser = JSONParser::new();
let mut buffer = Vec::new();
{
    let mut writer = BufWriter::new(&mut buffer);
    parser.extract_json_from_stream(&mut writer, log_entries).unwrap();
}

let json_only = String::from_utf8(buffer).unwrap();
println!("{}", json_only);
// Output:
// {"component":"database","status":"connected","latency_ms":45}
// {"error":"connection_timeout","service":"auth","attempts":3}
```

## Installation

Add `surfing` to your project:

```bash
cargo add surfing
```

Or with serde support:

```bash
cargo add surfing --features serde
```

## How It Works

Surfing works by:

1. Watching for JSON opening markers (`{` or `[`)
2. Tracking nested JSON structures
3. Writing only the JSON content to your output
4. Resetting state when complete JSON objects are found

The parser is stateful, so it can handle JSON objects split across multiple chunks.

## Key Advantages

- **Zero external dependencies** in the core library
- **Streaming-friendly** for processing large files or API responses
- **Memory-efficient** with minimal state tracking
- **Serde integration** for direct deserialization (optional)
- **Simple API** with both high and low-level options

## Learn More

Check out the examples directory for more use cases:

- `openai_json_extraction.rs` - Extracting JSON from OpenAI API responses
- `basic.rs` - Simple extraction from mixed text
- `streaming.rs` - Processing data in chunks
- `stdout.rs` - Filtering JSON to standard output

## License

This project is licensed under the MIT License - see the LICENSE file for details.