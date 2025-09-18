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