use reqwest::blocking::Client;
use std::{io::Read, str::FromStr, time::Duration};

pub struct Product {
    pub name: [u8; 256],
    pub name_length: usize,
    pub code: [u8; 64],
    pub code_length: usize,
    pub price: f32,
}

pub struct MyData {
    pub min: Product,
    pub max: Product,

    pub price: f32,

    pub count: u64,
    pub sum: u64,

    pub warehouse: u32,
    pub carousel: u32,
    pub rrc: u32,
}

impl MyData {
    // Структура за замовчуванням
    fn new() -> Self {
        Self {
            min: Product {
                name: [0; 256],
                name_length: 0,
                code: [0; 64],
                code_length: 0,
                price: std::f32::MAX,
            },
            max: Product {
                name: [0; 256],
                name_length: 0,
                code: [0; 64],
                code_length: 0,
                price: 0.0,
            },
            price: 0.0,
            count: 0,
            sum: 0,
            warehouse: 0,
            carousel: 0,
            rrc: 0,
        }
    }

    // Мінімальна ціна разом з кодом товару і назвою
    fn update_min_product(&mut self, price: f32, code_bytes: &[u8], name_bytes: &[u8]) {
        if price < self.min.price {
            self.min.price = price;

            let len = code_bytes.len().min(self.min.code.len());
            self.min.code[..len].copy_from_slice(&code_bytes[..len]);
            self.min.code_length = len;

            let len = name_bytes.len().min(self.min.name.len());
            self.min.name[..len].copy_from_slice(&name_bytes[..len]);
            self.min.name_length = len;
        }
    }

    // Максимальна ціна разом з кодом товару і назвою
    fn update_max_product(&mut self, price: f32, code_bytes: &[u8], name_bytes: &[u8]) {
        if price > self.max.price {
            self.max.price = price;

            let len = code_bytes.len().min(self.max.code.len());
            self.max.code[..len].copy_from_slice(&code_bytes[..len]);
            self.max.code_length = len;

            let len = name_bytes.len().min(self.max.name.len());
            self.max.name[..len].copy_from_slice(&name_bytes[..len]);
            self.max.name_length = len;
        }
    }

    // Середня ціна товару
    fn update_avg_product(&mut self) {
        self.price = (self.sum / self.count) as f32;
    }

    // Загальна кількість товарів
    fn update_count(&mut self) {
        self.count += 1;
    }

    // Загальна сума товарів
    fn update_sum(&mut self, price: f32) {
        self.sum += price as u64;
    }

    // Кількість на складі
    fn update_warehouse(&mut self, stock: u8) {
        if stock == 1 {
            self.warehouse += 1;
        }
    }

    // Кількість в каруселі
    fn update_carousel(&mut self, stock: u8, day: u8) {
        if stock == 0 && day > 0 {
            self.carousel += 1;
        }
    }

    // Кількість РІЦ
    fn update_rrc(&mut self, stock: u8, day: u8) {
        if stock == 0 && day == 0 {
            self.rrc += 1;
        }
    }
}

// Власний вивід для товарів з мінімальною і максимальною ціною
impl std::fmt::Debug for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = std::str::from_utf8(&self.name[..self.name_length]).unwrap_or("<Invalid UTF-8>");
        let code = std::str::from_utf8(&self.code[..self.code_length]).unwrap_or("<Invalid UTF-8>");

        f.debug_struct("")
            .field("Назва", &name)
            .field("Код", &code)
            .field("Ціна", &self.price)
            .finish()
    }
}

// Власний вивід для всіх даних
impl std::fmt::Debug for MyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("")
            .field("Всього товарів", &self.count)
            .field("Середня ціна товару", &self.price)
            .field("Кількість на складі", &self.warehouse)
            .field("Кількість в каруселі", &self.carousel)
            .field("Кількість РІЦ", &self.rrc)
            .field("Мінімальна ціна товару", &self.min)
            .field("Максимальна ціна товару", &self.max)
            .finish()
    }
}

// Знаходимо в кожному товарі потрібні поля
fn process_part(part: &[u8], data: &mut MyData) -> Option<bool> {
    // Шукаємо код товару
    let needle = b"\"Code\":\"";
    let start = part
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| index + needle.len())
        .unwrap_or(0);
    let end = part[start..]
        .iter()
        .position(|&byte| byte == b'"')
        .map(|index| start + index)
        .unwrap_or(part.len());
    let code = &part[start..end];

    // Шукаємо назву товару
    let needle = b"\"Name\":\"";
    let start = part
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| index + needle.len())
        .unwrap_or(0);
    let end = part[start..]
        .iter()
        .position(|&byte| byte == b'"')
        .map(|index| start + index)
        .unwrap_or(part.len());
    let name = &part[start..end];

    // Шукаємо навність
    let needle = b"\"Stock\":\"";
    let start = part
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| index + needle.len())
        .unwrap_or(0);
    let end = part[start..]
        .iter()
        .position(|&byte| byte == b'"')
        .map(|index| start + index)
        .unwrap_or(part.len());
    let stock = &part[start..end];

    let stock = match std::str::from_utf8(stock) {
        Ok(s) => match u8::from_str(s) {
            Ok(stock) => stock,
            Err(e) => {
                println!("Stock digit: {:?} {:?}", e, s);
                return None;
            }
        },
        Err(e) => {
            println!("Stock UTF 8: {:?}", e);
            return None;
        }
    };

    // Шукаємо дні доставки
    let needle = b"\"DayDelivery\":\"";
    let start = part
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| index + needle.len())
        .unwrap_or(0);
    let end = part[start..]
        .iter()
        .position(|&byte| byte == b'"')
        .map(|index| start + index)
        .unwrap_or(part.len());
    let day = &part[start..end];

    let day = match std::str::from_utf8(day) {
        Ok(s) => match u8::from_str(s) {
            Ok(day) => day,
            Err(e) => {
                println!("Day digit: {:?} {}", e, s);
                return None;
            }
        },
        Err(e) => {
            println!("Day UTF 8: {:?}", e);
            return None;
        }
    };

    // Шукаємо ціну
    let needle = b"\"RetailPrice\":";
    let start = part
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| index + needle.len())
        .unwrap_or(0);
    let end = part[start..]
        .iter()
        .position(|&byte| byte == b',')
        .map(|index| start + index)
        .unwrap_or(part.len());
    let price = &part[start..end];

    let price = match std::str::from_utf8(price) {
        Ok(s) => match f32::from_str(s) {
            Ok(price) => price,
            Err(e) => {
                println!("Price digit: {:?} {}", e, s);
                return None;
            }
        },
        Err(e) => {
            println!("Price UTF 8: {:?}", e);
            return None;
        }
    };

    data.update_count();
    data.update_sum(price);
    data.update_warehouse(stock);
    data.update_carousel(stock, day);
    data.update_rrc(stock, day);
    data.update_min_product(price, code, name);
    data.update_max_product(price, code, name);

    Some(true)
}

fn main() {
    let token = "";
    let url = "https://pricelist.brain.com.ua/index.php?time=1685452584&companyID=5260&userID=10788&targetID=29&format=json&lang=ua&full=1&token=".to_owned() + token;
    
    let client = match Client::builder()
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(300))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Common error. Error text: {}", e);
            return;
        }
    };
    let mut response = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Common error. Error text: {}", e);
            return;
        }
    };

    // Отримуємо структуру за замовчуванням
    let mut data = MyData::new();

    // Довжина записаних даних в буфер
    let mut buffer_length = 0;

    // Буфер для зберігання даних
    let mut buffer = [0; 8196];

    // Роздільник
    let delimiter = b"},";

    // Читання даних по частинах
    loop {
        match response.read(&mut buffer[buffer_length..]) {
            Ok(0) => break, // Кінець даних
            Ok(n) => {
                buffer_length += n;

                // Обробляємо дані до роздільника
                let mut index = 0;
                while let Some(pos) = buffer[index..buffer_length]
                    .windows(delimiter.len())
                    .position(|window| window == delimiter)
                {
                    let part = &buffer[index..index + pos + delimiter.len()];
                    let status = process_part(part, &mut data);
                    if status.is_none() {
                        println!("Щось пішло не так");
                        return;
                    }
                    index += pos + delimiter.len();
                }

                // Якщо буфер закінчився і роздільника не знайшли
                // Переміщуємо решту даних на початок буферу, потім до цих даних продовжимо додавати дані з потоку
                let remaining_bytes = buffer_length - index;
                buffer.copy_within(index..buffer_length, 0);
                buffer_length = remaining_bytes;
            }
            Err(error) => {
                eprintln!("Error reading file: {}", error);
                break;
            }
        }
    }

    // Решта даних
    if buffer_length > 0 {
        let part = &buffer[..buffer_length];
        let status = process_part(part, &mut data);
        if status.is_none() {
            println!("Щось пішло не так");
            return;
        }
    }

    data.update_avg_product();

    println!("{:#?}", data);
}
