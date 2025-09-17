use chrono::NaiveDate;
use rust_decimal::Decimal;

pub struct IndonesianFormatter;

impl IndonesianFormatter {
    /// Formats currency in Indonesian Rupiah format
    /// Example: 1234567.89 -> "Rp 1.234.567,89"
    pub fn format_currency(amount: Decimal) -> String {
        let formatted = Self::format_number_indonesian(amount);
        format!("Rp {}", formatted)
    }

    /// Formats number in Indonesian format (dot as thousands separator, comma as decimal)
    /// Example: 1234567.89 -> "1.234.567,89"
    pub fn format_number_indonesian(amount: Decimal) -> String {
        let amount_str = amount.to_string();
        let parts: Vec<&str> = amount_str.split('.').collect();
        
        let integer_part = parts[0];
        let decimal_part = if parts.len() > 1 { parts[1] } else { "00" };
        
        // Add thousand separators
        let formatted_integer = Self::add_thousand_separators(integer_part, ".");
        
        if decimal_part == "00" || decimal_part.is_empty() {
            formatted_integer
        } else {
            format!("{},{}", formatted_integer, decimal_part)
        }
    }

    /// Formats number in international format (comma as thousands separator, dot as decimal)
    /// Example: 1234567.89 -> "1,234,567.89"
    pub fn format_number_international(amount: Decimal) -> String {
        let amount_str = amount.to_string();
        let parts: Vec<&str> = amount_str.split('.').collect();
        
        let integer_part = parts[0];
        let decimal_part = if parts.len() > 1 { parts[1] } else { "" };
        
        let formatted_integer = Self::add_thousand_separators(integer_part, ",");
        
        if decimal_part.is_empty() {
            formatted_integer
        } else {
            format!("{}.{}", formatted_integer, decimal_part)
        }
    }

    fn add_thousand_separators(number_str: &str, separator: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = number_str.chars().collect();
        let len = chars.len();
        
        for (i, ch) in chars.iter().enumerate() {
            result.push(*ch);
            let remaining = len - i - 1;
            if remaining > 0 && remaining % 3 == 0 {
                result.push_str(separator);
            }
        }
        
        result
    }

    /// Formats Indonesian date (DD/MM/YYYY)
    pub fn format_date_indonesian(date: NaiveDate) -> String {
        date.format("%d/%m/%Y").to_string()
    }

    /// Formats international date (YYYY-MM-DD)
    pub fn format_date_international(date: NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    /// Formats Indonesian date with month name
    /// Example: "15 Januari 2024"
    pub fn format_date_indonesian_long(date: NaiveDate) -> String {
        let month_names = [
            "Januari", "Februari", "Maret", "April", "Mei", "Juni",
            "Juli", "Agustus", "September", "Oktober", "November", "Desember"
        ];
        
        let month_name = month_names[date.month() as usize - 1];
        format!("{} {} {}", date.day(), month_name, date.year())
    }

    /// Formats NPWP with standard formatting
    /// Example: "123456789012345" -> "12.345.678.9-012.345"
    pub fn format_npwp(npwp: &str) -> String {
        let clean_npwp: String = npwp.chars().filter(|c| c.is_ascii_digit()).collect();
        
        if clean_npwp.len() != 15 {
            return npwp.to_string(); // Return as-is if invalid
        }
        
        format!(
            "{}.{}.{}.{}-{}.{}",
            &clean_npwp[0..2],
            &clean_npwp[2..5],
            &clean_npwp[5..8],
            &clean_npwp[8..9],
            &clean_npwp[9..12],
            &clean_npwp[12..15]
        )
    }

    /// Formats Indonesian phone number
    /// Example: "628123456789" -> "+62 812-3456-789"
    pub fn format_phone_number(phone: &str) -> String {
        let clean_phone: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        
        if clean_phone.starts_with("62") {
            let number = &clean_phone[2..];
            if number.len() >= 9 {
                return format!("+62 {}-{}-{}", 
                    &number[0..3], 
                    &number[3..7], 
                    &number[7..]
                );
            }
        } else if clean_phone.starts_with("08") {
            let number = &clean_phone[1..];
            if number.len() >= 9 {
                return format!("0{}-{}-{}", 
                    &number[0..3], 
                    &number[3..7], 
                    &number[7..]
                );
            }
        }
        
        phone.to_string() // Return as-is if can't format
    }
}

pub struct TextFormatter;

impl TextFormatter {
    /// Converts text to title case
    pub fn to_title_case(text: &str) -> String {
        text.split_whitespace()
            .map(|word| {
                let mut chars: Vec<char> = word.chars().collect();
                if !chars.is_empty() {
                    chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
                    for i in 1..chars.len() {
                        chars[i] = chars[i].to_lowercase().next().unwrap_or(chars[i]);
                    }
                }
                chars.into_iter().collect::<String>()
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Generates a slug from text
    pub fn to_slug(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }

    /// Truncates text to specified length with ellipsis
    pub fn truncate(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length.saturating_sub(3)])
        }
    }
}