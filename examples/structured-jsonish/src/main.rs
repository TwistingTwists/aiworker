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

#[cfg(test)]
mod test { 
    use super::*;
        use schemars::JsonSchema;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, JsonSchema)]
    struct Person {
        name: String,
        age: u8,
        address: Address,
        hobbies: Vec<Hobby>,
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    struct Address {
        street: String,
        city: String,
        country: String,
        coordinates: Option<Coordinates>,
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    struct Coordinates {
        lat: f64,
        lng: f64,
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    struct Hobby {
        name: String,
        years_active: u8,
        proficiency: ProficiencyLevel,
    }

    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum ProficiencyLevel {
        Beginner,
        Intermediate,
        Expert,
    }

#[test]
fn test_complex_parsing() {
    let response = r#"
    ```json
    {
        "name": "Bob",
        "age": 42,
        "address": {
            "street": "789 Pine Rd",
            "city": "Metropolis",
            "country": "USA",
            "coordinates": {"lat": 40.7128, "lng": -74.0060}
        },
        "hobbies": [
            {"name": "Cooking", "years_active": 5, "proficiency": "intermediate"},
            {"name": "Cycling", "years_active": 10, "proficiency": "expert"}
        ]
    }
    ```
    "#;

    let parsed = jsonish::parse(response, Default::default()).unwrap();
    let value = jsonish_to_serde(&parsed);
    let person: Person = serde_json::from_value(value).unwrap();

    assert_eq!(person.name, "Bob");
    assert_eq!(person.hobbies[1].proficiency, ProficiencyLevel::Expert);
    assert!(person.address.coordinates.is_some());
}


#[test]
fn test_not_marked_code_block() {
    let response = r#"
    ```
    {
        "name": "Bob",
        "age": 42,
        "address": {
            "street": "789 Pine Rd",
            "city": "Metropolis",
            "country": "USA",
            "coordinates": {"lat": 40.7128, "lng": -74.0060}
        },
        "hobbies": [
            {"name": "Cooking", "years_active": 5, "proficiency": "intermediate"},
            {"name": "Cycling", "years_active": 10, "proficiency": "expert"}
        ]
    }
    ```
    "#;

    let parsed = jsonish::parse(response, Default::default()).unwrap();
    let value = jsonish_to_serde(&parsed);
    let person: Person = serde_json::from_value(value).unwrap();

    assert_eq!(person.name, "Bob");
    assert_eq!(person.hobbies[1].proficiency, ProficiencyLevel::Expert);
    assert!(person.address.coordinates.is_some());
}



#[test]
fn test_markdown() {
    let response = r#"
    Here is your json response:
    
    {
        "name": "Bob",
        "age": 42,
        "address": {
            "street": "789 Pine Rd",
            "city": "Metropolis",
            "country": "USA",
            "coordinates": {"lat": 40.7128, "lng": -74.0060}
        },
        "hobbies": [
            {"name": "Cooking", "years_active": 5, "proficiency": "intermediate"},
            {"name": "Cycling", "years_active": 10, "proficiency": "expert"}
        ]
    }
    
    You can use this now.
    "#;

    let parsed = jsonish::parse(response, Default::default()).unwrap();
    let value = jsonish_to_serde(&parsed);
    let person: Person = serde_json::from_value(value).unwrap();

    assert_eq!(person.name, "Bob");
    assert_eq!(person.hobbies[1].proficiency, ProficiencyLevel::Expert);
    assert!(person.address.coordinates.is_some());
}


#[test]
#[should_panic(expected = "missing field `name`")]
fn test_missing_required_field() {
    // Missing "name" field should fail
    let response = r#"
    ```json
    {
        "age": 15,
        "address": {
            "street": "123 Main St",
            "city": "Anytown",
            "country": "USA"
        },
        "hobbies": []
    }
    ```
    "#;

    let parsed = jsonish::parse(response, Default::default()).unwrap();
    let value = jsonish_to_serde(&parsed);
    let _person: Person = serde_json::from_value(value).unwrap(); // Should panic
}

#[test]
#[should_panic(expected = "unknown variant `EXPERT`, expected one of `beginner`, `intermediate`, `expert`")]
fn test_enum_value_mismatch() {
    // Invalid enum value format (uppercase)
    let response = r#"
    ```json
    {
        "name": "Eve",
        "age": 28,
        "address": {
            "street": "456 Oak Rd",
            "city": "Techville",
            "country": "Digitaland"
        },
        "hobbies": [{
            "name": "Hacking",
            "years_active": 6,
            "proficiency": "EXPERT"  // Should be lowercase
        }]
    }
    ```
    "#;

    let parsed = jsonish::parse(response, Default::default()).unwrap();
    let value = jsonish_to_serde(&parsed);
    let _person: Person = serde_json::from_value(value).unwrap(); // Should panic
}


}