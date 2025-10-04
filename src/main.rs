use screenshots::Screen;
use image::{ImageBuffer, Rgba};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct CursorPosition {
    x: i32,
    y: i32,
}

struct CaptureManager {
    output_dir: String,
}

impl CaptureManager {
    fn new(output_dir: &str) -> Self {
        // Создаем директорию если не существует
        let path = Path::new(output_dir);
        if !path.exists() {
            std::fs::create_dir_all(path).expect("Не удалось создать директорию");
            println!("📁 Создана директория: {}", output_dir);
        }

        CaptureManager {
            output_dir: output_dir.to_string(),
        }
    }

    fn generate_filename(&self, x: i32, y: i32) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        format!(
            "{}/capture_{}_{}_{}.png",
            self.output_dir, timestamp, x, y
        )
    }

    fn capture_area_around_cursor(&self, x: i32, y: i32, radius: i32) -> Result<String, Box<dyn std::error::Error>> {
        println!("🎯 Захват области вокруг позиции: ({}, {}) радиус: {}", x, y, radius);

        // Получаем все экраны
        let screens = Screen::all()?;
        if screens.is_empty() {
            return Err("Не найдено ни одного экрана".into());
        }

        // Для простоты используем основной экран (первый)
        let screen = &screens[0];
        let screen_info = screen.display_info;

        println!("📺 Используем экран: {}x{}", screen_info.width, screen_info.height);

        // Рассчитываем область захвата
        let capture_x = (x - radius).max(0);
        let capture_y = (y - radius).max(0);

        // Проверяем границы экрана
        let end_x = (x + radius).min(screen_info.width as i32);
        let end_y = (y + radius).min(screen_info.height as i32);

        let actual_width = (end_x - capture_x).max(1);
        let actual_height = (end_y - capture_y).max(1);

        println!("📷 Область захвата: ({}, {}) размер: {}x{}",
                 capture_x, capture_y, actual_width, actual_height);

        // Захватываем область
        let capture = screen.capture_area(capture_x, capture_y, actual_width as u32, actual_height as u32)?;

        // Создаем изображение из данных
        let image_buffer = ImageBuffer::from_raw(
            capture.width() as u32,
            capture.height() as u32,
            capture.to_vec()
        ).ok_or("Не удалось создать изображение из данных захвата")?;

        // Генерируем имя файла и сохраняем
        let filename = self.generate_filename(x, y);
        image_buffer.save(&filename)?;

        println!("✅ Изображение сохранено: {}", filename);

        // Анализируем цвет в центре
        self.analyze_center_color(&image_buffer);

        Ok(filename)
    }

    fn analyze_center_color(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let center_x = image.width() / 2;
        let center_y = image.height() / 2;

        if center_x < image.width() && center_y < image.height() {
            let pixel = image.get_pixel(center_x, center_y);
            let color = [pixel[0], pixel[1], pixel[2], pixel[3]];

            println!("🎨 Цвет в центре: RGB({}, {}, {})", color[0], color[1], color[2]);
            println!("   Примерный цвет: {}", self.approximate_color_name(color[0], color[1], color[2]));
        }
    }

    fn approximate_color_name(&self, r: u8, g: u8, b: u8) -> String {
        match (r, g, b) {
            (r, g, b) if r > 200 && g < 100 && b < 100 => "Красный".to_string(),
            (r, g, b) if r < 100 && g > 200 && b < 100 => "Зеленый".to_string(),
            (r, g, b) if r < 100 && g < 100 && b > 200 => "Синий".to_string(),
            (r, g, b) if r > 200 && g > 200 && b < 100 => "Желтый".to_string(),
            (r, g, b) if r < 100 && g > 200 && b > 200 => "Голубой".to_string(),
            (r, g, b) if r > 200 && g < 100 && b > 200 => "Пурпурный".to_string(),
            (r, g, b) if r > 200 && g > 200 && b > 200 => "Белый".to_string(),
            (r, g, b) if r < 50 && g < 50 && b < 50 => "Черный".to_string(),
            (r, g, b) if r == g && g == b => format!("Серый {}", r),
            _ => format!("RGB({}, {}, {})", r, g, b),
        }
    }

    fn print_screen_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        let screens = Screen::all()?;
        println!("🖥️  Обнаружено экранов: {}", screens.len());

        for (i, screen) in screens.iter().enumerate() {
            let info = screen.display_info;
            println!("   Экран {}: {}x{} (позиция: {}, {})",
                     i, info.width, info.height, info.x, info.y);
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запуск захвата областей экрана...");

    // Инициализируем менеджер захватов
    let capture_manager = CaptureManager::new("./screenshots");

    // Показываем информацию об экранах
    capture_manager.print_screen_info()?;

    println!("\n💡 Будут захвачены области вокруг следующих позиций:");

    // Тестовые позиции для захвата (измените на нужные вам координаты)
    let test_positions = vec![
        CursorPosition { x: 500, y: 300 },
        CursorPosition { x: 600, y: 400 },
        CursorPosition { x: 700, y: 500 },
        CursorPosition { x: 800, y: 300 },
        CursorPosition { x: 900, y: 400 },
    ];

    for (i, pos) in test_positions.iter().enumerate() {
        println!("   {}. ({}, {})", i + 1, pos.x, pos.y);
    }

    println!("\n🎮 Начинаем захват через 3 секунды...");
    thread::sleep(Duration::from_secs(3));

    println!("--------------------------------------------------");

    // Выполняем захваты для каждой позиции
    for (i, cursor_pos) in test_positions.iter().enumerate() {
        println!("\n📸 Захват #{} из {}", i + 1, test_positions.len());

        match capture_manager.capture_area_around_cursor(cursor_pos.x, cursor_pos.y, 50) {
            Ok(filename) => {
                println!("💾 Успешно: {}", filename);
            }
            Err(e) => {
                eprintln!("❌ Ошибка: {}", e);
            }
        }

        // Пауза между захватами (кроме последнего)
        if i < test_positions.len() - 1 {
            println!("⏳ Ожидание 2 секунды...");
            thread::sleep(Duration::from_secs(2));
        }
    }

    println!("\n--------------------------------------------------");
    println!("✅ Все захваты завершены! Проверьте папку './screenshots'");

    Ok(())
}