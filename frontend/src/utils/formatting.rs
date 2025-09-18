pub fn format_currency(amount: f64) -> String {
    let formatted = format!("{:.2}", amount);
    let parts: Vec<&str> = formatted.split('.').collect();
    let whole = parts[0];
    let decimal = if parts.len() > 1 { parts[1] } else { "00" };
    
    let mut result = String::new();
    let chars: Vec<char> = whole.chars().collect();
    
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(*ch);
    }
    
    format!("Rp {},{}", result, decimal)
}

pub fn format_date(date_str: &str) -> String {
    if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return parsed.format("%d/%m/%Y").to_string();
    }
    
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return parsed.format("%d/%m/%Y").to_string();
    }
    
    date_str.to_string()
}

pub fn format_npwp(npwp: &str) -> String {
    let clean = npwp.replace(&['.', '-'][..], "");
    
    if clean.len() == 15 {
        format!("{}.{}.{}.{}-{}.{}", 
            &clean[0..2], &clean[2..5], &clean[5..8], 
            &clean[8..9], &clean[9..12], &clean[12..15])
    } else {
        npwp.to_string()
    }
}