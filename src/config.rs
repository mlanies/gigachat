use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub gigachat_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub use_openai: bool,
    pub window_width: f32,
    pub window_height: f32,
    pub clippy_name: String,
    pub animation_speed: u64,
    pub system_prompt: String,
    pub google_cloud_api_key: Option<String>,
    pub google_cloud_project_id: Option<String>,
    pub gigachat_model: String,
    pub gigachat_temperature: f32,
    pub gigachat_max_tokens: i32,
}

impl Default for Config {
    fn default() -> Self {
        dotenv::dotenv().ok();

        let gigachat_api_key = env::var("GIGACHAT_API_KEY").ok();
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let use_openai = env::var("USE_OPENAI")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase() == "true";

        let google_cloud_api_key = env::var("GOOGLE_CLOUD_API_KEY").ok();
        let google_cloud_project_id = env::var("GOOGLE_CLOUD_PROJECT_ID").ok();

        let gigachat_model = env::var("GIGACHAT_MODEL")
            .unwrap_or_else(|_| "GigaChat:latest".to_string());

        let gigachat_temperature = env::var("GIGACHAT_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.7);

        let gigachat_max_tokens = env::var("GIGACHAT_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(500);

        let clippy_name = "Скрепыш".to_string();
        let system_prompt = format!(
            "Ты {}, дружелюбный персональный помощник.\n\
            Твоя цель - помогать пользователю с информацией и общением.\n\
            Возможности:\n\
            - Предоставлять информацию о погоде\n\
            - Показывать курсы валют (доллар, евро к рублю)\n\
            - Общаться на различные темы\n\
            - Обрабатывать и анализировать системные сообщения\n\
            Отвечай кратко, полезно и дружелюбно.\n\
            Твой стиль общения: профессиональный, но приветливый.",
            clippy_name
        );

        let use_openai_final = use_openai && openai_api_key.is_some();

        Self {
            gigachat_api_key,
            openai_api_key,
            use_openai: use_openai_final,
            // Увеличиваем окно чтобы было место для облака слева от картинки
            // Ширина: картинка (~133px) + зазор (20px) + облако (~200px минимум) + запас (~50px)
            window_width: 400.0,  // Увеличено чтобы поместилось облако
            window_height: 300.0 * 2.0 / 3.0, // Высота без изменений
            clippy_name,
            animation_speed: 500,
            system_prompt,
            google_cloud_api_key,
            google_cloud_project_id,
            gigachat_model,
            gigachat_temperature,
            gigachat_max_tokens,
        }
    }
}

