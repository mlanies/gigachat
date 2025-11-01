use anyhow::Result;
use chrono::Local;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Структура для хранения одного сообщения в БД
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: i32,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub model: String,
    pub timestamp: String,
}

/// Сервис для работы с хранилищем разговоров в SQLite
pub struct SQLiteStorage {
    conn: Connection,
    session_id: String,
}

impl SQLiteStorage {
    /// Создает или открывает базу данных
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        // Используем путь по умолчанию если не указан
        let db_path = db_path.unwrap_or_else(|| {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(".config/clippy/clippy.db")
        });

        // Создаем директорию если её нет
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        log::info!("📦 SQLite БД открыта: {}", db_path.display());

        // Создаем таблицу если её нет
        Self::init_schema(&conn)?;

        // Генерируем уникальный session_id
        let session_id = Uuid::new_v4().to_string();
        log::info!("📍 Session ID: {}", session_id);

        Ok(Self { conn, session_id })
    }

    /// Инициализирует схему БД
    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                model TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        // Создаем индекс для быстрого поиска по session_id
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_id ON conversations(session_id)",
            [],
        )?;

        log::info!("✓ Схема БД инициализирована");
        Ok(())
    }

    /// Сохраняет сообщение в БД
    pub fn save_message(&self, role: &str, content: &str, model: &str) -> Result<()> {
        let timestamp = Local::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO conversations (session_id, role, content, model, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&self.session_id, role, content, model, &timestamp],
        )?;

        log::debug!("💾 Сохранено сообщение: {} - {}", role, &content[..content.len().min(50)]);
        Ok(())
    }

    /// Загружает историю разговора из текущей сессии
    pub fn load_session_history(&self) -> Result<Vec<StoredMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, role, content, model, timestamp 
             FROM conversations 
             WHERE session_id = ?1 
             ORDER BY id ASC",
        )?;

        let messages = stmt.query_map(params![&self.session_id], |row| {
            Ok(StoredMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                model: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;

        let mut result = Vec::new();
        for msg in messages {
            result.push(msg?);
        }

        log::info!("📖 Загружено {} сообщений из сессии", result.len());
        Ok(result)
    }

    /// Загружает последние N сессий
    pub fn load_recent_sessions(&self, limit: usize) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, COUNT(*) as count 
             FROM conversations 
             GROUP BY session_id 
             ORDER BY MAX(id) DESC 
             LIMIT ?1",
        )?;

        let sessions = stmt.query_map(params![limit as i32], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })?;

        let mut result = Vec::new();
        for session in sessions {
            result.push(session?);
        }

        log::info!("📚 Загружено {} сессий", result.len());
        Ok(result)
    }

    /// Очищает историю текущей сессии
    pub fn clear_session_history(&self) -> Result<()> {
        let affected = self.conn.execute(
            "DELETE FROM conversations WHERE session_id = ?1",
            params![&self.session_id],
        )?;

        log::warn!("🗑️  Очищено {} сообщений из текущей сессии", affected);
        Ok(())
    }

    /// Очищает всю историю (осторожно!)
    pub fn clear_all_history(&self) -> Result<()> {
        let affected = self.conn.execute("DELETE FROM conversations", [])?;
        log::warn!("🗑️  ⚠️  Очищено {} сообщений ИЗ ВСЕй ИСТОРИИ", affected);
        Ok(())
    }

    /// Возвращает количество сообщений в БД
    pub fn message_count(&self) -> Result<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM conversations",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Возвращает информацию о статистике БД
    pub fn get_stats(&self) -> Result<String> {
        let total: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM conversations",
            [],
            |row| row.get(0),
        )?;

        let sessions: usize = self.conn.query_row(
            "SELECT COUNT(DISTINCT session_id) FROM conversations",
            [],
            |row| row.get(0),
        )?;

        let current_session: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM conversations WHERE session_id = ?1",
            params![&self.session_id],
            |row| row.get(0),
        )?;

        Ok(format!(
            "📊 Статистика БД: {} всего, {} сессий, {} в текущей",
            total, sessions, current_session
        ))
    }
}

