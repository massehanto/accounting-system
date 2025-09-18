pub fn required(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        Some("This field is required".to_string())
    } else {
        None
    }
}

pub fn email(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if email_regex.is_match(value) {
        None
    } else {
        Some("Please enter a valid email address".to_string())
    }
}

pub fn npwp(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    
    let npwp_regex = regex::Regex::new(r"^\d{2}\.\d{3}\.\d{3}\.\d{1}-\d{3}\.\d{3}$").unwrap();
    if npwp_regex.is_match(value) {
        None
    } else {
        Some("Please enter a valid NPWP format (XX.XXX.XXX.X-XXX.XXX)".to_string())
    }
}

pub fn phone(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    
    let phone_regex = regex::Regex::new(r"^(\+62|0)[0-9]{8,13}$").unwrap();
    if phone_regex.is_match(value) {
        None
    } else {
        Some("Please enter a valid Indonesian phone number".to_string())
    }
}

pub fn currency(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    
    if value.parse::<f64>().is_ok() {
        None
    } else {
        Some("Please enter a valid amount".to_string())
    }
}

pub fn percentage(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    
    if let Ok(val) = value.parse::<f64>() {
        if val >= 0.0 && val <= 100.0 {
            None
        } else {
            Some("Percentage must be between 0 and 100".to_string())
        }
    } else {
        Some("Please enter a valid percentage".to_string())
    }
}