/// Button rendering and interaction module
use eframe::egui;

/// Button styling constants
pub const BUTTON_SIZE: f32 = 20.0;
pub const BUTTON_PADDING: f32 = 5.0;

/// Renders the show cloud button (green circle with +)
///
/// Returns true if the button was clicked
pub fn draw_show_button(
    ctx: &egui::Context,
    image_rect: egui::Rect,
) -> bool {
    let button_pos = egui::pos2(
        image_rect.min.x - BUTTON_PADDING - BUTTON_SIZE / 2.0,
        image_rect.min.y + BUTTON_PADDING + BUTTON_SIZE / 2.0,
    );

    let button_rect = egui::Rect::from_center_size(button_pos, egui::vec2(BUTTON_SIZE, BUTTON_SIZE));

    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("show_button")));

    // Проверяем наведение мыши
    if let Some(mouse_pos) = ctx.input(|i| i.pointer.latest_pos()) {
        let is_hovered = button_rect.contains(mouse_pos);

        if is_hovered {
            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);

            // Рисуем кнопку в состоянии hover (более яркая)
            painter.circle_filled(button_pos, BUTTON_SIZE / 2.0, egui::Color32::from_rgb(50, 150, 100));
            painter.circle_stroke(
                button_pos,
                BUTTON_SIZE / 2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(30, 100, 50)),
            );
        } else {
            // Рисуем кнопку в нормальном состоянии
            painter.circle_filled(button_pos, BUTTON_SIZE / 2.0, egui::Color32::from_rgb(40, 130, 80));
            painter.circle_stroke(
                button_pos,
                BUTTON_SIZE / 2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(20, 80, 40)),
            );
        }

        // Рисуем плюс (+) в центре кнопки
        let plus_size = 8.0;
        let plus_color = egui::Color32::WHITE;

        // Вертикальная линия
        painter.line_segment(
            [
                egui::pos2(button_pos.x, button_pos.y - plus_size / 2.0),
                egui::pos2(button_pos.x, button_pos.y + plus_size / 2.0),
            ],
            egui::Stroke::new(2.0, plus_color),
        );

        // Горизонтальная линия
        painter.line_segment(
            [
                egui::pos2(button_pos.x - plus_size / 2.0, button_pos.y),
                egui::pos2(button_pos.x + plus_size / 2.0, button_pos.y),
            ],
            egui::Stroke::new(2.0, plus_color),
        );

        is_hovered && ctx.input(|i| i.pointer.primary_clicked())
    } else {
        // Рисуем кнопку в нормальном состоянии без мыши
        painter.circle_filled(button_pos, BUTTON_SIZE / 2.0, egui::Color32::from_rgb(40, 130, 80));
        painter.circle_stroke(
            button_pos,
            BUTTON_SIZE / 2.0,
            egui::Stroke::new(1.5, egui::Color32::from_rgb(20, 80, 40)),
        );

        // Рисуем плюс (+) в центре кнопки
        let plus_size = 8.0;
        let plus_color = egui::Color32::WHITE;

        // Вертикальная линия
        painter.line_segment(
            [
                egui::pos2(button_pos.x, button_pos.y - plus_size / 2.0),
                egui::pos2(button_pos.x, button_pos.y + plus_size / 2.0),
            ],
            egui::Stroke::new(2.0, plus_color),
        );

        // Горизонтальная линия
        painter.line_segment(
            [
                egui::pos2(button_pos.x - plus_size / 2.0, button_pos.y),
                egui::pos2(button_pos.x + plus_size / 2.0, button_pos.y),
            ],
            egui::Stroke::new(2.0, plus_color),
        );

        false
    }
}
