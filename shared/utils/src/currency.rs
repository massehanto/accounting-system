use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::HashMap;

pub struct CurrencyUtils;

impl CurrencyUtils {
    /// Rounds amount to currency precision (2 decimal places for most currencies)
    pub fn round_to_currency(amount: Decimal, currency: &str) -> Decimal {
        let precision = Self::get_currency_precision(currency);
        amount.round_dp_with_strategy(precision, RoundingStrategy::MidpointNearestEven)
    }

    /// Gets the precision (decimal places) for a currency
    pub fn get_currency_precision(currency: &str) -> u32 {
        match currency.to_uppercase().as_str() {
            "IDR" => 0, // Indonesian Rupiah doesn't use decimals in practice
            "JPY" => 0, // Japanese Yen
            "KRW" => 0, // Korean Won
            "VND" => 0, // Vietnamese Dong
            "CLP" => 0, // Chilean Peso
            "ISK" => 0, // Icelandic Krona
            "BHD" => 3, // Bahraini Dinar
            "IQD" => 3, // Iraqi Dinar
            "JOD" => 3, // Jordanian Dinar
            "KWD" => 3, // Kuwaiti Dinar
            "LYD" => 3, // Libyan Dinar
            "OMR" => 3, // Omani Rial
            "TND" => 3, // Tunisian Dinar
            _ => 2,     // Most other currencies use 2 decimal places
        }
    }

    /// Gets currency symbol
    pub fn get_currency_symbol(currency: &str) -> &'static str {
        match currency.to_uppercase().as_str() {
            "IDR" => "Rp",
            "USD" => "$",
            "EUR" => "€",
            "GBP" => "£",
            "JPY" => "¥",
            "CNY" => "¥",
            "KRW" => "₩",
            "SGD" => "S$",
            "MYR" => "RM",
            "THB" => "฿",
            "PHP" => "₱",
            "VND" => "₫",
            "AUD" => "A$",
            "CAD" => "C$",
            "CHF" => "CHF",
            "HKD" => "HK$",
            "INR" => "₹",
            "PKR" => "Rs",
            "BDT" => "৳",
            "LKR" => "Rs",
            "NPR" => "Rs",
            "MMK" => "K",
            "KHR" => "៛",
            "LAK" => "₭",
            _ => "",
        }
    }

    /// Converts between currencies (placeholder - would need real exchange rates)
    pub fn convert_currency(
        amount: Decimal,
        from_currency: &str,
        to_currency: &str,
        exchange_rate: Option<Decimal>,
    ) -> Result<Decimal, String> {
        if from_currency == to_currency {
            return Ok(amount);
        }

        let rate = exchange_rate.ok_or("Exchange rate required for currency conversion")?;
        
        if rate <= Decimal::ZERO {
            return Err("Exchange rate must be positive".to_string());
        }

        let converted = amount * rate;
        Ok(Self::round_to_currency(converted, to_currency))
    }

    /// Gets supported currencies for Indonesian accounting
    pub fn supported_currencies() -> HashMap<String, &'static str> {
        let mut currencies = HashMap::new();
        
        // Primary currency
        currencies.insert("IDR".to_string(), "Indonesian Rupiah");
        
        // Major international currencies
        currencies.insert("USD".to_string(), "US Dollar");
        currencies.insert("EUR".to_string(), "Euro");
        currencies.insert("GBP".to_string(), "British Pound");
        currencies.insert("JPY".to_string(), "Japanese Yen");
        currencies.insert("CNY".to_string(), "Chinese Yuan");
        currencies.insert("AUD".to_string(), "Australian Dollar");
        currencies.insert("CAD".to_string(), "Canadian Dollar");
        currencies.insert("CHF".to_string(), "Swiss Franc");
        currencies.insert("HKD".to_string(), "Hong Kong Dollar");
        
        // ASEAN currencies
        currencies.insert("SGD".to_string(), "Singapore Dollar");
        currencies.insert("MYR".to_string(), "Malaysian Ringgit");
        currencies.insert("THB".to_string(), "Thai Baht");
        currencies.insert("PHP".to_string(), "Philippine Peso");
        currencies.insert("VND".to_string(), "Vietnamese Dong");
        currencies.insert("MMK".to_string(), "Myanmar Kyat");
        currencies.insert("KHR".to_string(), "Cambodian Riel");
        currencies.insert("LAK".to_string(), "Lao Kip");
        currencies.insert("BND".to_string(), "Brunei Dollar");
        
        currencies
    }

    /// Validates currency code
    pub fn is_valid_currency(currency: &str) -> bool {
        Self::supported_currencies().contains_key(&currency.to_uppercase())
    }

    /// Gets default exchange rates (for demo purposes - would come from API in production)
    pub fn get_default_exchange_rates() -> HashMap<String, Decimal> {
        let mut rates = HashMap::new();
        
        // Rates against IDR (as of example date)
        rates.insert("USD".to_string(), Decimal::new(15000, 0)); // 1 USD = 15,000 IDR
        rates.insert("EUR".to_string(), Decimal::new(16500, 0)); // 1 EUR = 16,500 IDR
        rates.insert("GBP".to_string(), Decimal::new(19000, 0)); // 1 GBP = 19,000 IDR
        rates.insert("JPY".to_string(), Decimal::new(100, 0));   // 1 JPY = 100 IDR
        rates.insert("SGD".to_string(), Decimal::new(11000, 0)); // 1 SGD = 11,000 IDR
        rates.insert("MYR".to_string(), Decimal::new(3500, 0));  // 1 MYR = 3,500 IDR
        rates.insert("THB".to_string(), Decimal::new(450, 0));   // 1 THB = 450 IDR
        rates.insert("PHP".to_string(), Decimal::new(270, 0));   // 1 PHP = 270 IDR
        rates.insert("VND".to_string(), Decimal::new(6, 1));     // 1 VND = 0.6 IDR
        
        rates
    }
}

pub struct IndonesianCurrencyUtils;

impl IndonesianCurrencyUtils {
    /// Converts amount to words in Indonesian
    pub fn amount_to_words(amount: Decimal) -> String {
        let integer_part = amount.trunc().to_string().parse::<i64>().unwrap_or(0);
        Self::number_to_indonesian_words(integer_part)
    }

    fn number_to_indonesian_words(number: i64) -> String {
        if number == 0 {
            return "nol".to_string();
        }

        let ones = ["", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh", "delapan", "sembilan"];
        let teens = ["sepuluh", "sebelas", "dua belas", "tiga belas", "empat belas", "lima belas", 
                     "enam belas", "tujuh belas", "delapan belas", "sembilan belas"];
        let tens = ["", "", "dua puluh", "tiga puluh", "empat puluh", "lima puluh", 
                    "enam puluh", "tujuh puluh", "delapan puluh", "sembilan puluh"];

        fn convert_hundreds(n: i64, ones: &[&str], teens: &[&str], tens: &[&str]) -> String {
            let mut result = String::new();
            
            if n >= 100 {
                let hundreds = n / 100;
                if hundreds == 1 {
                    result.push_str("seratus");
                } else {
                    result.push_str(&format!("{} ratus", ones[hundreds as usize]));
                }
                let remainder = n % 100;
                if remainder > 0 {
                    result.push(' ');
                    result.push_str(&convert_tens(remainder, ones, teens, tens));
                }
            } else {
                result.push_str(&convert_tens(n, ones, teens, tens));
            }
            
            result
        }

        fn convert_tens(n: i64, ones: &[&str], teens: &[&str], tens: &[&str]) -> String {
            if n < 10 {
                ones[n as usize].to_string()
            } else if n < 20 {
                if n == 10 {
                    "sepuluh".to_string()
                } else if n == 11 {
                    "sebelas".to_string()
                } else {
                    teens[(n - 10) as usize].to_string()
                }
            } else {
                let tens_digit = n / 10;
                let ones_digit = n % 10;
                if ones_digit == 0 {
                    tens[tens_digit as usize].to_string()
                } else {
                    format!("{} {}", tens[tens_digit as usize], ones[ones_digit as usize])
                }
            }
        }

        let mut result = String::new();
        let mut num = number;

        if num >= 1_000_000_000 {
            let billions = num / 1_000_000_000;
            result.push_str(&convert_hundreds(billions, &ones, &teens, &tens));
            result.push_str(" miliar");
            num %= 1_000_000_000;
            if num > 0 {
                result.push(' ');
            }
        }

        if num >= 1_000_000 {
            let millions = num / 1_000_000;
            result.push_str(&convert_hundreds(millions, &ones, &teens, &tens));
            result.push_str(" juta");
            num %= 1_000_000;
            if num > 0 {
                result.push(' ');
            }
        }

        if num >= 1_000 {
            let thousands = num / 1_000;
            if thousands == 1 {
                result.push_str("seribu");
            } else {
                result.push_str(&convert_hundreds(thousands, &ones, &teens, &tens));
                result.push_str(" ribu");
            }
            num %= 1_000;
            if num > 0 {
                result.push(' ');
            }
        }

        if num > 0 {
            result.push_str(&convert_hundreds(num, &ones, &teens, &tens));
        }

        result
    }

    /// Formats amount with Indonesian currency format and words
    pub fn format_amount_with_words(amount: Decimal) -> String {
        let formatted_amount = crate::formatting::IndonesianFormatter::format_currency(amount);
        let words = Self::amount_to_words(amount);
        format!("{}\n({})", formatted_amount, words)
    }
}