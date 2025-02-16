use reqwest::Client;
// use serde::{Deserialize, Serialize};
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::env;
use serde_json::Value;

// #[derive(Serialize)]
// struct AIRequest {
//     model: String,
//     messages: Vec<Message>,
//     response_format: ResponseFormat,
// }

// #[derive(Serialize)]
// struct Message {
//     role: String,
//     content: String,
// }

// #[derive(Serialize)]
// struct ResponseFormat {
//     #[serde(rename = "type")]
//     format_type: String,
//     json_schema: Schema,
// }

// #[derive(Serialize)]
// struct Schema {
//     name: String,
//     strict: bool,
//     schema: serde_json::Value,
// }

#[derive(Deserialize)]
struct AIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Option<String>,
    refusal: Option<String>,
}

pub async fn get_ai_response(command: &str, error: &str) -> Result<Value, Box<dyn Error>> {
    // let openai_api_key = OPENAI_API_KEY
    let openai_api_key = match env::var_os("OPENAI_API_KEY") {
        Some(key) => key.into_string().unwrap_or_else(|_| "default_key".to_string()),
        None => "default_key".to_string(),
    };
    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";
    let instruction = "You are a tool that receives failed shell commands and their output, and your job is to suggest a corrected command. You will receive a JSON schema that describes the expected output. Your response should be a JSON object that matches the schema.";
    let schema = response_schema();

    let payload = json!({
        "model": "gpt-4o",
        "messages": [
          {"role": "system", "content": instruction},
          {"role": "user", "content": command.to_string()},
          {"role": "user", "content": error.to_string()}
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "command_info",
                "strict": true,
                "schema": schema
            }
        }
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&payload)
        .send()
        .await?;

    let response_text = response.text().await?;
    // println!("🔍 API Response: {}", response_text); // Debugging output

    let parsed_response: AIResponse = serde_json::from_str(&response_text)?;

    if let Some(refusal) = parsed_response.choices.get(0).and_then(|c| c.message.refusal.clone()) {
        return Ok(json!({ "error": "Refused request", "message": refusal }));
    }

    if let Some(content) = parsed_response.choices.get(0).and_then(|c| c.message.content.clone()) {
        let structured_json: serde_json::Value = serde_json::from_str(&content)?;
        return Ok(structured_json);
    }

    Err("Unexpected API response format".into())
}

pub fn response_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "suggested_exact_command": {
                "type": "string",
                "description": "ONLY the exact command that a user could be looking to type, no explanations here."
            },
            "command_explanation": { "type": "string" }
        },
        "required": ["suggested_exact_command", "command_explanation"],
        "additionalProperties": false
    })
}
