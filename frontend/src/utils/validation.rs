use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_npwp(npwp: &str) -> bool {
    let npwp_regex = Regex::new(r"^\d{2}\.\d{3}\.\d{3}\.\d{1}-\d{3}\.\d{3}$").unwrap();
    npwp_regex.is_match(npwp)
}

pub fn validate_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^(\+62|0)[0-9]{8,13}$").unwrap();
    phone_regex.is_match(phone)
}

pub fn validate_currency(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

pub fn validate_required(value: &str) -> bool {
    !value.trim().is_empty()
}