use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub city: String,
    pub temperature: i32,
    pub description: String,
    pub humidity: i32,
}

// Response structures for Open-Meteo API
#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: f32,
    relative_humidity_2m: i32,
    weather_code: i32,
}

// Geocoding response for city coordinates
#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodingResult {
    latitude: f32,
    longitude: f32,
    name: String,
    admin1: Option<String>,
    country: Option<String>,
}

/// Сервис для получения информации о погоде через Open-Meteo API
pub struct WeatherService {
    http_client: reqwest::Client,
}

impl WeatherService {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    /// Преобразует WMO код погоды в описание
    fn weather_code_to_description(&self, code: i32) -> String {
        match code {
            0 => "Ясно".to_string(),
            1 | 2 => "Облачно".to_string(),
            3 => "Пасмурно".to_string(),
            45 | 48 => "Туман".to_string(),
            51 | 53 | 55 => "Морось".to_string(),
            61 | 63 | 65 => "Дождь".to_string(),
            71 | 73 | 75 => "Снег".to_string(),
            77 => "Снег".to_string(),
            80 | 82 | 81 => "Ливень".to_string(),
            85 | 86 => "Снегопад".to_string(),
            95 | 96 | 99 => "Гроза".to_string(),
            _ => "Неизвестно".to_string(),
        }
    }

    /// Получает координаты города через Geocoding API
    async fn get_city_coordinates(&self, city: &str) -> Result<(f32, f32, String)> {
        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=ru&format=json",
            urlencoding::encode(city)
        );

        let response = self.http_client.get(&url).send().await?;
        let geo_response: GeocodingResponse = response.json().await?;

        if let Some(mut results) = geo_response.results {
            if !results.is_empty() {
                let result = results.remove(0);
                Ok((result.latitude, result.longitude, result.name))
            } else {
                Err(anyhow::anyhow!("Город '{}' не найден", city))
            }
        } else {
            Err(anyhow::anyhow!("Ошибка при поиске города '{}'", city))
        }
    }

    /// Получает информацию о погоде для города через Open-Meteo API
    pub async fn get_weather(&self, city: &str) -> Result<WeatherInfo> {
        // Получаем координаты города
        let (latitude, longitude, city_name) = self.get_city_coordinates(city).await?;

        // Запрашиваем данные погоды
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code&temperature_unit=celsius&timezone=auto",
            latitude, longitude
        );

        let response = self.http_client.get(&url).send().await?;
        let weather_response: OpenMeteoResponse = response.json().await?;

        let current = weather_response.current;
        let description = self.weather_code_to_description(current.weather_code);

        Ok(WeatherInfo {
            city: city_name,
            temperature: current.temperature_2m as i32,
            description,
            humidity: current.relative_humidity_2m,
        })
    }

    /// Форматирует информацию о погоде в читаемый текст
    pub async fn format_weather_info(&self, city: &str) -> Result<String> {
        let weather = self.get_weather(city).await?;
        let result = format!(
            "🌍 Погода в городе {}:\n• 🌡️ Температура: {}°C\n• ☁️ Условия: {}\n• 💧 Влажность: {}%",
            weather.city, weather.temperature, weather.description, weather.humidity
        );
        Ok(result)
    }
}
