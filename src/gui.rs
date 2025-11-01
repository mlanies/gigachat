use crate::core::ClippyAgent;
use crate::config::Config;
use crate::core::TextToSpeech;
use crate::ui;
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
    cloud_visible: bool, // Флаг видимости облака
    storage_stats: String, // Статистика хранилища
    show_clear_confirmation: bool, // Показать диалог подтверждения очистки
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
            cloud_visible: true,
            storage_stats: String::new(),
            show_clear_confirmation: false,
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

    /// Обновляет статистику хранилища из агента
    fn update_storage_stats(&mut self) {
        let agent = Arc::clone(&self.agent);

        let sender = self.response_sender.clone();
        tokio::spawn(async move {
            let agent = agent.lock().await;
            let stats = agent.get_storage_stats();
            // Отправляем статистику как специальное сообщение (не используется, но можем позже)
            let _ = sender.send(format!("[stats: {}]", stats));
        });
    }

    /// Очищает историю разговора из агента
    fn clear_agent_history(&mut self) {
        let agent = Arc::clone(&self.agent);
        tokio::spawn(async move {
            let mut agent = agent.lock().await;
            agent.clear_history();
        });
        self.messages.clear();
        self.show_clear_confirmation = false;
    }

    /// Рисует кнопку закрытия облака (маленький белый круг сверху-слева) и кнопку очистки истории
    fn draw_close_button(&mut self, ctx: &egui::Context, cloud_rect: egui::Rect) {
        let button_size = 16.0; // размер кнопки (маленький)
        let padding = 6.0; // отступ от края облака

        // Позиция: сверху-слева облака
        let button_pos = egui::pos2(
            cloud_rect.min.x + padding + button_size / 2.0,
            cloud_rect.min.y + padding + button_size / 2.0,
        );

        let button_rect = egui::Rect::from_center_size(button_pos, egui::vec2(button_size + 4.0, button_size + 4.0));

        // Проверяем нажатие на кнопку в интерактивной зоне
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("close_button")));

        // Получаем позицию мыши
        if let Some(mouse_pos) = ctx.input(|i| i.pointer.latest_pos()) {
            // Проверяем, находится ли мышь над кнопкой
            if button_rect.contains(mouse_pos) {
                // Меняем курсор на pointer
                ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);

                // Проверяем нажатие левой кнопки мыши
                if ctx.input(|i| i.pointer.primary_clicked()) {
                    self.cloud_visible = false;
                    ctx.request_repaint();
                }

                // Рисуем кнопку в состоянии hover (слегка более насыщенная обводка)
                painter.circle_filled(button_pos, button_size / 2.0, egui::Color32::WHITE);
                painter.circle_stroke(
                    button_pos,
                    button_size / 2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
                );
            } else {
                // Рисуем кнопку в нормальном состоянии
                painter.circle_filled(button_pos, button_size / 2.0, egui::Color32::WHITE);
                painter.circle_stroke(
                    button_pos,
                    button_size / 2.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 150, 150)),
                );
            }
        } else {
            // Рисуем кнопку в нормальном состоянии
            painter.circle_filled(button_pos, button_size / 2.0, egui::Color32::WHITE);
            painter.circle_stroke(
                button_pos,
                button_size / 2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 150, 150)),
            );
        }

        // Кнопка очистки истории (чуть правее от кнопки закрытия)
        let clear_button_pos = egui::pos2(
            cloud_rect.min.x + padding + button_size / 2.0 + button_size + 8.0,
            cloud_rect.min.y + padding + button_size / 2.0,
        );
        let clear_button_rect = egui::Rect::from_center_size(clear_button_pos, egui::vec2(button_size + 4.0, button_size + 4.0));

        if let Some(mouse_pos) = ctx.input(|i| i.pointer.latest_pos()) {
            if clear_button_rect.contains(mouse_pos) {
                ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);

                if ctx.input(|i| i.pointer.primary_clicked()) {
                    self.show_clear_confirmation = !self.show_clear_confirmation;
                    ctx.request_repaint();
                }

                // Рисуем в состоянии hover (более яркая обводка)
                painter.circle_filled(clear_button_pos, button_size / 2.0, egui::Color32::from_rgb(220, 100, 100));
                painter.circle_stroke(
                    clear_button_pos,
                    button_size / 2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(150, 50, 50)),
                );
            } else {
                // Нормальное состояние
                painter.circle_filled(clear_button_pos, button_size / 2.0, egui::Color32::from_rgb(200, 80, 80));
                painter.circle_stroke(
                    clear_button_pos,
                    button_size / 2.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 50, 50)),
                );
            }
        } else {
            painter.circle_filled(clear_button_pos, button_size / 2.0, egui::Color32::from_rgb(200, 80, 80));
            painter.circle_stroke(
                clear_button_pos,
                button_size / 2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(150, 50, 50)),
            );
        }

        // Рисуем букву "X" на кнопке очистки
        let x_size = 4.0;
        let x_color = egui::Color32::WHITE;
        painter.line_segment(
            [
                egui::pos2(clear_button_pos.x - x_size, clear_button_pos.y - x_size),
                egui::pos2(clear_button_pos.x + x_size, clear_button_pos.y + x_size),
            ],
            egui::Stroke::new(1.5, x_color),
        );
        painter.line_segment(
            [
                egui::pos2(clear_button_pos.x + x_size, clear_button_pos.y - x_size),
                egui::pos2(clear_button_pos.x - x_size, clear_button_pos.y + x_size),
            ],
            egui::Stroke::new(1.5, x_color),
        );

        // Показываем диалог подтверждения если требуется
        if self.show_clear_confirmation {
            let dialog_pos = egui::pos2(cloud_rect.center().x - 100.0, cloud_rect.min.y - 60.0);
            let dialog_rect = egui::Rect::from_min_size(dialog_pos, egui::vec2(200.0, 50.0));

            // Фон диалога
            painter.rect_filled(dialog_rect, 5.0, egui::Color32::from_rgb(40, 40, 40));
            painter.rect_stroke(dialog_rect, 5.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)), egui::epaint::StrokeKind::Outside);

            // Текст подтверждения
            painter.text(
                dialog_rect.center() - egui::vec2(0.0, 8.0),
                egui::Align2::CENTER_CENTER,
                "Очистить историю?",
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );

            // Кнопка "Да"
            let yes_rect = egui::Rect::from_min_size(
                egui::pos2(dialog_rect.min.x + 10.0, dialog_rect.max.y - 20.0),
                egui::vec2(35.0, 15.0),
            );
            let yes_hovered = ctx.input(|i| i.pointer.latest_pos())
                .map(|p| yes_rect.contains(p))
                .unwrap_or(false);

            painter.rect_filled(
                yes_rect,
                2.0,
                if yes_hovered {
                    egui::Color32::from_rgb(50, 150, 50)
                } else {
                    egui::Color32::from_rgb(40, 120, 40)
                },
            );

            painter.text(
                yes_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Да",
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
            );

            if yes_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                self.clear_agent_history();
            }

            // Кнопка "Нет"
            let no_rect = egui::Rect::from_min_size(
                egui::pos2(dialog_rect.max.x - 45.0, dialog_rect.max.y - 20.0),
                egui::vec2(35.0, 15.0),
            );
            let no_hovered = ctx.input(|i| i.pointer.latest_pos())
                .map(|p| no_rect.contains(p))
                .unwrap_or(false);

            painter.rect_filled(
                no_rect,
                2.0,
                if no_hovered {
                    egui::Color32::from_rgb(150, 50, 50)
                } else {
                    egui::Color32::from_rgb(120, 40, 40)
                },
            );

            painter.text(
                no_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Нет",
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
            );

            if no_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                self.show_clear_confirmation = false;
                ctx.request_repaint();
            }
        }
    }

    /// Рисует кнопку открытия облака (маленький синий круг + рядом с картинкой)
    fn draw_show_button(&mut self, ctx: &egui::Context, image_rect: egui::Rect) {
        let button_size = 20.0; // маленький размер
        let padding = 5.0;

        // Позиция: слева-сверху от картинки
        let button_pos = egui::pos2(
            image_rect.min.x - padding - button_size / 2.0,
            image_rect.min.y + padding + button_size / 2.0,
        );

        let button_rect = egui::Rect::from_center_size(button_pos, egui::vec2(button_size + 4.0, button_size + 4.0));

        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("show_button")));

        // Получаем позицию мыши
        if let Some(mouse_pos) = ctx.input(|i| i.pointer.latest_pos()) {
            // Проверяем, находится ли мышь над кнопкой
            if button_rect.contains(mouse_pos) {
                ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);

                // Проверяем нажатие левой кнопки мыши
                if ctx.input(|i| i.pointer.primary_clicked()) {
                    self.cloud_visible = true;
                    ctx.request_repaint();
                }

                // Рисуем кнопку в состоянии hover (более яркая)
                painter.circle_filled(button_pos, button_size / 2.0, egui::Color32::from_rgb(50, 150, 200));
                painter.circle_stroke(
                    button_pos,
                    button_size / 2.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(30, 100, 150)),
                );
            } else {
                // Рисуем кнопку в нормальном состоянии
                painter.circle_filled(button_pos, button_size / 2.0, egui::Color32::from_rgb(40, 130, 180));
                painter.circle_stroke(
                    button_pos,
                    button_size / 2.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(20, 80, 130)),
                );
            }
        } else {
            // Рисуем кнопку в нормальном состоянии
            painter.circle_filled(button_pos, button_size / 2.0, egui::Color32::from_rgb(40, 130, 180));
            painter.circle_stroke(
                button_pos,
                button_size / 2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(20, 80, 130)),
            );
        }

        // Рисуем плюсик (+) в центре кнопки
        let plus_size = 6.0;
        let plus_color = egui::Color32::WHITE;

        // Вертикальная линия
        painter.line_segment(
            [
                egui::pos2(button_pos.x, button_pos.y - plus_size),
                egui::pos2(button_pos.x, button_pos.y + plus_size),
            ],
            egui::Stroke::new(1.5, plus_color),
        );

        // Горизонтальная линия
        painter.line_segment(
            [
                egui::pos2(button_pos.x - plus_size, button_pos.y),
                egui::pos2(button_pos.x + plus_size, button_pos.y),
            ],
            egui::Stroke::new(1.5, plus_color),
        );
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
                        
                        // Обработка двойного клика для открытия облака
                        if image_response.double_clicked() {
                            self.cloud_visible = true;
                            ctx.request_repaint();
                        }

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
            // Показываем облако только если оно видимо
            if self.cloud_visible {
                let cloud_rect = ui::show_talk_cloud_side(
                    ctx,
                    text,
                    image_rect,                 // В экранных координатах
                    110,                        // ~110 символов в строке
                    120.0,                      // макс. высота видимой области (px)
                    20.0,                       // зазор до картинки
                    true,                       // prefer_left: старайся ставить слева (картинка теперь справа)
                    egui::FontId::proportional(16.0),
                );

                // Рисуем кнопку закрытия над облаком
                self.draw_close_button(ctx, cloud_rect);
            } else {
                // Показываем кнопку + чтобы открыть облако снова
                self.draw_show_button(ctx, image_rect);
            }
        }
        
        // Показываем простой интерфейс для ввода текста (если облако видимо)
        if self.cloud_visible {
            self.draw_input_interface(ctx);
        }
    }
}

impl ClippyApp {
    /// Рисует интерфейс для ввода сообщений
    fn draw_input_interface(&mut self, ctx: &egui::Context) {
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("input_interface")));
        let screen_rect = ctx.screen_rect();

        // Нижняя панель для ввода
        let input_height = 50.0;
        let padding = 10.0;
        let input_rect = egui::Rect::from_min_max(
            egui::pos2(screen_rect.min.x + padding, screen_rect.max.y - input_height - padding),
            egui::pos2(screen_rect.max.x - padding, screen_rect.max.y - padding),
        );

        // Фон панели ввода
        painter.rect_filled(input_rect, 8.0, egui::Color32::from_rgb(240, 240, 240));
        painter.rect_stroke(input_rect, 8.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 180)), egui::epaint::StrokeKind::Outside);

        // Текстовое поле на Foreground слое (через egui::Area для интерактивности)
        let input_area_rect = egui::Rect::from_min_max(
            egui::pos2(input_rect.min.x + padding, input_rect.min.y + 8.0),
            egui::pos2(input_rect.max.x - 60.0, input_rect.max.y - 8.0),
        );

        // Кнопка отправки
        let send_button_rect = egui::Rect::from_min_max(
            egui::pos2(input_rect.max.x - 50.0, input_rect.min.y + 8.0),
            egui::pos2(input_rect.max.x - 10.0, input_rect.max.y - 8.0),
        );

        // Проверяем наведение на кнопку отправки
        let send_hovered = ctx.input(|i| i.pointer.latest_pos())
            .map(|p| send_button_rect.contains(p))
            .unwrap_or(false);

        // Рисуем кнопку отправки
        painter.rect_filled(
            send_button_rect,
            4.0,
            if send_hovered {
                egui::Color32::from_rgb(100, 200, 100)
            } else {
                egui::Color32::from_rgb(80, 180, 80)
            },
        );

        painter.text(
            send_button_rect.center(),
            egui::Align2::CENTER_CENTER,
            "↑",
            egui::FontId::proportional(20.0),
            egui::Color32::WHITE,
        );

        // Проверяем клик на кнопку отправки
        if send_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
            self.send_message(ctx);
            ctx.request_repaint();
        }

        // Показываем текущий статус или подсказку
        let hint_text = if self.is_thinking {
            "Думаю..."
        } else if self.input_text.is_empty() {
            "Введите сообщение..."
        } else {
            ""
        };

        if !hint_text.is_empty() && self.input_text.is_empty() {
            painter.text(
                input_area_rect.min + egui::vec2(8.0, 12.0),
                egui::Align2::LEFT_CENTER,
                hint_text,
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(160, 160, 160),
            );
        }

        // Area для интерактивного текстового ввода
        egui::Area::new(egui::Id::new("input_field_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(input_area_rect.min)
            .show(ctx, |ui| {
                ui.set_width(input_area_rect.width());
                ui.set_height(input_area_rect.height());

                ui.horizontal(|ui| {
                    // Используем TextEdit для ввода
                    let response = ui.text_edit_singleline(&mut self.input_text);

                    // Проверяем Enter для отправки сообщения
                    if response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.send_message(ctx);
                    }
                });
            });
    }
}
