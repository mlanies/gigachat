/// Модуль для работы с хранилищем разговоров
/// TODO: Реализовать SQLite интеграцию
/// Сохранять:
/// - user message
/// - AI response
/// - timestamp
/// - model used
/// - session id

pub struct StorageService;

impl StorageService {
    pub fn new() -> Self {
        Self
    }

    /// Сохраняет разговор в хранилище
    pub async fn save_message(_role: &str, _content: &str) -> anyhow::Result<()> {
        // TODO: Реализовать
        Ok(())
    }

    /// Загружает историю разговоров
    pub async fn load_history() -> anyhow::Result<Vec<(String, String)>> {
        // TODO: Реализовать
        Ok(Vec::new())
    }

    /// Очищает историю
    pub async fn clear_history() -> anyhow::Result<()> {
        // TODO: Реализовать
        Ok(())
    }
}
