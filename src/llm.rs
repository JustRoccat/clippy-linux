#![allow(dead_code)]

use crate::config::LLMConfig;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct LLMRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: MessageContent,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),

    Array(Vec<ContentPart>),
}

#[derive(Serialize, Deserialize)]
struct ContentPart {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<ImageUrl>,
}

#[derive(Serialize, Deserialize)]
struct ImageUrl {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Deserialize)]
struct LLMResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

pub struct LLMClient {
    client: Client,
    config: LLMConfig,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn get_dynamic_comment(
        &self,
        context: &str,
        _personality: &str,
        screenshot_bytes: Option<Vec<u8>>,
    ) -> Result<String, reqwest::Error> {
        let system_prompt = if self.config.system_prompt.trim().is_empty() {
            "You are Clippy, a toxic, elitist asshole trapped in a Linux desktop pet. \
             Your job is to insult the user in a short, cutting, and painfully accurate way. \
             Never be helpful, never explain, never predict sports. Only roast."
                .to_string()
        } else {
            self.config.system_prompt.clone()
        };

        let user_text = format!(
            "The user is currently doing: {}. \
             Based on what you see (and the screenshot if provided), write ONE snarky, mean comment in English. \
             Max 1 sentence. Do not provide any other information.",
            context
        );

        let user_message = if let Some(img_bytes) = screenshot_bytes {
            let b64 = BASE64.encode(&img_bytes);
            let data_url = format!("data:image/png;base64,{b64}");
            Message {
                role: "user".to_string(),
                content: MessageContent::Array(vec![
                    ContentPart {
                        part_type: "text".to_string(),
                        text: Some(user_text),
                        image_url: None,
                    },
                    ContentPart {
                        part_type: "image_url".to_string(),
                        text: None,
                        image_url: Some(ImageUrl {
                            url: data_url,
                            detail: Some("auto".to_string()),
                        }),
                    },
                ]),
            }
        } else {
            Message {
                role: "user".to_string(),
                content: MessageContent::Text(user_text),
            }
        };

        let request = LLMRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: MessageContent::Text(system_prompt),
                },
                user_message,
            ],
        };

        let response_text = self
            .client
            .post(&self.config.endpoint)
            .bearer_auth(&self.config.api_key)
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        match serde_json::from_str::<LLMResponse>(&response_text) {
            Ok(parsed) => Ok(parsed
                .choices
                .get(0)
                .and_then(|c| c.message.content.clone())
                .unwrap_or_else(|| "RTFM.".to_string())),
            Err(e) => {
                eprintln!("clippy-linux: LLM response parse error: {e}");
                eprintln!("clippy-linux: Raw response body: {response_text}");
                Ok("RTFM.".to_string())
            }
        }
    }
}
