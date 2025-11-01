use crate::config::Config;
use crate::ai::GigaChatClient;
use crate::ai::local::LocalAI;
use crate::services::SQLiteStorage;
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
    storage: Option<SQLiteStorage>,
    current_model: String,
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

        // Инициализируем хранилище
        let storage = match SQLiteStorage::new(None) {
            Ok(s) => {
                log::info!("✓ SQLiteStorage инициализирован");
                Some(s)
            }
            Err(e) => {
                log::warn!("⚠️ Ошибка инициализации SQLiteStorage: {}", e);
                None
            }
        };

        Self {
            config,
            conversation_history: VecDeque::new(),
            gigachat_client,
            storage,
            current_model: "Local".to_string(),
        }
    }

    pub async fn get_response(&mut self, user_input: &str) -> String {
        if user_input.trim().is_empty() {
            return "Чем могу помочь?".to_string();
        }

        let response = self.get_ai_response(user_input).await;

        // Сохраняем в историю памяти
        self.conversation_history.push_back(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });
        self.conversation_history.push_back(Message {
            role: "assistant".to_string(),
            content: response.clone(),
        });

        // Ограничиваем историю 10 сообщениями в памяти
        while self.conversation_history.len() > 10 {
            self.conversation_history.pop_front();
        }

        // Сохраняем в БД (асинхронно, не блокируем ответ)
        if let Some(ref storage) = self.storage {
            if let Err(e) = storage.save_message("user", user_input, &self.current_model) {
                log::error!("Ошибка сохранения user message в БД: {}", e);
            }
            if let Err(e) = storage.save_message("assistant", &response, &self.current_model) {
                log::error!("Ошибка сохранения assistant message в БД: {}", e);
            }
        }

        response
    }

    async fn get_ai_response(&mut self, user_input: &str) -> String {
        // Приоритет: GigaChat → OpenAI → Local
        if let Some(client) = &mut self.gigachat_client {
            match client.get_response(user_input).await {
                Ok(response) => {
                    self.current_model = "GigaChat".to_string();
                    log::debug!("📡 Используется GigaChat");
                    return response;
                }
                Err(e) => {
                    log::warn!("⚠️ GigaChat ошибка: {}", e);
                    // Fallback на OpenAI или Local
                }
            }
        }

        // Fallback на OpenAI
        if self.config.use_openai && self.config.openai_api_key.is_some() {
            self.current_model = "OpenAI".to_string();
            log::debug!("📡 Используется OpenAI");
            return self.get_openai_response(user_input).await;
        }

        // Fallback на Local
        self.current_model = "Local".to_string();
        log::debug!("📡 Используются локальные правила");
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

        if let Some(ref storage) = self.storage {
            if let Err(e) = storage.clear_session_history() {
                log::error!("Ошибка при очистке истории в БД: {}", e);
            }
        }

        log::info!("🗑️  История разговора очищена");
    }

    pub fn get_history(&self) -> Vec<(String, String)> {
        self.conversation_history
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect()
    }

    pub fn get_current_model(&self) -> &str {
        &self.current_model
    }

    pub fn get_storage_stats(&self) -> String {
        if let Some(ref storage) = self.storage {
            match storage.get_stats() {
                Ok(stats) => stats,
                Err(e) => format!("Ошибка получения статистики: {}", e),
            }
        } else {
            "Хранилище недоступно".to_string()
        }
    }
}
