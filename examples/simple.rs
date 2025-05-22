//! Simple example using the extract_json_to_string utility
//!
//! This example demonstrates the simplest way to extract JSON from text
//! using the high-level utility function.

use surfing::extract_json_to_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Input containing mixed text and JSON
    let inputs = [
        "System log: {\"level\":\"info\",\"component\":\"auth\",\"message\":\"User logged in\"} at 2023-06-15",
        "Error occurred: {\"code\":500,\"details\":{\"reason\":\"Database connection failed\"}}",
        "Metrics: [1, 2, 3, 4, 5] recorded at 12:34:56",
        "Configuration: {\"debug\":true,\"environment\":\"production\"} loaded successfully",
    ];

    println!("Extracting JSON from various text inputs:\n");

    for (i, input) in inputs.iter().enumerate() {
        // Extract the JSON directly to a string
        let json = extract_json_to_string(input)?;

        // Display the results
        println!("Input {}: {}", i + 1, input);
        println!("Extracted JSON: {}\n", json);
    }

    // You can also chain multiple processing steps
    let complex_input = "First config: {\"id\":1} Second config: {\"id\":2} Third: {\"id\":3}";
    println!(
        "Complex input with multiple JSON objects: {}",
        complex_input
    );

    let extracted = extract_json_to_string(complex_input)?;
    println!("All extracted: {}", extracted);

    Ok(())
}
