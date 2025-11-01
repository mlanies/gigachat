use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

/// Клиент для работы с OpenAI API
pub struct OpenAIClient {
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
    conversation_history: VecDeque<Message>,
}

impl OpenAIClient {
    pub fn new(
        api_key: String,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            temperature: temperature.unwrap_or(0.7),
            max_tokens: max_tokens.unwrap_or(200),
            conversation_history: VecDeque::with_capacity(10),
        }
    }

    pub async fn get_response(&mut self, user_input: &str) -> anyhow::Result<String> {
        self.conversation_history.push_back(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let messages: Vec<Message> = self.conversation_history.iter().cloned().collect();

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("OpenAI API error: {}", response.status()));
        }

        let openai_resp: OpenAIResponse = response.json().await?;

        if let Some(choice) = openai_resp.choices.first() {
            let assistant_message = choice.message.content.clone();
            self.conversation_history.push_back(Message {
                role: "assistant".to_string(),
                content: assistant_message.clone(),
            });

            while self.conversation_history.len() > 10 {
                self.conversation_history.pop_front();
            }

            Ok(assistant_message)
        } else {
            Err(anyhow::anyhow!("No response from OpenAI"))
        }
    }

    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && self.api_key != "not-configured"
    }
}
