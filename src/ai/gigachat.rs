use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::collections::VecDeque;

/// Клиент для работы с Yandex GigaChat API
pub struct GigaChatClient {
    api_key: String,
    base_url: String,
    model: String,
    temperature: f32,
    max_tokens: i32,
    conversation_history: VecDeque<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct GigaChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
    pub n: i32,
}

#[derive(Debug, Deserialize)]
pub struct GigaChatResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

impl GigaChatClient {
    /// Создает новый клиент GigaChat
    pub fn new(
        api_key: String,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<i32>,
    ) -> Self {
        Self {
            api_key,
            base_url: "https://gigachat.devices.sberbank.ru/api/v1".to_string(),
            model: model.unwrap_or_else(|| "GigaChat:latest".to_string()),
            temperature: temperature.unwrap_or(0.7),
            max_tokens: max_tokens.unwrap_or(200),
            conversation_history: VecDeque::with_capacity(10),
        }
    }

    /// Отправляет сообщение в GigaChat и получает ответ
    pub async fn get_response(&mut self, user_input: &str) -> Result<String> {
        // Добавляем сообщение пользователя в историю
        self.conversation_history.push_back(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        // Ограничиваем историю последними 10 сообщениями
        while self.conversation_history.len() > 10 {
            self.conversation_history.pop_front();
        }

        // Создаем запрос
        let request = GigaChatRequest {
            model: self.model.clone(),
            messages: self.conversation_history.iter().cloned().collect(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            top_p: 0.9,
            n: 1,
        };

        // Отправляем запрос к GigaChat API
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            eprintln!(
                "GigaChat API error ({}): {}",
                status, text
            );
            return Err(anyhow::anyhow!(
                "GigaChat API error: {} - {}",
                status,
                text
            ));
        }

        let chat_response: GigaChatResponse = serde_json::from_str(&text)?;

        if let Some(choice) = chat_response.choices.first() {
            let assistant_message = choice.message.content.clone();

            // Добавляем ответ ассистента в историю
            self.conversation_history.push_back(Message {
                role: "assistant".to_string(),
                content: assistant_message.clone(),
            });

            // Ограничиваем историю
            while self.conversation_history.len() > 10 {
                self.conversation_history.pop_front();
            }

            Ok(assistant_message)
        } else {
            Err(anyhow::anyhow!("No response from GigaChat"))
        }
    }

    /// Устанавливает модель GigaChat
    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    /// Устанавливает температуру (0.0 - 1.0)
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature.clamp(0.0, 1.0);
    }

    /// Устанавливает максимальное количество токенов
    pub fn set_max_tokens(&mut self, max_tokens: i32) {
        self.max_tokens = max_tokens.max(1);
    }

    /// Очищает историю разговора
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }

    /// Возвращает текущую историю разговора
    pub fn get_history(&self) -> Vec<Message> {
        self.conversation_history.iter().cloned().collect()
    }

    /// Проверяет, доступен ли API ключ
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && self.api_key != "not-configured"
    }
}
