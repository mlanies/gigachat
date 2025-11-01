use crate::config::Config;
use crate::ai::GigaChatClient;
use crate::ai::local::LocalAI;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
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
        LocalAI::get_response(user_input)
    }

    async fn get_openai_response(&mut self, user_input: &str) -> String {
        // TODO: Реализовать OpenAI интеграцию через модуль ai::openai
        "OpenAI ещё не интегрирован в эту версию.".to_string()
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
