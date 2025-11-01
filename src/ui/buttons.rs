/// Send button rendering and interaction module
use eframe::egui;

/// Button styling constants
pub const BUTTON_SIZE: f32 = 50.0;
pub const BUTTON_PADDING: f32 = 10.0;
pub const ARROW_SIZE: f32 = 16.0;

/// Renders the send button (green square with up arrow)
///
/// Returns true if the button was clicked
pub fn draw_send_button(
    ctx: &egui::Context,
    image_rect: egui::Rect,
) -> bool {
    let button_pos = egui::pos2(
        image_rect.max.x + BUTTON_PADDING,
        image_rect.center().y - BUTTON_SIZE / 2.0,
    );

    let button_rect = egui::Rect::from_min_size(button_pos, egui::vec2(BUTTON_SIZE, BUTTON_SIZE));

    // Проверяем наведение мыши
    let mouse_pos = ctx.input(|i| i.pointer.latest_pos());
    let is_hovered = mouse_pos.map(|p| button_rect.contains(p)).unwrap_or(false);

    // Рисуем кнопку
    let button_color = if is_hovered {
        egui::Color32::from_rgb(60, 180, 120)
    } else {
        egui::Color32::from_rgb(40, 150, 100)
    };

    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("send_button")));
    painter.rect_filled(button_rect, 8.0, button_color);

    // Стрелка вверх
    let center = button_rect.center();

    // Стрелка (треугольник вверх)
    let arrow_points = vec![
        egui::pos2(center.x, center.y - ARROW_SIZE / 2.0),           // Верхняя точка
        egui::pos2(center.x - ARROW_SIZE / 3.0, center.y + ARROW_SIZE / 3.0), // Левый угол
        egui::pos2(center.x + ARROW_SIZE / 3.0, center.y + ARROW_SIZE / 3.0), // Правый угол
    ];

    painter.add(egui::Shape::Path(egui::epaint::PathShape {
        points: arrow_points,
        closed: true,
        fill: egui::Color32::WHITE,
        stroke: egui::Stroke::NONE.into(),
    }));

    // Меняем курсор
    if is_hovered {
        ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    is_hovered && ctx.input(|i| i.pointer.primary_clicked())
}
