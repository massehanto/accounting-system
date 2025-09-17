use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct IndonesianValidator;

impl IndonesianValidator {
    /// Validates Indonesian NPWP (Nomor Pokok Wajib Pajak)
    /// Format: XX.XXX.XXX.X-XXX.XXX or 15 digits
    pub fn validate_npwp(npwp: &str) -> bool {
        let clean_npwp: String = npwp.chars().filter(|c| c.is_ascii_digit()).collect();
        
        if clean_npwp.len() != 15 {
            return false;
        }

        // Additional NPWP validation rules
        // First 2 digits should be 01-39 (individual) or 02,31 (corporate)
        if let Ok(first_two) = clean_npwp[0..2].parse::<u32>() {
            first_two >= 1 && first_two <= 39
        } else {
            false
        }
    }

    /// Validates Indonesian phone number
    /// Formats: +62, 62, 08, 8 followed by 8-12 digits
    pub fn validate_phone_number(phone: &str) -> bool {
        let phone_regex = Regex::new(r"^(\+62|62|0)?8[1-9][0-9]{6,11}$").unwrap();
        phone_regex.is_match(phone)
    }

    /// Validates Indonesian postal code (5 digits)
    pub fn validate_postal_code(postal_code: &str) -> bool {
        let postal_regex = Regex::new(r"^\d{5}$").unwrap();
        postal_regex.is_match(postal_code)
    }

    /// Validates Indonesian bank account number
    pub fn validate_bank_account(account_number: &str, bank_code: Option<&str>) -> bool {
        let clean_account: String = account_number.chars().filter(|c| c.is_ascii_digit()).collect();
        
        // Basic validation: 10-16 digits
        if clean_account.len() < 10 || clean_account.len() > 16 {
            return false;
        }

        // Bank-specific validation
        if let Some(bank) = bank_code {
            match bank.to_uppercase().as_str() {
                "BCA" => clean_account.len() >= 10 && clean_account.len() <= 10,
                "BNI" => clean_account.len() >= 10 && clean_account.len() <= 10,
                "BRI" => clean_account.len() >= 15 && clean_account.len() <= 15,
                "MANDIRI" => clean_account.len() >= 13 && clean_account.len() <= 13,
                _ => true, // Generic validation for other banks
            }
        } else {
            true
        }
    }

    /// Validates business license number (NIB - Nomor Induk Berusaha)
    pub fn validate_nib(nib: &str) -> bool {
        let clean_nib: String = nib.chars().filter(|c| c.is_ascii_digit()).collect();
        clean_nib.len() == 13
    }
}

pub struct AmountValidator;

impl AmountValidator {
    /// Validates that amount is positive
    pub fn is_positive(amount: Decimal) -> bool {
        amount > Decimal::ZERO
    }

    /// Validates that amount is not negative
    pub fn is_non_negative(amount: Decimal) -> bool {
        amount >= Decimal::ZERO
    }

    /// Validates amount precision (max decimal places)
    pub fn validate_precision(amount: Decimal, max_decimals: u32) -> bool {
        let scale = amount.scale();
        scale <= max_decimals
    }

    /// Validates amount range
    pub fn validate_range(amount: Decimal, min: Option<Decimal>, max: Option<Decimal>) -> bool {
        if let Some(min_val) = min {
            if amount < min_val {
                return false;
            }
        }

        if let Some(max_val) = max {
            if amount > max_val {
                return false;
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, field: &str, message: &str) {
        self.is_valid = false;
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        });
    }

    pub fn combine(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}

pub fn validate_required_fields(data: &HashMap<String, Option<String>>) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    for (field, value) in data {
        if value.is_none() || value.as_ref().unwrap().trim().is_empty() {
            result.add_error(field, "This field is required");
        }
    }
    
    result
}