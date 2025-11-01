use crate::config::Config;
use crate::gigachat::GigaChatClient;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    message: OpenAIMessage,
}

pub struct ClippyAgent {
    config: Config,
    conversation_history: VecDeque<Message>,
    gigachat_client: Option<GigaChatClient>,
}

impl ClippyAgent {
    pub fn new(config: Config) -> Self {
        // Пытаемся создать GigaChat клиент если доступен API ключ
        let gigachat_client = config.gigachat_api_key.as_ref().and_then(|key| {
            if key.is_empty() {
                None
            } else {
                Some(GigaChatClient::new(
                    key.clone(),
                    Some(config.gigachat_model.clone()),
                    Some(config.gigachat_temperature),
                    Some(config.gigachat_max_tokens),
                ))
            }
        });

        Self {
            config,
            conversation_history: VecDeque::new(),
            gigachat_client,
        }
    }

    pub async fn get_response(&mut self, user_input: &str) -> String {
        if user_input.trim().is_empty() {
            return "Чем могу помочь?".to_string();
        }

        // Приоритет: GigaChat → OpenAI → Local
        if let Some(client) = &mut self.gigachat_client {
            match client.get_response(user_input).await {
                Ok(response) => {
                    self.conversation_history.push_back(Message {
                        role: "user".to_string(),
                        content: user_input.to_string(),
                    });
                    self.conversation_history.push_back(Message {
                        role: "assistant".to_string(),
                        content: response.clone(),
                    });

                    // Ограничиваем историю 10 сообщениями
                    while self.conversation_history.len() > 10 {
                        self.conversation_history.pop_front();
                    }

                    return response;
                }
                Err(e) => {
                    eprintln!("GigaChat ошибка: {}", e);
                    // Fallback на OpenAI или Local
                }
            }
        }

        // Fallback на OpenAI
        if self.config.use_openai && self.config.openai_api_key.is_some() {
            return self.get_openai_response(user_input).await;
        }

        // Fallback на Local
        self.get_local_response(user_input)
    }

    async fn get_openai_response(&mut self, user_input: &str) -> String {
        let api_key = match &self.config.openai_api_key {
            Some(key) => key,
            None => return "OpenAI API ключ не установлен".to_string(),
        };

        self.conversation_history.push_back(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let messages: Vec<OpenAIMessage> = self
            .conversation_history
            .iter()
            .map(|m| OpenAIMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages,
            temperature: 0.7,
            max_tokens: 200,
        };

        let client = reqwest::Client::new();
        let response = match client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("OpenAI запрос ошибка: {}", e);
                return "Ошибка связи с OpenAI".to_string();
            }
        };

        match response.json::<OpenAIResponse>().await {
            Ok(openai_resp) => {
                if let Some(choice) = openai_resp.choices.first() {
                    let assistant_message = choice.message.content.clone();
                    self.conversation_history.push_back(Message {
                        role: "assistant".to_string(),
                        content: assistant_message.clone(),
                    });

                    // Ограничиваем историю 10 сообщениями
                    while self.conversation_history.len() > 10 {
                        self.conversation_history.pop_front();
                    }

                    assistant_message
                } else {
                    "Нет ответа от OpenAI".to_string()
                }
            }
            Err(e) => {
                eprintln!("OpenAI разбор ошибка: {}", e);
                "Ошибка разбора ответа от OpenAI".to_string()
            }
        }
    }

    fn get_local_response(&self, user_input: &str) -> String {
        let input_lower = user_input.to_lowercase();

        // Приветствия
        if input_lower.contains("привет") || input_lower.contains("здравствуй") {
            return "Привет! Как дела? Чем я могу тебе помочь?".to_string();
        }

        // Прощание
        if input_lower.contains("пока") || input_lower.contains("до свидания") {
            return "До свидания! Удачи тебе!".to_string();
        }

        // Помощь
        if input_lower.contains("помощь") || input_lower.contains("помоги") {
            return "Я могу помочь с:\n• Информацией о погоде\n• Курсами валют\n• Ответами на вопросы\n• Общением и консультациями".to_string();
        }

        // Время
        if input_lower.contains("время") || input_lower.contains("который час") {
            return "Пожалуйста, посмотрите время в системе.".to_string();
        }

        // По умолчанию
        "Интересный вопрос! Для более полного ответа рекомендую подключить GigaChat API. Могу ли я чем-то ещё помочь?".to_string()
    }

    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
        if let Some(client) = &mut self.gigachat_client {
            client.clear_history();
        }
    }

    pub fn get_history(&self) -> Vec<(String, String)> {
        self.conversation_history
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect()
    }
}
