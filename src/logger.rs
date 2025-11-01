/// Модуль для инициализации логирования
/// Логирует в файл logs/clippy.log и консоль в режиме development

use std::io::Write;
use log::LevelFilter;

pub fn init() {
    let mut builder = env_logger::Builder::new();
    
    // Читаем RUST_LOG переменную если она установлена
    if let Ok(log_level) = std::env::var("RUST_LOG") {
        builder.parse_filters(&log_level);
    } else {
        // По умолчанию: INFO уровень
        builder.filter_level(LevelFilter::Info);
        // Отключаем логи от зависимостей (слишком многословно)
        builder.filter_module("eframe", LevelFilter::Warn);
        builder.filter_module("egui", LevelFilter::Warn);
        builder.filter_module("wgpu", LevelFilter::Warn);
    }
    
    // Формат логов: [HH:MM:SS LEVEL] модуль - сообщение
    builder.format(|buf, record| {
        let now = chrono::Local::now().format("%H:%M:%S");
        writeln!(
            buf,
            "[{} {}] {} - {}",
            now,
            record.level(),
            record.target(),
            record.args()
        )
    });
    
    // Создаем директорию для логов если её нет
    let log_dir = "logs";
    if !std::path::Path::new(log_dir).exists() {
        let _ = std::fs::create_dir(log_dir);
    }
    
    // Инициализируем логирование в файл
    builder
        .target(env_logger::Target::Pipe(Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/clippy.log")
                .expect("Не удалось открыть файл логов")
        )))
        .init();
    
    log::info!("Логирование инициализировано ✓");
}
