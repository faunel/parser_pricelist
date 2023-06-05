 
# Парсер прайслиста, який не використовує кучу, а тільки стек

Увага. Для роботи коду потрібно вказати актуальний токен в 275 рядку

Виводить наступну інформацію

```
{
    Всьго товарів: 207860,
    Середня ціна товару: 3775.0,
    Кількість на складі: 29221,
    Кількість в каруселі: 178639,
    Кількість РІЦ: 0,
    Мінімальна ціна товару:  {
        Назва: "Стержень кульковий Buromax black, 140мм, JOBMAX (BM.8001-02)",
        Код: "U0184691",
        Ціна: 1.0,
    },
    Максимальна ціна товару:  {
        Назва: "Генератор GenPower GNT 475 380kW (F_141306)",
        Код: "U0792387",
        Ціна: 2780001.0,
    },
}
```

## Залежності
Для роботи програми потрібна бібліотека reqwest

Додайте в файл Cargo.toml

```
[dependencies]
reqwest = { version = "0.11", default-features = false, features = ["stream", "blocking", "rustls-tls"] }
```

## Запуск

Клонувати проект

~~~bash  
git clone https://github.com/faunel/parser_pricelist.git
~~~

Перейти в папку 

~~~bash  
  cd parser_pricelist
~~~

Скомпілювати

~~~bash  
cargo build --release
~~~

Перейти в папку з виконуваним файлом 

~~~bash  
  cd target/release
~~~

Запустити програму

Mac

~~~bash  
  ./parser_pricelist
~~~

Windows

~~~bash  
  ./parser_pricelist.exe
~~~