/// Widget system for displaying weather, currency rates, and stats
use eframe::egui;

/// Constants for widget styling
pub const WIDGET_WIDTH: f32 = 100.0;
pub const WIDGET_HEIGHT: f32 = 70.0;
pub const WIDGET_PADDING: f32 = 8.0;
pub const WIDGET_SPACING: f32 = 8.0;

/// Weather widget data
#[derive(Clone, Debug)]
pub struct WeatherWidget {
    pub temperature: String,
    pub condition: String,
    pub humidity: String,
}

impl Default for WeatherWidget {
    fn default() -> Self {
        Self {
            temperature: "-- °C".to_string(),
            condition: "...".to_string(),
            humidity: "-- %".to_string(),
        }
    }
}

/// Currency rate widget data
#[derive(Clone, Debug)]
pub struct CurrencyWidget {
    pub code: String,
    pub symbol: String,
    pub rate: String,
}

impl CurrencyWidget {
    pub fn new(code: &str, symbol: &str, rate: &str) -> Self {
        Self {
            code: code.to_string(),
            symbol: symbol.to_string(),
            rate: rate.to_string(),
        }
    }
}

/// Renders a single widget box with title and content
pub fn draw_widget(
    painter: &egui::Painter,
    rect: egui::Rect,
    title: &str,
    content: &str,
    alpha: u8,
) {
    // Widget background (light gray)
    let bg_color = egui::Color32::from_rgba_unmultiplied(240, 240, 240, alpha);
    painter.rect_filled(rect, 6.0, bg_color);

    // Widget border (light gray)
    let border_color = egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha);
    painter.rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(1.0, border_color),
        egui::epaint::StrokeKind::Outside,
    );

    // Title text (dark gray, small)
    let title_y = rect.min.y + WIDGET_PADDING;
    painter.text(
        egui::pos2(rect.min.x + WIDGET_PADDING, title_y),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgba_unmultiplied(100, 100, 100, alpha),
    );

    // Content text (dark, larger)
    let content_y = title_y + 16.0;
    painter.text(
        egui::pos2(rect.min.x + WIDGET_PADDING, content_y),
        egui::Align2::LEFT_TOP,
        content,
        egui::FontId::proportional(12.0),
        egui::Color32::from_rgba_unmultiplied(40, 40, 40, alpha),
    );
}

/// Renders the weather widget
pub fn draw_weather_widget(
    painter: &egui::Painter,
    rect: egui::Rect,
    alpha: u8,
    weather: &WeatherWidget,
) {
    draw_widget(painter, rect, "🌡️ Погода", &weather.temperature, alpha);

    // Secondary info (humidity)
    let info_y = rect.min.y + 50.0;
    painter.text(
        egui::pos2(rect.min.x + WIDGET_PADDING, info_y),
        egui::Align2::LEFT_TOP,
        &format!("💧 {}", weather.humidity),
        egui::FontId::proportional(8.0),
        egui::Color32::from_rgba_unmultiplied(120, 120, 120, alpha),
    );
}

/// Renders a currency widget
pub fn draw_currency_widget(
    painter: &egui::Painter,
    rect: egui::Rect,
    alpha: u8,
    currency: &CurrencyWidget,
) {
    draw_widget(
        painter,
        rect,
        &format!("{} {}", currency.symbol, currency.code),
        &currency.rate,
        alpha,
    );
}

/// Renders the stats widget
pub fn draw_stats_widget(
    painter: &egui::Painter,
    rect: egui::Rect,
    alpha: u8,
    messages_count: usize,
) {
    let content = format!("{}", messages_count);
    draw_widget(painter, rect, "📊 Сообщений", &content, alpha);
}
