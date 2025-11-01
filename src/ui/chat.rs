/// Chat window rendering and interaction module
use eframe::egui;

/// Constants for chat window styling
pub const CHAT_WINDOW_WIDTH: f32 = 350.0;
pub const CHAT_WINDOW_HEIGHT: f32 = 420.0;
pub const CHAT_PADDING: f32 = 15.0;
pub const SEND_BUTTON_SIZE: f32 = 35.0;
pub const LINE_HEIGHT: f32 = 14.0;

/// Renders the animated chat window with messages
///
/// # Arguments
/// * `painter` - The painter for drawing
/// * `animated_rect` - The rectangle with animation transformations applied
/// * `alpha` - Transparency value (0-255)
/// * `messages` - List of messages (role, content)
/// * `is_thinking` - Whether the agent is currently processing
pub fn draw_messages(
    painter: &egui::Painter,
    animated_rect: egui::Rect,
    alpha: u8,
    messages: &[(String, String)],
    is_thinking: bool,
) {
    // Разделитель под заголовком
    let title_y = animated_rect.min.y + 15.0 + 20.0;
    painter.line_segment(
        [
            egui::pos2(animated_rect.min.x + 10.0, title_y),
            egui::pos2(animated_rect.max.x - 10.0, title_y),
        ],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha)),
    );

    // Область сообщений
    let messages_area_top = title_y + 8.0;
    let messages_area_bottom = animated_rect.max.y - 45.0;
    let messages_area = egui::Rect::from_min_max(
        egui::pos2(animated_rect.min.x + 10.0, messages_area_top),
        egui::pos2(animated_rect.max.x - 10.0, messages_area_bottom),
    );

    // Рисуем сообщения с улучшенным форматированием
    let mut y = messages_area.min.y + 5.0;
    let message_max_width = messages_area.width() - 16.0;

    for (role, msg) in messages {
        let is_user = role == "user";

        let (bubble_color, text_color, alignment) = if is_user {
            (
                egui::Color32::from_rgba_unmultiplied(100, 200, 100, alpha),
                egui::Color32::WHITE,
                "Вы",
            )
        } else {
            (
                egui::Color32::from_rgba_unmultiplied(220, 220, 220, alpha),
                egui::Color32::from_rgb(40, 40, 40),
                "Скрепыш",
            )
        };

        // Отправитель
        painter.text(
            egui::pos2(messages_area.min.x + 8.0, y),
            egui::Align2::LEFT_TOP,
            alignment,
            egui::FontId::proportional(10.0),
            egui::Color32::from_rgba_unmultiplied(120, 120, 120, alpha),
        );

        y += 16.0;

        // Сообщение в пузыре
        let msg_lines: Vec<&str> = msg.lines().collect();
        let mut max_msg_height = LINE_HEIGHT;

        for _ in &msg_lines {
            max_msg_height += LINE_HEIGHT;
        }

        let bubble_rect = egui::Rect::from_min_size(
            egui::pos2(messages_area.min.x + 8.0, y),
            egui::vec2(message_max_width, max_msg_height),
        );

        painter.rect_filled(bubble_rect, 6.0, bubble_color);

        // Текст сообщения
        let mut text_y = y + 4.0;
        for line in &msg_lines {
            painter.text(
                egui::pos2(bubble_rect.min.x + 8.0, text_y),
                egui::Align2::LEFT_TOP,
                *line,
                egui::FontId::proportional(11.0),
                text_color,
            );
            text_y += LINE_HEIGHT;
        }

        y += max_msg_height + 10.0;

        if y > messages_area.max.y {
            break;
        }
    }

    // Индикатор "думаю..."
    if is_thinking {
        let thinking_y = messages_area.max.y - 25.0;
        painter.text(
            egui::pos2(messages_area.min.x + 8.0, thinking_y),
            egui::Align2::LEFT_TOP,
            "⏳ Думаю...",
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgba_unmultiplied(150, 150, 150, alpha),
        );
    }
}

/// Renders the send button and returns true if clicked
pub fn draw_send_button(
    painter: &egui::Painter,
    ctx: &egui::Context,
    animated_rect: egui::Rect,
    alpha: u8,
    is_thinking: bool,
) -> bool {
    let send_btn_rect = egui::Rect::from_min_size(
        egui::pos2(animated_rect.max.x - 45.0, animated_rect.max.y - 38.0),
        egui::vec2(SEND_BUTTON_SIZE, 30.0),
    );

    let is_send_hovered = ctx.input(|i| i.pointer.latest_pos())
        .map(|p| send_btn_rect.contains(p))
        .unwrap_or(false);

    let send_btn_color = if is_thinking {
        egui::Color32::from_rgba_unmultiplied(150, 150, 150, alpha)
    } else if is_send_hovered {
        egui::Color32::from_rgba_unmultiplied(60, 180, 120, alpha)
    } else {
        egui::Color32::from_rgba_unmultiplied(40, 150, 100, alpha)
    };

    painter.rect_filled(send_btn_rect, 6.0, send_btn_color);

    painter.text(
        send_btn_rect.center(),
        egui::Align2::CENTER_CENTER,
        "↑",
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );

    !is_thinking && is_send_hovered && ctx.input(|i| i.pointer.primary_clicked())
}

/// Renders the input field and returns true if Enter was pressed
pub fn draw_input_field(
    painter: &egui::Painter,
    ctx: &egui::Context,
    animated_rect: egui::Rect,
    alpha: u8,
    input_text: &mut String,
) -> bool {
    // Input field background
    let input_area = egui::Rect::from_min_size(
        egui::pos2(animated_rect.min.x + 10.0, animated_rect.max.y - 38.0),
        egui::vec2(animated_rect.width() - 55.0, 30.0),
    );

    // Input field border
    painter.rect_stroke(
        input_area,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(180, 180, 180, alpha)),
        egui::epaint::StrokeKind::Outside,
    );

    let mut enter_pressed = false;

    // Для интерактивного ввода используем egui::Area
    egui::Area::new(egui::Id::new("chat_input"))
        .order(egui::Order::Foreground)
        .fixed_pos(input_area.min + egui::vec2(8.0, 5.0))
        .show(ctx, |ui| {
            ui.set_width(input_area.width() - 16.0);
            let response = ui.text_edit_singleline(input_text);

            // Отправка по Enter
            enter_pressed = response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter));
        });

    enter_pressed
}
