// frontend/src/utils.rs
use web_sys::{window, Storage};

pub fn set_token(token: &str) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth_token", token);
        }
    }
}

pub fn get_token() -> Option<String> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            return storage.get_item("auth_token").unwrap_or(None);
        }
    }
    None
}

pub fn remove_token() {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth_token");
        }
    }
}

pub fn set_user_info(user_id: &str, company_id: &str) {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("user_id", user_id);
            let _ = storage.set_item("company_id", company_id);
        }
    }
}

pub fn get_user_id() -> Option<String> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            return storage.get_item("user_id").unwrap_or(None);
        }
    }
    None
}

pub fn get_company_id() -> Option<String> {
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            return storage.get_item("company_id").unwrap_or(None);
        }
    }
    None
}

pub fn clear_user_data() {
    remove_token();
    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("user_id");
            let _ = storage.remove_item("company_id");
        }
    }
}

pub fn format_currency(amount: f64) -> String {
    // Indonesian Rupiah formatting
    let formatted = format!("{:.2}", amount);
    let parts: Vec<&str> = formatted.split('.').collect();
    let whole = parts[0];
    let decimal = if parts.len() > 1 { parts[1] } else { "00" };
    
    // Add thousand separators
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

pub fn format_percentage(rate: f64) -> String {
    format!("{:.2}%", rate)
}

pub fn format_date(date_str: &str) -> String {
    // Parse ISO date and format for Indonesian locale (DD/MM/YYYY)
    if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return parsed.format("%d/%m/%Y").to_string();
    }
    
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return parsed.format("%d/%m/%Y").to_string();
    }
    
    // Fallback: return as-is
    date_str.to_string()
}

pub fn format_datetime(datetime_str: &str) -> String {
    // Parse ISO datetime and format for Indonesian locale
    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(datetime_str) {
        return parsed.format("%d/%m/%Y %H:%M").to_string();
    }
    
    if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return parsed.format("%d/%m/%Y %H:%M").to_string();
    }
    
    datetime_str.to_string()
}

pub fn validate_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_npwp(npwp: &str) -> bool {
    // Indonesian NPWP format: XX.XXX.XXX.X-XXX.XXX
    let npwp_regex = regex::Regex::new(r"^\d{2}\.\d{3}\.\d{3}\.\d{1}-\d{3}\.\d{3}$").unwrap();
    npwp_regex.is_match(npwp)
}

pub fn validate_phone(phone: &str) -> bool {
    // Indonesian phone number validation
    let phone_regex = regex::Regex::new(r"^(\+62|0)[0-9]{8,13}$").unwrap();
    phone_regex.is_match(phone)
}

pub fn handle_api_error(error: &str) -> String {
    if error.contains("Network error") {
        "Koneksi jaringan gagal. Periksa koneksi internet Anda.".to_string()
    } else if error.contains("401") || error.contains("Unauthorized") {
        "Sesi Anda telah berakhir. Silakan login kembali.".to_string()
    } else if error.contains("403") || error.contains("Forbidden") {
        "Anda tidak memiliki izin untuk melakukan tindakan ini.".to_string()
    } else if error.contains("404") || error.contains("Not found") {
        "Data yang dicari tidak ditemukan.".to_string()
    } else if error.contains("409") || error.contains("Conflict") {
        "Operasi tidak dapat dilakukan dalam kondisi saat ini.".to_string()
    } else if error.contains("422") || error.contains("Unprocessable Entity") {
        "Data yang dimasukkan tidak valid. Periksa kembali input Anda.".to_string()
    } else if error.contains("500") || error.contains("Internal server error") {
        "Terjadi kesalahan pada server. Silakan coba lagi nanti.".to_string()
    } else if error.contains("502") || error.contains("Bad Gateway") {
        "Layanan sementara tidak tersedia.".to_string()
    } else if error.contains("503") || error.contains("Service Unavailable") {
        "Layanan sedang dalam pemeliharaan.".to_string()
    } else {
        error.to_string()
    }
}

// Journal Entry Status Helpers
pub fn get_status_color(status: &str) -> &'static str {
    match status {
        "DRAFT" => "gray",
        "PENDING_APPROVAL" => "yellow", 
        "APPROVED" => "blue",
        "POSTED" => "green",
        "CANCELLED" => "red",
        _ => "gray",
    }
}

pub fn get_status_display_name(status: &str) -> &'static str {
    match status {
        "DRAFT" => "Draft",
        "PENDING_APPROVAL" => "Menunggu Persetujuan",
        "APPROVED" => "Disetujui",
        "POSTED" => "Telah Diposting",
        "CANCELLED" => "Dibatalkan",
        _ => status,
    }
}

pub fn can_edit_journal_entry(status: &str) -> bool {
    matches!(status, "DRAFT")
}

pub fn can_delete_journal_entry(status: &str) -> bool {
    matches!(status, "DRAFT")
}

pub fn get_available_status_transitions(current_status: &str) -> Vec<(String, String)> {
    match current_status {
        "DRAFT" => vec![
            ("PENDING_APPROVAL".to_string(), "Ajukan Persetujuan".to_string()),
            ("CANCELLED".to_string(), "Batalkan".to_string())
        ],
        "PENDING_APPROVAL" => vec![
            ("APPROVED".to_string(), "Setujui".to_string()),
            ("DRAFT".to_string(), "Kembalikan ke Draft".to_string()),
            ("CANCELLED".to_string(), "Batalkan".to_string())
        ],
        "APPROVED" => vec![
            ("POSTED".to_string(), "Posting ke Buku Besar".to_string()),
            ("CANCELLED".to_string(), "Batalkan".to_string())
        ],
        "POSTED" => vec![], // No transitions allowed
        "CANCELLED" => vec![], // No transitions allowed
        _ => vec![],
    }
}

// Indonesian business helpers
pub fn format_npwp(npwp: &str) -> String {
    // Remove any existing formatting
    let clean = npwp.replace(&['.', '-'][..], "");
    
    if clean.len() == 15 {
        // Format as XX.XXX.XXX.X-XXX.XXX
        format!("{}.{}.{}.{}-{}.{}", 
            &clean[0..2], &clean[2..5], &clean[5..8], 
            &clean[8..9], &clean[9..12], &clean[12..15])
    } else {
        npwp.to_string()
    }
}

pub fn parse_currency_input(input: &str) -> Option<f64> {
    // Remove currency symbols and formatting
    let clean = input
        .replace("Rp", "")
        .replace(".", "")
        .replace(",", ".")
        .trim()
        .to_string();
    
    clean.parse().ok()
}

pub fn format_account_display(code: &str, name: &str) -> String {
    format!("{} - {}", code, name)
}

// Validation helpers for Indonesian business rules
pub fn validate_journal_entry_balance(debit_total: f64, credit_total: f64) -> bool {
    (debit_total - credit_total).abs() < 0.01 && debit_total > 0.0
}

pub fn validate_tax_rate(rate: f64, tax_type: &str) -> bool {
    match tax_type {
        "PPN" => rate >= 0.0 && rate <= 15.0, // PPN max 15%
        "PPH21" => rate >= 0.0 && rate <= 35.0, // PPh21 progressive up to 35%
        "PPH22" => rate >= 0.0 && rate <= 10.0, // PPh22 typically 0.3%-10%
        "PPH23" => rate >= 0.0 && rate <= 15.0, // PPh23 typically 2%-15%
        _ => rate >= 0.0 && rate <= 100.0,
    }
}

pub fn calculate_ppn(base_amount: f64, rate: f64) -> f64 {
    base_amount * rate / 100.0
}

pub fn calculate_pph23(service_amount: f64, rate: f64) -> f64 {
    service_amount * rate / 100.0
}

// Date helpers for Indonesian fiscal year
pub fn get_fiscal_year_start(date: &str) -> String {
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        return format!("{}-01-01", parsed.year());
    }
    chrono::Local::now().format("%Y-01-01").to_string()
}

pub fn get_fiscal_year_end(date: &str) -> String {
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        return format!("{}-12-31", parsed.year());
    }
    chrono::Local::now().format("%Y-12-31").to_string()
}

pub fn is_within_fiscal_year(date: &str, fiscal_year: i32) -> bool {
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        return parsed.year() == fiscal_year;
    }
    false
}

// Indonesian month names
pub fn get_indonesian_month_name(month: u32) -> &'static str {
    match month {
        1 => "Januari",
        2 => "Februari", 
        3 => "Maret",
        4 => "April",
        5 => "Mei",
        6 => "Juni",
        7 => "Juli",
        8 => "Agustus",
        9 => "September",
        10 => "Oktober",
        11 => "November",
        12 => "Desember",
        _ => "Unknown",
    }
}

// Number to words for Indonesian (useful for checks/invoices)
pub fn number_to_words_idr(amount: f64) -> String {
    // This is a simplified version - in production you'd want a complete implementation
    let whole_part = amount as i64;
    if whole_part == 0 {
        return "Nol Rupiah".to_string();
    }
    
    // This would need a complete Indonesian number-to-words implementation
    // For now, return a placeholder
    format!("{} Rupiah", whole_part)
}

// Helper for generating sequential numbers
pub fn generate_next_sequence_number(prefix: &str, current_max: i32, year: i32) -> String {
    format!("{}-{}-{:06}", prefix, year, current_max + 1)
}

// Helper for pagination
pub struct PaginationInfo {
    pub current_page: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub items_per_page: usize,
}

impl PaginationInfo {
    pub fn new(current_page: usize, total_items: usize, items_per_page: usize) -> Self {
        let total_pages = (total_items + items_per_page - 1) / items_per_page;
        Self {
            current_page,
            total_pages,
            total_items,
            items_per_page,
        }
    }
    
    pub fn has_next_page(&self) -> bool {
        self.current_page < self.total_pages
    }
    
    pub fn has_prev_page(&self) -> bool {
        self.current_page > 1
    }
    
    pub fn get_offset(&self) -> usize {
        (self.current_page - 1) * self.items_per_page
    }
}

// Debounce helper for search inputs
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static DEBOUNCE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn debounce<F>(func: F, delay_ms: u64) -> impl Fn() 
where 
    F: Fn() + 'static,
{
    let func = Arc::new(func);
    let debounce_id = DEBOUNCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    move || {
        let func = func.clone();
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(delay_ms as u32).await;
            func();
        });
    }
}

// Toast notification helper (would need implementation)
pub fn show_toast(message: &str, toast_type: &str) {
    // This would integrate with a toast notification system
    // For now, just log to console
    web_sys::console::log_1(&format!("{}: {}", toast_type, message).into());
}

pub fn show_success_toast(message: &str) {
    show_toast(message, "SUCCESS");
}

pub fn show_error_toast(message: &str) {
    show_toast(message, "ERROR");
}

pub fn show_warning_toast(message: &str) {
    show_toast(message, "WARNING");
}

pub fn show_info_toast(message: &str) {
    show_toast(message, "INFO");
}