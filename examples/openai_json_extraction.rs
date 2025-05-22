use std::io::{stdout, StdoutLock, Write};

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use surfing::serde::StreamingDeserializer;
use surfing::JSONParser;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pizza {
    name: String,
    toppings: Vec<String>,
}

impl Default for Pizza {
    fn default() -> Self {
        Self {
            name: "the name of the pizza".to_string(),
            toppings: vec!["toppings of the pizza".to_string()],
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: OpenAIConfig = OpenAIConfig::default()
        .with_api_base("https://api.deepseek.com")
        .with_api_key("your_api_key");
    let client: Client<OpenAIConfig> = Client::with_config(config);

    let request: CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
        .model("deepseek-reasoner")
        .messages(vec![ChatCompletionRequestUserMessageArgs::default()
            .content(format!(
                "Output the ingredients of pizza in json. json format: {}",
                serde_json::to_string_pretty(&Pizza::default())?
            ))
            .build()?
            .into()])
        .build()?;

    let mut json_parser = JSONParser::new();
    let mut deserializer: StreamingDeserializer<Pizza> = StreamingDeserializer::new();
    let mut stream = client.chat().create_stream(request).await?;
    let mut lock: StdoutLock<'_> = stdout().lock();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        // Just parse the json
                        json_parser
                            .extract_json_from_stream(&mut lock, content)
                            .unwrap();
                        
                        // Or, stream directly into a defined struct
                        if let Some(data) = deserializer.process_chunk(content) {
                            println!("{:#?}", data);
                        }
                    }
                });
            }
            Err(err) => {
                writeln!(lock, "error: {err}").unwrap();
            }
        }
        stdout().flush()?;
    }

    Ok(())
}
