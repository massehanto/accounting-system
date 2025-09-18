pub fn validate_required(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        Some("This field is required".to_string())
    } else {
        None
    }
}

pub fn validate_email(value: &str) -> Option<String> {
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

pub fn validate_npwp(value: &str) -> Option<String> {
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

pub fn validate_currency(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    
    if value.parse::<f64>().is_ok() {
        None
    } else {
        Some("Please enter a valid amount".to_string())
    }
}