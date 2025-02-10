use std::error::Error;
use futures::StreamExt;
use common::types::ChatCompletionRequestUserMessageArgs;
use common::{types::CreateChatCompletionRequestArgs, Client};

use json_partial::jsonish::{self, jsonish_to_serde,ParseOptions};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    age: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create an instance of the client.
    let client = Client::new();

    // Update the prompt so the LLM returns only valid JSON conforming to our schema.
    let prompt = r#"Please return a valid JSON string that conforms to the following structure:
{
  "name": "Alice",
  "age": 30
}
Do not include any extra text."#;

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .max_tokens(512u32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into()])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    // Instead of printing chunks as they arrive, accumulate them.
    let mut output = String::new();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                // Accumulate all received content.
                for chat_choice in &response.choices {
                    if let Some(ref content) = chat_choice.delta.content {
                        output.push_str(content);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error while streaming response: {err}");
            }
        }
    }

    // (Optional) Print the raw output from the LLM.
    println!("Raw output from LLM:\n{}\n", output);
    // After the while loop
    let parsed = jsonish::parse(&output, jsonish::ParseOptions::default())
    .map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))?;

    let parsed_value = jsonish_to_serde(&parsed);
    let person: Person = serde_json::from_value(parsed_value)
    .map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))?;

    println!( "\n\nParsed Person: {:?}", person);

    Ok(())
}
