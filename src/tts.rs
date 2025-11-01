use crate::config::Config;

pub struct TextToSpeech {
    config: Config,
}

impl TextToSpeech {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub async fn speak(&self, text: &str) -> Result<(), String> {
        // Используем Google Cloud Text-to-Speech API
        if let Some(api_key) = &self.config.google_cloud_api_key {
            self.speak_google_cloud(text, api_key).await
        } else {
            // Fallback: используем системный TTS на macOS
            self.speak_system(text)
        }
    }
    
    async fn speak_google_cloud(&self, text: &str, api_key: &str) -> Result<(), String> {
        let client = reqwest::Client::new();
        
        let project_id = self.config.google_cloud_project_id.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("clippy-tts");
        
        let url = format!(
            "https://texttospeech.googleapis.com/v1/projects/{}/locations/global:synthesize",
            project_id
        );
        
        let request_body = serde_json::json!({
            "input": {
                "text": text
            },
            "voice": {
                "languageCode": "ru-RU",
                "name": "ru-RU-Wavenet-D",
                "ssmlGender": "MALE"
            },
            "audioConfig": {
                "audioEncoding": "MP3"
            }
        });
        
        match client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(audio_content) = json.get("audioContent").and_then(|v| v.as_str()) {
                                // Декодируем base64 и воспроизводим
                                use base64::Engine;
                                match base64::engine::general_purpose::STANDARD.decode(audio_content) {
                                    Ok(audio_data) => {
                                        // Сохраняем во временный файл и воспроизводим
                                        self.play_audio(&audio_data).await
                                    }
                                    Err(e) => Err(format!("Ошибка декодирования аудио: {}", e)),
                                }
                            } else {
                                Err("Не найден audioContent в ответе".to_string())
                            }
                        }
                        Err(e) => Err(format!("Ошибка парсинга ответа: {}", e)),
                    }
                } else {
                    Err(format!("Ошибка API: {}", response.status()))
                }
            }
            Err(e) => Err(format!("Ошибка запроса: {}", e)),
        }
    }
    
    fn speak_system(&self, text: &str) -> Result<(), String> {
        // macOS системный TTS
        use std::process::Command;
        
        match Command::new("say")
            .arg(text)
            .output()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Ошибка системного TTS: {}", e)),
        }
    }
    
    async fn play_audio(&self, audio_data: &[u8]) -> Result<(), String> {
        // Используем системный проигрыватель для воспроизведения MP3
        use std::process::Command;
        use std::fs;
        
        let temp_file = format!("/tmp/clippy_audio_{}.mp3", std::process::id());
        
        match fs::write(&temp_file, audio_data) {
            Ok(_) => {
                let result = Command::new("afplay")
                    .arg(&temp_file)
                    .output();
                
                // Удаляем временный файл
                let _ = fs::remove_file(&temp_file);
                
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Ошибка воспроизведения: {}", e)),
                }
            }
            Err(e) => Err(format!("Ошибка записи временного файла: {}", e)),
        }
    }
}

