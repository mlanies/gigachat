/// Переиспользуемые UI компоненты
/// TODO: Добавить компоненты:
/// - Button
/// - TextInput  
/// - MessageBox
/// - SettingsPanel
/// - StatusBar

pub struct Button {
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Button {
    pub fn new(label: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            label: label.to_string(),
            x,
            y,
            width,
            height,
        }
    }
}
