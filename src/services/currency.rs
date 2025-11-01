use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub currency: String,
    pub rate: f32,
}

/// Сервис для получения курсов валют
pub struct CurrencyService {
    http_client: reqwest::Client,
}

impl CurrencyService {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    /// Получает курс валют USD и EUR к RUB
    /// TODO: Интегрировать реальный API для курсов валют (Open Exchange Rates, etc)
    pub async fn get_rates(&self) -> Result<Vec<ExchangeRate>> {
        Ok(vec![
            ExchangeRate {
                currency: "USD".to_string(),
                rate: 90.0,
            },
            ExchangeRate {
                currency: "EUR".to_string(),
                rate: 98.0,
            },
        ])
    }

    /// Форматирует информацию о курсах в читаемый текст
    pub async fn format_rates_info(&self) -> Result<String> {
        let rates = self.get_rates().await?;
        let mut result = "Курсы валют к рублю:\n".to_string();

        for rate in rates {
            result.push_str(&format!("• {} = {:.2} ₽\n", rate.currency, rate.rate));
        }

        Ok(result)
    }
}
