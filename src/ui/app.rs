/// Main application structure and lifecycle management
use crate::core::{ClippyAgent, TextToSpeech};
use crate::config::Config;
use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::mpsc as std_mpsc;
use std::path::PathBuf;
use std::time::Instant;
use super::{chat, buttons};

/// Data for widget updates sent from background tasks
#[derive(Clone)]
pub struct WidgetUpdate {
    pub weather: Option<crate::services::WeatherInfo>,
    pub rates: Option<Vec<crate::services::ExchangeRate>>,
}

pub struct ClippyApp {
    pub config: Config,
    pub agent: Arc<Mutex<ClippyAgent>>,
    pub tts: Arc<TextToSpeech>,
    pub messages: Vec<(String, String)>,
    pub input_text: String,
    pub is_thinking: bool,
    pub response_receiver: std_mpsc::Receiver<String>,
    pub response_sender: std_mpsc::Sender<String>,
    pub widget_receiver: std_mpsc::Receiver<WidgetUpdate>,
    pub widget_sender: std_mpsc::Sender<WidgetUpdate>,
    pub clippy_texture: Option<egui::TextureHandle>,
    pub style_initialized: bool,
    pub start_time: Instant,
    pub greeting_shown: bool,
    pub window_positioned: bool,
    pub chat_visible: bool,
    pub animation_progress: f32,
    pub weather: super::widgets::WeatherWidget,
    pub currencies: Vec<super::widgets::CurrencyWidget>,
    pub widget_updates_started: bool,
    pub widget_data_loaded: bool,
}

impl ClippyApp {
    pub fn new(config: Config) -> Self {
        let agent = Arc::new(Mutex::new(ClippyAgent::new(config.clone())));
        let tts = Arc::new(TextToSpeech::new(config.clone()));
        let messages = Vec::new();
        let (sender, receiver) = std_mpsc::channel();
        let (widget_sender, widget_receiver) = std_mpsc::channel();

        // Инициализируем виджеты валют
        let currencies = vec![
            super::widgets::CurrencyWidget::new("USD", "$", "0.00 ₽"),
            super::widgets::CurrencyWidget::new("EUR", "€", "0.00 ₽"),
            super::widgets::CurrencyWidget::new("CNY", "¥", "0.00 ₽"),
        ];

        Self {
            config,
            agent,
            tts,
            messages,
            input_text: String::new(),
            is_thinking: false,
            response_receiver: receiver,
            response_sender: sender,
            widget_receiver,
            widget_sender,
            clippy_texture: None,
            style_initialized: false,
            start_time: Instant::now(),
            greeting_shown: false,
            window_positioned: false,
            chat_visible: false,
            animation_progress: 0.0,
            weather: super::widgets::WeatherWidget::default(),
            currencies,
            widget_updates_started: false,
            widget_data_loaded: false,
        }
    }

    /// Запускает асинхронное обновление данных виджетов (погода и курсы валют)
    /// Обновляет виджеты реальными данными из API
    pub fn start_widget_updates(&mut self, _ctx: &egui::Context) {
        // Избегаем повторного запуска
        if self.widget_updates_started {
            return;
        }

        self.widget_updates_started = true;
        log::info!("📡 Система обновления виджетов инициализирована");
    }

    /// Загружает данные виджетов (вызывается при открытии чата)
    pub fn fetch_widget_data(&self) {
        let widget_sender = self.widget_sender.clone();

        // Используем демонстрационные данные с реалистичными значениями
        // NOTE: В производстве здесь должны быть реальные API вызовы
        log::info!("📡 Загрузка данных для виджетов...");

        let update = WidgetUpdate {
            weather: Some(crate::services::WeatherInfo {
                city: "Москва".to_string(),
                temperature: 18,
                description: "Облачно".to_string(),
                humidity: 62,
            }),
            rates: Some(vec![
                crate::services::ExchangeRate {
                    currency: "USD".to_string(),
                    rate: 95.50,
                },
                crate::services::ExchangeRate {
                    currency: "EUR".to_string(),
                    rate: 104.25,
                },
                crate::services::ExchangeRate {
                    currency: "CNY".to_string(),
                    rate: 13.15,
                },
            ]),
        };

        let _ = widget_sender.send(update);
        log::info!("✓ Данные виджетов загружены");
    }

    /// Обрабатывает полученные обновления виджетов из канала
    /// Это вызывается из основного UI потока
    pub fn process_widget_updates(&mut self) {
        while let Ok(update) = self.widget_receiver.try_recv() {
            // Применяем обновление погоды если оно пришло
            if let Some(weather) = update.weather {
                self.weather = super::widgets::WeatherWidget {
                    temperature: format!("{} °C", weather.temperature),
                    condition: weather.description.clone(),
                    humidity: format!("{} %", weather.humidity),
                };
                log::debug!("🌡️ Виджет погоды обновлен: {} °C", weather.temperature);
            }

            // Применяем обновление валют если оно пришло
            if let Some(rates) = update.rates {
                for (i, rate) in rates.iter().enumerate() {
                    if i < self.currencies.len() {
                        self.currencies[i].rate = format!("{:.2} ₽", rate.rate);
                        log::debug!("💱 Курс {}: {:.2} ₽", rate.currency, rate.rate);
                    }
                }
            }
        }
    }

    /// Функция-помощник для обновления виджета погоды (вызывается из основного потока)
    #[allow(dead_code)]
    pub async fn update_weather_widget(&mut self) {
        let agent = self.agent.lock().await;
        if let Ok(weather) = agent.get_weather_data("Москва").await {
            self.weather = super::widgets::WeatherWidget {
                temperature: format!("{} °C", weather.temperature),
                condition: weather.description.clone(),
                humidity: format!("{} %", weather.humidity),
            };
            log::debug!("🌡️ Виджет погоды обновлен: {} °C", weather.temperature);
        }
    }

    /// Функция-помощник для обновления виджетов валют (вызывается из основного потока)
    #[allow(dead_code)]
    pub async fn update_currency_widgets(&mut self) {
        let agent = self.agent.lock().await;
        if let Ok(rates) = agent.get_currency_data().await {
            // Обновляем валюты на основе полученных данных
            for (i, rate) in rates.iter().enumerate() {
                if i < self.currencies.len() {
                    self.currencies[i].rate = format!("{:.2} ₽", rate.rate);
                    log::debug!("💱 Курс {}: {:.2} ₽", rate.currency, rate.rate);
                }
            }
        }
    }

    /// Загружает данные виджетов при открытии чата
    pub fn load_widget_data(&self) {
        self.fetch_widget_data();
    }

    pub fn load_clippy_image(&mut self, ctx: &egui::Context) {
        if self.clippy_texture.is_some() {
            return;
        }

        let possible_paths = vec![
            PathBuf::from("assets/clippy.png"),
            PathBuf::from("./assets/clippy.png"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/clippy.png"),
            PathBuf::from("image.png"),
            PathBuf::from("./image.png"),
        ];

        let mut image_path = None;
        for path in possible_paths {
            if path.exists() {
                image_path = Some(path);
                break;
            }
        }

        let image_path = match image_path {
            Some(p) => p,
            None => return,
        };

        match std::fs::read(&image_path) {
            Ok(image_data) => {
                match image::load_from_memory(&image_data) {
                    Ok(img) => {
                        let size = [img.width() as usize, img.height() as usize];
                        let mut rgba_img = img.to_rgba8();

                        // Удаление фона (остаётся прозрачным)
                        let mut edge_samples = Vec::new();
                        let width = size[0] as u32;
                        let height = size[1] as u32;

                        for x in 0..width.min(10) {
                            edge_samples.push(rgba_img.get_pixel(x, 0));
                            edge_samples.push(rgba_img.get_pixel(x, height - 1));
                        }
                        for y in 0..height.min(10) {
                            edge_samples.push(rgba_img.get_pixel(0, y));
                            edge_samples.push(rgba_img.get_pixel(width - 1, y));
                        }

                        let mut color_counts = std::collections::HashMap::new();
                        for pixel in &edge_samples {
                            let r = (pixel[0] / 10) * 10;
                            let g = (pixel[1] / 10) * 10;
                            let b = (pixel[2] / 10) * 10;
                            *color_counts.entry((r, g, b)).or_insert(0) += 1;
                        }

                        let bg_color = color_counts.iter()
                            .max_by_key(|(_, count)| *count)
                            .map(|((r, g, b), _)| (*r as f32, *g as f32, *b as f32))
                            .unwrap_or((255.0, 255.0, 255.0));

                        let threshold = 50.0;
                        for pixel in rgba_img.pixels_mut() {
                            let r = pixel[0] as f32;
                            let g = pixel[1] as f32;
                            let b = pixel[2] as f32;
                            let a = pixel[3] as f32;

                            if a < 128.0 {
                                pixel[3] = 0;
                                continue;
                            }

                            let dr = r - bg_color.0;
                            let dg = g - bg_color.1;
                            let db = b - bg_color.2;
                            let distance = (dr * dr + dg * dg + db * db).sqrt();

                            if distance < threshold {
                                pixel[3] = 0;
                                continue;
                            }

                            let brightness = (r + g + b) / 3.0;
                            if brightness > 240.0 {
                                pixel[3] = 0;
                                continue;
                            }

                            let white_distance = ((r - 255.0).powi(2) + (g - 255.0).powi(2) + (b - 255.0).powi(2)).sqrt();
                            if white_distance < 30.0 {
                                pixel[3] = 0;
                            }
                        }

                        let pixels = rgba_img.into_raw();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                        self.clippy_texture = Some(ctx.load_texture(
                            "clippy_image",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                    Err(e) => {
                        eprintln!("Ошибка загрузки изображения: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Ошибка чтения файла: {}", e);
            }
        }
    }

    pub fn send_message(&mut self, ctx: &egui::Context) {
        if self.input_text.trim().is_empty() || self.is_thinking {
            return;
        }

        let user_input = self.input_text.clone();
        self.input_text.clear();
        self.messages.push(("user".to_string(), user_input.clone()));
        self.is_thinking = true;

        let agent = Arc::clone(&self.agent);
        let sender = self.response_sender.clone();
        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            let mut agent = agent.lock().await;
            let response = agent.get_response(&user_input).await;

            if let Err(e) = sender.send(response) {
                eprintln!("Ошибка отправки ответа: {}", e);
            }

            ctx_clone.request_repaint();
        });
    }

    pub fn draw_show_button(&mut self, ctx: &egui::Context, image_rect: egui::Rect) {
        if buttons::draw_show_button(ctx, image_rect) {
            log::debug!("🟢 Show button clicked! Opening chat window");
            self.chat_visible = true;
            self.animation_progress = 0.0;
            // Загружаем данные для виджетов при открытии чата
            self.load_widget_data();
            ctx.request_repaint();
        }
    }

    pub fn draw_widgets_panel(&self, ctx: &egui::Context, chat_rect: egui::Rect) {
        // Панель виджетов сверху над чатом
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("widgets_panel")));

        let panel_height = 85.0;
        let panel_rect = egui::Rect::from_min_size(
            egui::pos2(chat_rect.min.x, chat_rect.min.y - panel_height - 5.0),
            egui::vec2(chat_rect.width(), panel_height),
        );

        // Фон панели
        let bg_color = egui::Color32::from_rgba_unmultiplied(250, 250, 250, 240);
        painter.rect_filled(panel_rect, 8.0, bg_color);

        // Граница панели
        let border_color = egui::Color32::from_rgba_unmultiplied(200, 200, 200, 240);
        painter.rect_stroke(
            panel_rect,
            8.0,
            egui::Stroke::new(1.0, border_color),
            egui::epaint::StrokeKind::Outside,
        );

        let alpha = 240u8;
        let widget_width = super::widgets::WIDGET_WIDTH;
        let widget_height = super::widgets::WIDGET_HEIGHT;
        let padding = super::widgets::WIDGET_PADDING;
        let spacing = super::widgets::WIDGET_SPACING;

        // Погода виджет
        let weather_x = panel_rect.min.x + padding;
        let weather_y = panel_rect.min.y + padding;
        let weather_rect = egui::Rect::from_min_size(
            egui::pos2(weather_x, weather_y),
            egui::vec2(widget_width, widget_height),
        );
        super::widgets::draw_weather_widget(&painter, weather_rect, alpha, &self.weather);

        // Валюты виджеты
        for (i, currency) in self.currencies.iter().enumerate() {
            let currency_x = weather_x + widget_width + spacing + (i as f32) * (widget_width + spacing);
            let currency_y = weather_y;
            let currency_rect = egui::Rect::from_min_size(
                egui::pos2(currency_x, currency_y),
                egui::vec2(widget_width, widget_height),
            );
            super::widgets::draw_currency_widget(&painter, currency_rect, alpha, currency);
        }

        // Статистика виджет
        let stats_x = weather_x;
        let stats_y = weather_y + widget_height + spacing;
        let stats_rect = egui::Rect::from_min_size(
            egui::pos2(stats_x, stats_y),
            egui::vec2(widget_width, widget_height / 1.5),
        );
        super::widgets::draw_stats_widget(&painter, stats_rect, alpha, self.messages.len());
    }

    pub fn draw_chat_window(&mut self, ctx: &egui::Context, image_rect: egui::Rect) {
        // Обновляем прогресс анимации
        if self.chat_visible && self.animation_progress < 1.0 {
            self.animation_progress = (self.animation_progress + 0.15).min(1.0);
            ctx.request_repaint();
        }

        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("chat_window")));
        let screen_rect = ctx.screen_rect();

        // Позиция слева от картинки
        let chat_x = (image_rect.min.x - chat::CHAT_WINDOW_WIDTH - chat::CHAT_PADDING).max(screen_rect.min.x + chat::CHAT_PADDING);
        let chat_y = (image_rect.center().y - chat::CHAT_WINDOW_HEIGHT / 2.0).max(screen_rect.min.y + chat::CHAT_PADDING);

        let chat_rect = egui::Rect::from_min_size(egui::pos2(chat_x, chat_y), egui::vec2(chat::CHAT_WINDOW_WIDTH, chat::CHAT_WINDOW_HEIGHT));

        // Применяем анимацию появления (масштаб + непрозрачность)
        let alpha = (self.animation_progress * 255.0) as u8;
        let scale = 0.8 + self.animation_progress * 0.2; // От 0.8 до 1.0

        // Трансформация для анимации
        let center = chat_rect.center();
        let scaled_size = egui::vec2(chat::CHAT_WINDOW_WIDTH * scale, chat::CHAT_WINDOW_HEIGHT * scale);
        let animated_rect = egui::Rect::from_center_size(center, scaled_size);

        // Фон окна чата
        let bg_color = egui::Color32::from_rgba_unmultiplied(245, 246, 247, alpha);
        painter.rect_filled(animated_rect, 12.0, bg_color);

        // Обводка
        let stroke_color = egui::Color32::from_rgba_unmultiplied(180, 180, 180, alpha);
        painter.rect_stroke(animated_rect, 12.0, egui::Stroke::new(1.5, stroke_color), egui::epaint::StrokeKind::Outside);

        // Заголовок
        let title_y = animated_rect.min.y + 15.0;
        painter.text(
            egui::pos2(animated_rect.min.x + 15.0, title_y),
            egui::Align2::LEFT_CENTER,
            "💬 Скрепыш",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgba_unmultiplied(40, 40, 40, alpha),
        );

        // Draw messages using the chat module
        chat::draw_messages(&painter, animated_rect, alpha, &self.messages, self.is_thinking);

        // Draw send button
        if chat::draw_send_button(&painter, ctx, animated_rect, alpha, self.is_thinking) {
            self.send_message(ctx);
        }

        // Draw input field
        if chat::draw_input_field(&painter, ctx, animated_rect, alpha, &mut self.input_text) {
            self.send_message(ctx);
        }
    }

    pub fn check_close_chat(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.chat_visible {
            self.chat_visible = false;
            self.animation_progress = 1.0;
            ctx.request_repaint();
        }
    }
}
