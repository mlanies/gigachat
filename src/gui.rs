use crate::agent::ClippyAgent;
use crate::config::Config;
use crate::tts::TextToSpeech;
use crate::talk_cloud;
use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::mpsc as std_mpsc;
use std::path::PathBuf;
use std::time::Instant;


pub struct ClippyApp {
    config: Config,
    agent: Arc<Mutex<ClippyAgent>>,
    tts: Arc<TextToSpeech>,
    messages: Vec<(String, String)>, // (sender, message)
    input_text: String,
    status: String,
    is_thinking: bool,
    response_receiver: std_mpsc::Receiver<String>,
    response_sender: std_mpsc::Sender<String>,
    clippy_texture: Option<egui::TextureHandle>,
    style_initialized: bool, // Флаг для инициализации стиля один раз
    start_time: Instant, // Время запуска приложения
    greeting_shown: bool, // Флаг, было ли показано приветственное сообщение
    window_positioned: bool, // Флаг, была ли установлена позиция окна
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
            status: "Готов к работе".to_string(),
            is_thinking: false,
            response_receiver: receiver,
            response_sender: sender,
            clippy_texture: None,
            style_initialized: false,
            start_time: Instant::now(),
            greeting_shown: false,
            window_positioned: false,
        }
    }
    
    fn load_clippy_image(&mut self, ctx: &egui::Context) {
        if self.clippy_texture.is_some() {
            return;
        }
        
        // Пробуем несколько путей для поиска изображения
        let possible_paths = vec![
            PathBuf::from("assets/clippy.png"),
            PathBuf::from("./assets/clippy.png"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/clippy.png"),
            // Fallback для обратной совместимости
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
                        
                        // Агрессивное удаление фона
                        // Анализируем края изображения (не только углы) для определения цвета фона
                        let mut edge_samples = Vec::new();
                        let width = size[0] as u32;
                        let height = size[1] as u32;
                        
                        // Берем пробы по краям изображения
                        for x in 0..width.min(10) {
                            edge_samples.push(rgba_img.get_pixel(x, 0));
                            edge_samples.push(rgba_img.get_pixel(x, height - 1));
                        }
                        for y in 0..height.min(10) {
                            edge_samples.push(rgba_img.get_pixel(0, y));
                            edge_samples.push(rgba_img.get_pixel(width - 1, y));
                        }
                        
                        // Также берем углы
                        edge_samples.push(rgba_img.get_pixel(0, 0));
                        edge_samples.push(rgba_img.get_pixel(width - 1, 0));
                        edge_samples.push(rgba_img.get_pixel(0, height - 1));
                        edge_samples.push(rgba_img.get_pixel(width - 1, height - 1));
                        
                        // Находим доминирующий цвет фона (используем модальное значение)
                        let mut color_counts = std::collections::HashMap::new();
                        for pixel in &edge_samples {
                            // Квантуем цвета для группировки похожих оттенков
                            let r = (pixel[0] / 10) * 10;
                            let g = (pixel[1] / 10) * 10;
                            let b = (pixel[2] / 10) * 10;
                            *color_counts.entry((r, g, b)).or_insert(0) += 1;
                        }
                        
                        let bg_color = color_counts.iter()
                            .max_by_key(|(_, count)| *count)
                            .map(|((r, g, b), _)| (*r as f32, *g as f32, *b as f32))
                            .unwrap_or((255.0, 255.0, 255.0));
                        
                        // Удаляем фон с использованием цветового расстояния
                        let threshold = 50.0; // Увеличенный порог для более агрессивного удаления
                        for pixel in rgba_img.pixels_mut() {
                            let r = pixel[0] as f32;
                            let g = pixel[1] as f32;
                            let b = pixel[2] as f32;
                            let a = pixel[3] as f32;
                            
                            // Если альфа уже установлена (из PNG), учитываем это
                            if a < 128.0 {
                                pixel[3] = 0;
                                continue;
                            }
                            
                            // Вычисляем расстояние до цвета фона (методом LAB для лучшего восприятия цвета)
                            let dr = r - bg_color.0;
                            let dg = g - bg_color.1;
                            let db = b - bg_color.2;
                            let distance = (dr * dr + dg * dg + db * db).sqrt();
                            
                            // Если пиксель похож на фон, делаем прозрачным
                            if distance < threshold {
                                pixel[3] = 0; // Полная прозрачность
                                continue;
                            }
                            
                            // Дополнительная проверка: очень светлые пиксели (белый фон)
                            let brightness = (r + g + b) / 3.0;
                            if brightness > 240.0 {
                                pixel[3] = 0;
                                continue;
                            }
                            
                            // Удаляем пиксели, которые очень похожи на белый
                            let white_distance = ((r - 255.0).powi(2) + (g - 255.0).powi(2) + (b - 255.0).powi(2)).sqrt();
                            if white_distance < 30.0 {
                                pixel[3] = 0;
                            }
                        }
                        
                        let pixels = rgba_img.into_raw();
                        
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            &pixels,
                        );
                        
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
                eprintln!("Ошибка чтения файла assets/clippy.png: {}", e);
            }
        }
    }

    fn send_message(&mut self, ctx: &egui::Context) {
        if self.input_text.trim().is_empty() || self.is_thinking {
            return;
        }

        let user_input = self.input_text.clone();
        self.input_text.clear();
        self.messages.push(("user".to_string(), user_input.clone()));
        self.status = "Думаю...".to_string();
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
}

impl eframe::App for ClippyApp {
    /// Возвращаем полностью прозрачный clear-color для GPU-поверхности
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Полностью прозрачная заливка (RGBA)
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Устанавливаем позицию окна в правом нижнем углу (один раз при первом запуске)
        if !self.window_positioned {
            let screen_rect = ctx.screen_rect();
            let margin = 20.0;
            let position = egui::pos2(
                screen_rect.max.x - self.config.window_width - margin,
                screen_rect.max.y - self.config.window_height - margin,
            );
            
            ctx.send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                egui::ViewportCommand::OuterPosition(position),
            );
            self.window_positioned = true;
        }
        
        // Настраиваем полностью прозрачный фон для всего приложения (один раз)
        if !self.style_initialized {
            let mut style = (*ctx.style()).clone();
            style.visuals.window_fill = egui::Color32::TRANSPARENT;
            style.visuals.panel_fill = egui::Color32::TRANSPARENT;
            style.visuals.window_stroke = egui::Stroke::NONE;
            style.visuals.faint_bg_color = egui::Color32::TRANSPARENT;
            style.visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
            ctx.set_style(style);
            self.style_initialized = true;
        }
        
        // Загружаем изображение при первой итерации
        self.load_clippy_image(ctx);
        
        // Показываем приветственное сообщение через 3 секунды после запуска
        if !self.greeting_shown && self.start_time.elapsed().as_secs() >= 3 {
            self.greeting_shown = true;
            let greeting = "Привет сообществу gigachat 👋".to_string();
            self.messages.push(("clippy".to_string(), greeting.clone()));
            
            // Озвучиваем приветствие
            let tts = Arc::clone(&self.tts);
            tokio::spawn(async move {
                if let Err(e) = tts.speak(&greeting).await {
                    eprintln!("Ошибка озвучивания: {}", e);
                }
            });
            
            ctx.request_repaint();
        }
        
        // Проверяем наличие новых ответов
        while let Ok(response) = self.response_receiver.try_recv() {
            self.messages.push(("clippy".to_string(), response.clone()));
            self.status = "Готов к работе".to_string();
            self.is_thinking = false;
            
            // Озвучиваем ответ
            let tts = Arc::clone(&self.tts);
            tokio::spawn(async move {
                if let Err(e) = tts.speak(&response).await {
                    eprintln!("Ошибка озвучивания: {}", e);
                }
            });
            
            ctx.request_repaint();
        }
        
        let mut last_image_rect: Option<egui::Rect> = None;
        
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                // Размещаем картинку справа, чтобы слева было место для облака
                // Используем right_to_left layout с выравниванием по правому краю
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    if let Some(texture) = &self.clippy_texture {
                        let size = texture.size_vec2();
                        let max_size = 200.0 * 2.0 / 3.0;
                        let scale = if size.x > max_size || size.y > max_size {
                            max_size / size.x.max(size.y)
                        } else {
                            1.0
                        };
                        
                        let image_size = egui::vec2(size.x * scale, size.y * scale);
                        
                        // Место под картинку + drag
                        let (image_rect, image_response) =
                            ui.allocate_exact_size(image_size, egui::Sense::drag());
                        
                        // Area::fixed_pos использует координаты относительно ctx.screen_rect()
                        // В CentralPanel с transparent окном clip_rect и screen_rect обычно совпадают
                        // Но для надежности преобразуем координаты
                        let clip_rect = ui.clip_rect();
                        let screen_rect = ctx.screen_rect();
                        
                        // Преобразование координат: из clip_rect в screen_rect
                        let offset = screen_rect.min - clip_rect.min;
                        let screen_image_rect = egui::Rect::from_min_size(
                            image_rect.min + offset,
                            image_size,
                        );
                        
                        // ВАЖНО: для отладки можно раскомментировать
                        // eprintln!("Image rect UI: {:?}, Screen: {:?}, Offset: {:?}", image_rect, screen_rect, offset);
                        
                        last_image_rect = Some(screen_image_rect);
                        
                        ui.painter().image(
                            texture.id(),
                            image_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                        
                        if image_response.drag_started() {
                            ctx.send_viewport_cmd_to(
                                egui::ViewportId::ROOT,
                                egui::ViewportCommand::StartDrag,
                            );
                        }
                        if image_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                        }
                        if image_response.dragged() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                        }
                    } else {
                        // Fallback для случая, когда изображение еще не загружено
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(100.0, 60.0),
                            egui::Sense::click(),
                        );
                        
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Загрузка Clippy...",
                            egui::FontId::default(),
                            egui::Color32::WHITE,
                        );
                    }
                });
            });
        
        // ПУЗЫРЬ: виджет на Foreground-слое, позиция считается от image_rect — «едет» вместе с картинкой
        if let (Some(image_rect), Some(text)) = (
            last_image_rect,
            self.messages.last()
                .filter(|(s, _)| s == "clippy")
                .map(|(_, t)| t.as_str()),
        ) {
            talk_cloud::show_talk_cloud_side(
                ctx,
                text,
                image_rect,                 // В экранных координатах
                110,                        // ~110 символов в строке
                120.0,                      // макс. высота видимой области (px)
                20.0,                       // зазор до картинки
                true,                       // prefer_left: старайся ставить слева (картинка теперь справа)
                egui::FontId::proportional(16.0),
            );
        }
        
        // Скрытый чат для работы - обрабатываем сообщения, но не показываем UI
        // Сообщения обрабатываются через hotkeys или автоматически
    }
}

