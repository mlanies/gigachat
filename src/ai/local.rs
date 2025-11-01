/// Локальный AI с правилами для базовых ответов
pub struct LocalAI;

impl LocalAI {
    pub fn get_response(user_input: &str) -> String {
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
}
