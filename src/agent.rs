use crate::config::Config;
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
}

impl ClippyAgent {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            conversation_history: VecDeque::new(),
        }
    }

    pub async fn get_response(&mut self, user_input: &str) -> String {
        if user_input.trim().is_empty() {
            return "Чем могу помочь?".to_string();
        }

        if self.config.use_openai {
            self.get_openai_response(user_input).await
        } else {
            self.get_local_response(user_input)
        }
    }

    async fn get_openai_response(&mut self, user_input: &str) -> String {
        let api_key = match &self.config.openai_api_key {
            Some(key) => key,
            None => return "Ошибка: API ключ не настроен".to_string(),
        };

        let mut messages = vec![OpenAIMessage {
            role: "system".to_string(),
            content: self.config.system_prompt.clone(),
        }];

        for msg in self.conversation_history.iter().take(10) {
            messages.push(OpenAIMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages,
            temperature: 0.7,
            max_tokens: 200,
        };

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await;

        match response {
            Ok(resp) => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                            if let Some(first_choice) = choices.first() {
                                if let Some(content) = first_choice
                                    .get("message")
                                    .and_then(|m| m.get("content"))
                                    .and_then(|c| c.as_str())
                                {
                                    let assistant_response = content.to_string();
                                    
                                    self.conversation_history.push_back(Message {
                                        role: "user".to_string(),
                                        content: user_input.to_string(),
                                    });
                                    self.conversation_history.push_back(Message {
                                        role: "assistant".to_string(),
                                        content: assistant_response.clone(),
                                    });
                                    
                                    return assistant_response;
                                }
                            }
                        }
                        "Ошибка при обработке ответа от OpenAI".to_string()
                    }
                    Err(e) => format!("Ошибка парсинга ответа: {}", e),
                }
            }
            Err(e) => format!("Ошибка запроса к OpenAI: {}", e),
        }
    }

    fn get_local_response(&self, user_input: &str) -> String {
        let user_input_lower = user_input.to_lowercase();
        
        let greetings = vec!["привет", "здравствуй", "добрый день", "добрый вечер", "доброе утро"];
        let farewells = vec!["пока", "до свидания", "увидимся", "прощай"];
        let help_words = vec!["помощь", "помоги", "как", "что", "помочь"];

        if greetings.iter().any(|&word| user_input_lower.contains(word)) {
            return format!("Привет! Я {}, твой помощник. Чем могу помочь?", self.config.clippy_name);
        }

        if farewells.iter().any(|&word| user_input_lower.contains(word)) {
            return "До свидания! Удачи!".to_string();
        }

        if help_words.iter().any(|&word| user_input_lower.contains(word)) {
            return "Я могу помочь с:\n- Ответами на вопросы\n- Подсказками по работе с компьютером\n- Советами по продуктивности\n- И многим другим!\n\nПросто спроси меня о чем угодно!".to_string();
        }

        if user_input_lower.contains("время") || user_input_lower.contains("час") {
            use std::time::SystemTime;
            let now = SystemTime::now();
            let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
            let hours = (since_epoch.as_secs() / 3600 % 24) as u32;
            let minutes = (since_epoch.as_secs() / 60 % 60) as u32;
            return format!("Сейчас примерно {:02}:{:02}", hours, minutes);
        }

        if user_input_lower.contains("погода") {
            return "Я не имею доступа к данным о погоде, но рекомендую проверить погодное приложение.".to_string();
        }

        format!(
            "Интересный вопрос! К сожалению, мои локальные возможности ограничены. \
            Для полной функциональности настройте OpenAI API в файле .env\n\n\
            Но я всегда готов помочь с базовыми вопросами!"
        )
    }

    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }
}

