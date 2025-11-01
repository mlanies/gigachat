use eframe::egui;

/// Пузырь сбоку от картинки, ширина ≈ N символов, макс. высота с прокруткой.
/// Автоматически выбирает сторону (лево/право) на основе доступного пространства.
/// Рисуется на Foreground-слое, поэтому всегда поверх картинки.
/// Возвращает Rect облака для использования в других операциях (например, для отрисовки кнопки).
pub fn show_talk_cloud_side(
    ctx: &egui::Context,
    text: &str,
    image_rect: egui::Rect,      // экранные координаты картинки
    max_chars_per_line: usize,   // «60 символов»
    max_height_px: f32,          // «100–120 px»
    gap: f32,                    // зазор от картинки
    prefer_left: bool,           // приоритет левой стороны
    font: egui::FontId,
) -> egui::Rect {
    let screen = ctx.screen_rect();

    // 1) Переведём «N символов» в пиксели (грубая верхняя граница)
    let wrap_w_target = ctx.fonts(|f| {
        let sample = "W".repeat(max_chars_per_line.max(1));
        f.layout_no_wrap(sample, font.clone(), egui::Color32::TRANSPARENT)
            .size().x
    });

    // Доступное место слева/справа от картинки
    let space_left  = (image_rect.min.x - screen.min.x - gap).max(0.0);
    let space_right = (screen.max.x - image_rect.max.x - gap).max(0.0);

    // Выбор стороны с более строгими проверками
    // Нужно минимум 120px + gap для размещения облака
    let min_required_space = 120.0 + gap;
    
    let mut place_left = if prefer_left {
        // Слева, только если есть достаточно места (минимум min_required_space)
        space_left >= min_required_space && (space_left >= space_right || space_right < min_required_space)
    } else {
        space_left >= min_required_space && space_left > space_right
    };

    // Реальная ширина обёртки (не больше wrap_w_target и не меньше разумного минимума)
    let mut wrap_w = if place_left {
        (space_left - gap).clamp(120.0, wrap_w_target)
    } else {
        (space_right - gap).clamp(120.0, wrap_w_target)
    };

    // Если с выбранной стороны нет достаточно места — пробуем другую
    if wrap_w < 120.0 {
        place_left = !place_left;
        wrap_w = if place_left {
            (space_left - gap).clamp(120.0, wrap_w_target)
        } else {
            (space_right - gap).clamp(120.0, wrap_w_target)
        };
    }
    
    // Если все еще нет места ни с одной стороны - принудительно используем правую
    if wrap_w < 120.0 {
        place_left = false;
        wrap_w = (space_right - gap).max(100.0).min(wrap_w_target); // хотя бы 100px минимум
    }

    // 2) Посчитаем фактический размер текста при такой ширине
    let text_size = ctx.fonts(|f| {
        f.layout(text.to_owned(), font.clone(), egui::Color32::TRANSPARENT, wrap_w).size()
    });

    let pad = 12.0;
    let rounding = 12.0;
    let full_size = egui::vec2(wrap_w + pad * 2.0, text_size.y + pad * 2.0);
    let visible_h = (text_size.y + pad * 2.0).min(max_height_px + pad * 2.0);
    let vis_size = egui::vec2(full_size.x, visible_h);

    // 3) Позиция облака: строго сбоку от картинки, по центру по Y, с клипом по экрану
    // ВАЖНО: облако должно быть строго СБОКУ, не перекрывая картинку
    // Вычисляем позицию так, чтобы между облаком и картинкой был зазор gap
    let mut cloud_min = if place_left {
        // Слева: облако должно заканчиваться ДО image_rect.min.x с зазором gap
        // cloud_min.x + vis_size.x = image_rect.min.x - gap
        let x_pos = image_rect.min.x - gap - vis_size.x;
        egui::pos2(x_pos, image_rect.center().y - vis_size.y / 2.0)
    } else {
        // Справа: облако должно начинаться ПОСЛЕ image_rect.max.x с зазором gap
        // cloud_min.x = image_rect.max.x + gap
        egui::pos2(image_rect.max.x + gap, image_rect.center().y - vis_size.y / 2.0)
    };
    
    // Подрежем по экрану (но НЕ перекрывая картинку!)
    if cloud_min.y < screen.min.y { cloud_min.y = screen.min.y + 5.0; }
    if cloud_min.y + vis_size.y > screen.max.y { cloud_min.y = screen.max.y - vis_size.y - 5.0; }
    
    // КРИТИЧНО: Если после обрезки по экрану облако перекрывает картинку - принудительно сдвигаем
    let cloud_right = cloud_min.x + vis_size.x;
    let cloud_left = cloud_min.x;
    
    if place_left {
        // Слева: правая граница облака не должна заходить за левую границу картинки
        if cloud_right > image_rect.min.x - gap {
            cloud_min.x = image_rect.min.x - gap - vis_size.x;
        }
        // Если вышли за левую границу экрана - сдвигаем, но проверяем что не перекрываем картинку
        if cloud_min.x < screen.min.x {
            // Если все равно перекрывает - перемещаем справа
            if cloud_right > image_rect.min.x - gap {
                place_left = false;
                cloud_min.x = image_rect.max.x + gap;
            } else {
                cloud_min.x = screen.min.x + 5.0;
            }
        }
    } else {
        // Справа: левая граница облака не должна заходить за правую границу картинки
        if cloud_left < image_rect.max.x + gap {
            cloud_min.x = image_rect.max.x + gap;
        }
        // Если вышли за правую границу экрана - сдвигаем
        if cloud_min.x + vis_size.x > screen.max.x {
            cloud_min.x = screen.max.x - vis_size.x - 5.0;
            // Проверяем что после сдвига не перекрываем картинку
            if cloud_min.x < image_rect.max.x + gap {
                // Если перекрываем - перемещаем слева
                place_left = true;
                cloud_min.x = image_rect.min.x - gap - vis_size.x;
            }
        }
    }
    
    // Финальная гарантия: проверяем что облако НЕ пересекается с картинкой
    let cloud_rect_final = egui::Rect::from_min_size(cloud_min, vis_size);
    if cloud_rect_final.intersects(image_rect) {
        // Принудительно размещаем справа, если слева не влезает
        if place_left {
            cloud_min.x = image_rect.max.x + gap;
            place_left = false;
        } else {
            cloud_min.x = image_rect.min.x - gap - vis_size.x;
            place_left = true;
        }
    }

    // 4) Рисуем облако как Area в Foreground (будет свой слой и скролл)
    let mut cloud_rect_drawn = egui::Rect::NAN;
    egui::Area::new(egui::Id::new("speech_bubble_area"))
        .order(egui::Order::Foreground)
        .fixed_pos(cloud_min)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_min_width(vis_size.x);
            ui.set_max_width(vis_size.x);

            let frame = egui::Frame::new()
                .fill(egui::Color32::from_rgb(245, 246, 247)) // светлый фон
                .stroke(egui::Stroke::new(1.5, egui::Color32::from_gray(180)))
                .corner_radius(rounding)
                .inner_margin(egui::Margin::same(pad as i8));

            frame.show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(max_height_px)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_width(wrap_w);
                        ui.label(
                            egui::RichText::new(text)
                                .font(font.clone())
                                .color(egui::Color32::from_rgb(40, 40, 40)),
                        );
                    });
            });

            cloud_rect_drawn = ui.min_rect();
        });

    // 5) Хвостик на том же слое (используем обновленное значение place_left после финальной проверки)
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("speech_bubble_tail")));
    let tail = 10.0;
    let final_place_left = place_left; // Сохраняем финальное значение
    let (a, b, c) = if final_place_left {
        // облако слева → хвост вправо, к картинке
        (
            egui::pos2(cloud_rect_drawn.max.x,              cloud_rect_drawn.center().y),
            egui::pos2(cloud_rect_drawn.max.x + tail * 0.7, cloud_rect_drawn.center().y),
            egui::pos2(cloud_rect_drawn.max.x + tail,       cloud_rect_drawn.center().y + tail * 0.5),
        )
    } else {
        // облако справа → хвост влево, к картинке
        (
            egui::pos2(cloud_rect_drawn.min.x,              cloud_rect_drawn.center().y),
            egui::pos2(cloud_rect_drawn.min.x - tail * 0.7, cloud_rect_drawn.center().y),
            egui::pos2(cloud_rect_drawn.min.x - tail,       cloud_rect_drawn.center().y + tail * 0.5),
        )
    };
    p.add(egui::Shape::Path(egui::epaint::PathShape {
        points: vec![a, b, c],
        closed: true,
        fill: egui::Color32::from_rgb(245, 246, 247),
        stroke: egui::Stroke::new(1.5, egui::Color32::from_gray(180)).into(),
    }));
    
    // Отладочная информация (можно удалить после проверки)
    // eprintln!("Cloud: {:?}, Image: {:?}, Place left: {}", cloud_rect_drawn, image_rect, final_place_left);

    // Возвращаем Rect облака
    cloud_rect_drawn
}

/// Пузырь слева от картинки, ширина ≈ N символов, макс. высота с прокруткой.
/// Рисуется на Foreground-слое, поэтому всегда поверх картинки.
/// @deprecated Используйте show_talk_cloud_side вместо этого
#[allow(dead_code)]
pub fn show_talk_cloud_widget(
    _ctx: &egui::Context,
    _text: &str,
    _image_rect: egui::Rect,
    _max_chars_per_line: usize,
    _max_height_px: f32,
    _gap: f32,
    _font: egui::FontId,
) {
    // Deprecated - используйте show_talk_cloud_side
}

/// Рисует облако с текстом (речевой пузырь) с приоритетом размещения слева
/// Сначала пытается уместить слева, уменьшая ширину за счёт переноса строк
/// Только при недостатке места (< 80px) перемещает справа
pub fn draw_talk_cloud_left(
    painter: &egui::Painter,
    text: &str,
    image_rect: egui::Rect,
    screen_rect: egui::Rect,
    max_width: f32,
    gap: f32,
) {
    let pad = 12.0;
    let r = 8.0;
    let tail = 10.0;
    let font = egui::FontId::proportional(14.0);

    // Сколько места есть слева от картинки
    let space_left = (image_rect.min.x - screen_rect.min.x - gap).max(0.0);
    let wrap_left = space_left.min(max_width);

    // Принудительно размещаем слева - уменьшаем минимальный порог еще больше
    // Чтобы почти всегда размещать слева (только если места меньше 30px)
    let place_left = wrap_left >= 30.0;
    let wrap_w = if place_left {
        wrap_left
    } else {
        (screen_rect.max.x - image_rect.max.x - gap).max(120.0).min(max_width)
    };

    // Точный размер текста с переносами
    // Убеждаемся что wrap_w не слишком маленький
    let wrap_w_final = wrap_w.max(80.0);
    
    // Создаем galley для текста
    let galley = painter.layout(
        text.to_owned(),
        font.clone(),
        egui::Color32::TRANSPARENT,
        wrap_w_final,
    );
    
    let galley_size = galley.size();
    // Используем реальный размер galley, если он валидный, иначе рассчитываем приблизительно
    let text_size = if galley_size.x > 1.0 && galley_size.y > 1.0 {
        galley_size
    } else {
        // Fallback: приблизительный расчет
        let char_count = text.chars().count();
        let approx_width = (char_count as f32 * 7.0).min(wrap_w_final);
        egui::vec2(approx_width, 20.0)
    };
    let cloud_size = text_size + egui::vec2(pad * 2.0, pad * 2.0);

    // Позиция облака (по умолчанию слева)
    let mut cloud_min = if place_left {
        egui::pos2(
            image_rect.min.x - gap - cloud_size.x,
            image_rect.center().y - cloud_size.y / 2.0,
        )
    } else {
        egui::pos2(
            image_rect.max.x + gap,
            image_rect.center().y - cloud_size.y / 2.0,
        )
    };

    // Вписываем по экрану по Y и X
    if cloud_min.y < screen_rect.min.y {
        cloud_min.y = screen_rect.min.y + 5.0;
    }
    if cloud_min.y + cloud_size.y > screen_rect.max.y {
        cloud_min.y = screen_rect.max.y - cloud_size.y - 5.0;
    }
    if place_left && cloud_min.x < screen_rect.min.x {
        cloud_min.x = screen_rect.min.x + 5.0;
    }
    // Если справа и выходит за правую границу - корректируем
    if !place_left && cloud_min.x + cloud_size.x > screen_rect.max.x {
        cloud_min.x = screen_rect.max.x - cloud_size.x - 5.0;
    }

    let cloud_rect = egui::Rect::from_min_size(cloud_min, cloud_size);

    // Тень → фон → обводка
    painter.rect_filled(
        cloud_rect.translate(egui::vec2(2.0, 2.0)),
        r,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30),
    );
    painter.rect_filled(cloud_rect, r, egui::Color32::from_rgb(255, 255, 255));
    painter.rect_stroke(
        cloud_rect,
        r,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(200, 200, 200)),
        egui::epaint::StrokeKind::Outside,
    );

    // Хвостик к картинке
    let (a, b, c) = if place_left {
        (
            egui::pos2(cloud_rect.max.x, cloud_rect.center().y),
            egui::pos2(cloud_rect.max.x + tail * 0.7, cloud_rect.center().y),
            egui::pos2(cloud_rect.max.x + tail, cloud_rect.center().y + tail * 0.5),
        )
    } else {
        (
            egui::pos2(cloud_rect.min.x, cloud_rect.center().y),
            egui::pos2(cloud_rect.min.x - tail * 0.7, cloud_rect.center().y),
            egui::pos2(cloud_rect.min.x - tail, cloud_rect.center().y + tail * 0.5),
        )
    };
    painter.add(egui::Shape::Path(egui::epaint::PathShape {
        points: vec![a, b, c],
        closed: true,
        fill: egui::Color32::from_rgb(255, 255, 255),
        stroke: egui::Stroke::new(1.5, egui::Color32::from_rgb(200, 200, 200)).into(),
    }));

    // Текст - всегда используем прямой text() для надежности на layer_painter
    if !text.is_empty() {
        // Рисуем текст по центру облака
        painter.text(
            cloud_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font.clone(),
            egui::Color32::from_rgb(50, 50, 50),
        );
    }
}

/// Рисует облако с текстом (речевой пузырь) слева от указанной позиции
/// Учитывает границы окна и корректирует позицию при необходимости
#[allow(dead_code)]
pub fn draw_talk_cloud(
    painter: &egui::Painter,
    text: &str,
    image_rect: egui::Rect,
    _image_size: egui::Vec2,
    available_rect: egui::Rect,
) {
    // Размеры облака
    let padding = 12.0;
    let corner_radius = 8.0;
    
    // Вычисляем размер текста точно через galley
    let font = egui::FontId::proportional(14.0);
    let galley = painter.layout_no_wrap(
        text.to_owned(),
        font.clone(),
        egui::Color32::TRANSPARENT, // Цвет не важен для расчета размера
    );
    
    // Размеры облака на основе точного размера текста
    let text_width = galley.size().x;
    let text_height = galley.size().y;
    let cloud_width = text_width + padding * 2.0;
    let cloud_height = text_height + padding * 2.0;
    
    // Позиция облака: слева от изображения, выровнено по вертикали
    let mut cloud_x = image_rect.min.x - cloud_width - 10.0; // Отступ от изображения
    let mut cloud_y = image_rect.center().y - cloud_height / 2.0;
    
    // Проверяем границы окна и корректируем позицию
    // Если облако выходит за левую границу - перемещаем справа от изображения
    if cloud_x < available_rect.min.x {
        cloud_x = image_rect.max.x + 10.0; // Справа от изображения
    }
    
    // Проверяем правую границу
    if cloud_x + cloud_width > available_rect.max.x {
        cloud_x = available_rect.max.x - cloud_width - 5.0; // Отступ от правого края
    }
    
    // Проверяем верхнюю и нижнюю границы
    if cloud_y < available_rect.min.y {
        cloud_y = available_rect.min.y + 5.0; // Отступ от верхнего края
    }
    if cloud_y + cloud_height > available_rect.max.y {
        cloud_y = available_rect.max.y - cloud_height - 5.0; // Отступ от нижнего края
    }
    
    let cloud_rect = egui::Rect::from_min_size(
        egui::pos2(cloud_x, cloud_y),
        egui::vec2(cloud_width, cloud_height),
    );
    
    // Лёгкая тень пузырю (чтобы «оторвать» от фона) - рисуем простую тень вручную
    let shadow_offset = 2.0;
    let shadow_rect = cloud_rect.translate(egui::vec2(shadow_offset, shadow_offset));
    painter.rect_filled(shadow_rect, corner_radius, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30));
    
    // Рисуем облако с закругленными углами (белый фон)
    painter.rect_filled(cloud_rect, corner_radius, egui::Color32::from_rgb(255, 255, 255));
    
    // Рисуем границу облака (только обводка, без заливки)
    painter.rect_stroke(
        cloud_rect,
        corner_radius,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(200, 200, 200)),
        egui::epaint::StrokeKind::Outside, // Тип обводки: внешняя граница
    );
    
    // Рисуем хвостик облака, указывающий на изображение
    // Хвостик направлен к изображению (слева или справа в зависимости от позиции облака)
    let tail_size = 10.0;
    let (tail_start, tail_middle, tail_end) = if cloud_x < image_rect.min.x {
        // Облако слева от изображения - хвостик направлен вправо
        (
            egui::pos2(cloud_rect.max.x, cloud_rect.center().y),
            egui::pos2(cloud_rect.max.x + tail_size * 0.7, cloud_rect.center().y),
            egui::pos2(
                cloud_rect.max.x + tail_size,
                cloud_rect.center().y + tail_size * 0.5,
            ),
        )
    } else {
        // Облако справа от изображения - хвостик направлен влево
        (
            egui::pos2(cloud_rect.min.x, cloud_rect.center().y),
            egui::pos2(cloud_rect.min.x - tail_size * 0.7, cloud_rect.center().y),
            egui::pos2(
                cloud_rect.min.x - tail_size,
                cloud_rect.center().y + tail_size * 0.5,
            ),
        )
    };
    
    // Хвостик как треугольник (заполненный)
    let tail_points = vec![tail_start, tail_middle, tail_end];
    
    // Рисуем заполненный треугольник через Path
    painter.add(egui::Shape::Path(egui::epaint::PathShape {
        points: tail_points.clone(),
        closed: true,
        fill: egui::Color32::from_rgb(255, 255, 255),
        stroke: egui::Stroke::new(1.5, egui::Color32::from_rgb(200, 200, 200)).into(),
    }));
    
    // Рисуем текст используя galley для точного отображения
    // Центрируем galley в облаке
    let text_pos = egui::pos2(
        cloud_rect.center().x - galley.size().x / 2.0,
        cloud_rect.center().y - galley.size().y / 2.0,
    );
    painter.galley(text_pos, galley, egui::Color32::from_rgb(50, 50, 50));
}
