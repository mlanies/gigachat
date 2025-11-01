use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub currency: String,
    pub rate: f32,
}

// Response structure for Exchangerate-API
#[derive(Debug, Deserialize)]
struct ExchangerateApiResponse {
    rates: HashMap<String, f32>,
}

/// Сервис для получения курсов валют через Exchangerate-API
pub struct CurrencyService {
    http_client: reqwest::Client,
    base_currency: String,
}

impl CurrencyService {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_currency: "RUB".to_string(),
        }
    }

    /// Получает курсы валют USD, EUR, GBP и JPY к RUB
    /// Использует бесплатный API exchangerate-api.com
    pub async fn get_rates(&self) -> Result<Vec<ExchangeRate>> {
        let target_currencies = vec!["USD", "EUR", "GBP", "JPY", "CNY", "CHF"];

        // Используем exchangerate-api.com для получения курсов
        // Endpoint: latest/{base_currency}
        let url = format!(
            "https://api.exchangerate-api.com/v4/latest/{}",
            self.base_currency
        );

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            log::warn!("⚠️ Ошибка получения курсов валют: {}", response.status());
            // Fallback на приблизительные значения если API недоступен
            return Ok(vec![
                ExchangeRate {
                    currency: "USD".to_string(),
                    rate: 90.0,
                },
                ExchangeRate {
                    currency: "EUR".to_string(),
                    rate: 98.0,
                },
                ExchangeRate {
                    currency: "GBP".to_string(),
                    rate: 113.0,
                },
                ExchangeRate {
                    currency: "JPY".to_string(),
                    rate: 0.60,
                },
            ]);
        }

        let api_response: ExchangerateApiResponse = response.json().await?;

        let mut rates = Vec::new();
        for currency in target_currencies {
            if let Some(&rate) = api_response.rates.get(currency) {
                rates.push(ExchangeRate {
                    currency: currency.to_string(),
                    rate: rate as f32,
                });
            }
        }

        if rates.is_empty() {
            log::warn!("⚠️ Не удалось получить курсы валют из API");
        }

        Ok(rates)
    }

    /// Форматирует информацию о курсах в читаемый текст
    pub async fn format_rates_info(&self) -> Result<String> {
        let rates = self.get_rates().await?;
        let mut result = "💱 Курсы валют к рублю (RUB):\n".to_string();

        for rate in rates {
            let symbol = match rate.currency.as_str() {
                "USD" => "$",
                "EUR" => "€",
                "GBP" => "£",
                "JPY" => "¥",
                "CNY" => "¥",
                "CHF" => "₣",
                _ => "",
            };

            if rate.rate < 1.0 {
                result.push_str(&format!("• {} {}: {:.4} ₽\n", symbol, rate.currency, rate.rate));
            } else {
                result.push_str(&format!("• {} {}: {:.2} ₽\n", symbol, rate.currency, rate.rate));
            }
        }

        Ok(result)
    }
}
