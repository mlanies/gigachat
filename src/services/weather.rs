use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub city: String,
    pub temperature: i32,
    pub description: String,
    pub humidity: i32,
}

/// Сервис для получения информации о погоде
pub struct WeatherService {
    http_client: reqwest::Client,
}

impl WeatherService {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    /// Получает информацию о погоде для города
    /// TODO: Интегрировать реальный OpenWeatherMap API
    pub async fn get_weather(&self, city: &str) -> Result<WeatherInfo> {
        Ok(WeatherInfo {
            city: city.to_string(),
            temperature: 15,
            description: "Облачно".to_string(),
            humidity: 70,
        })
    }

    /// Форматирует информацию о погоде в читаемый текст
    pub async fn format_weather_info(&self, city: &str) -> Result<String> {
        let weather = self.get_weather(city).await?;
        let result = format!(
            "Погода в городе {}:\n• Температура: {}°C\n• Условия: {}\n• Влажность: {}%",
            weather.city, weather.temperature, weather.description, weather.humidity
        );
        Ok(result)
    }
}
