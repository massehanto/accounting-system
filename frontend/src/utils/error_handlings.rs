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

pub fn show_toast(message: &str, toast_type: &str) {
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