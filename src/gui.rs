/// Main GUI module that delegates to UI submodules
use crate::ui::app::ClippyApp;
use eframe::egui;

impl eframe::App for ClippyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Position the window on first run
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

        // Initialize UI style
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

        // Load Clippy image
        self.load_clippy_image(ctx);

        // Show greeting message
        if !self.greeting_shown && self.start_time.elapsed().as_secs() >= 3 {
            self.greeting_shown = true;
            let greeting = "Привет! 👋 Нажми на зелёную кнопку, чтобы поговорить.".to_string();
            self.messages.push(("clippy".to_string(), greeting.clone()));

            let tts = std::sync::Arc::clone(&self.tts);
            tokio::spawn(async move {
                if let Err(e) = tts.speak(&greeting).await {
                    eprintln!("Ошибка озвучивания: {}", e);
                }
            });

            ctx.request_repaint();
        }

        // Process responses from AI agent
        while let Ok(response) = self.response_receiver.try_recv() {
            self.messages.push(("clippy".to_string(), response.clone()));
            self.is_thinking = false;

            let tts = std::sync::Arc::clone(&self.tts);
            tokio::spawn(async move {
                if let Err(e) = tts.speak(&response).await {
                    eprintln!("Ошибка озвучивания: {}", e);
                }
            });

            ctx.request_repaint();
        }

        let mut image_rect: Option<egui::Rect> = None;

        // Render main UI
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
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
                        let (rect, response) = ui.allocate_exact_size(image_size, egui::Sense::drag());

                        let clip_rect = ui.clip_rect();
                        let screen_rect = ctx.screen_rect();
                        let offset = screen_rect.min - clip_rect.min;
                        let screen_image_rect = egui::Rect::from_min_size(
                            rect.min + offset,
                            image_size,
                        );

                        image_rect = Some(screen_image_rect);

                        ui.painter().image(
                            texture.id(),
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );

                        // Toggle chat on double-click
                        if response.double_clicked() {
                            self.chat_visible = !self.chat_visible;
                            self.animation_progress = 0.0;
                            ctx.request_repaint();
                        }

                        // Allow dragging the window
                        if response.drag_started() {
                            ctx.send_viewport_cmd_to(
                                egui::ViewportId::ROOT,
                                egui::ViewportCommand::StartDrag,
                            );
                        }
                    } else {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(100.0, 60.0),
                            egui::Sense::click(),
                        );

                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Загрузка...",
                            egui::FontId::default(),
                            egui::Color32::WHITE,
                        );
                    }
                });
            });

        // Render send button and chat window if image was loaded
        if let Some(rect) = image_rect {
            self.draw_send_button(ctx, rect);

            if self.chat_visible {
                self.draw_chat_window(ctx, rect);
                self.check_close_chat(ctx);
            }
        }
    }
}
