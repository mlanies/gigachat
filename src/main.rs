// Модули приложения
mod config;
mod ai;
mod services;
mod ui;
mod core;
mod gui;
mod logger;

use config::Config;
use gui::ClippyApp;
use eframe::NativeOptions;

fn main() -> Result<(), eframe::Error> {
    // Инициализируем логирование
    logger::init();
    log::info!("🚀 Скрепыш запущен");

    // Создаем tokio runtime для async операций
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let config = Config::default();
    log::info!("📁 Конфигурация загружена");
    let clippy_name = config.clippy_name.clone();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.window_width, config.window_height])
            .with_transparent(true) // Ключевой флаг: окно реально прозрачное
            .with_decorations(false) // Без рамок
            .with_titlebar_buttons_shown(false)
            .with_titlebar_shown(false)
            .with_always_on_top() // Всегда поверх других окон
            .with_resizable(false), // Нельзя изменять размер
            // Позиция будет установлена динамически в update() с использованием реального размера экрана
        ..Default::default()
    };

    eframe::run_native(
        &clippy_name,
        options,
        Box::new(move |_cc| -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new(ClippyApp::new(config)))
        }),
    )
}
