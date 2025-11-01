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

pub struct ClippyApp {
    pub config: Config,
    pub agent: Arc<Mutex<ClippyAgent>>,
    pub tts: Arc<TextToSpeech>,
    pub messages: Vec<(String, String)>,
    pub input_text: String,
    pub is_thinking: bool,
    pub response_receiver: std_mpsc::Receiver<String>,
    pub response_sender: std_mpsc::Sender<String>,
    pub clippy_texture: Option<egui::TextureHandle>,
    pub style_initialized: bool,
    pub start_time: Instant,
    pub greeting_shown: bool,
    pub window_positioned: bool,
    pub chat_visible: bool,
    pub animation_progress: f32,
}

impl ClippyApp {
    pub fn new(config: Config) -> Self {
        let agent = Arc::new(Mutex::new(ClippyAgent::new(config.clone())));
        let tts = Arc::new(TextToSpeech::new(config.clone()));
        let messages = Vec::new();
        let (sender, receiver) = std_mpsc::channel();

        Self {
            config,
            agent,
            tts,
            messages,
            input_text: String::new(),
            is_thinking: false,
            response_receiver: receiver,
            response_sender: sender,
            clippy_texture: None,
            style_initialized: false,
            start_time: Instant::now(),
            greeting_shown: false,
            window_positioned: false,
            chat_visible: false,
            animation_progress: 0.0,
        }
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

    pub fn draw_send_button(&mut self, ctx: &egui::Context, image_rect: egui::Rect) {
        log::debug!("🔘 Drawing send button at position: {:?}", image_rect.max);
        if buttons::draw_send_button(ctx, image_rect) {
            log::debug!("🔘 Send button clicked!");
            if !self.chat_visible {
                self.chat_visible = true;
                self.animation_progress = 0.0;
                ctx.request_repaint();
            } else {
                self.send_message(ctx);
                ctx.request_repaint();
            }
        }
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
