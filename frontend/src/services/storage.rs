// frontend/src/services/storage.rs
use web_sys::{window, Storage};
use serde::{Serialize, Deserialize};

pub struct LocalStorageService;

impl LocalStorageService {
    pub fn set_item<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let serialized = serde_json::to_string(value)
                    .map_err(|e| format!("Serialization error: {}", e))?;
                storage.set_item(key, &serialized)
                    .map_err(|e| format!("Storage error: {:?}", e))?;
                return Ok(());
            }
        }
        Err("Local storage not available".to_string())
    }

    pub fn get_item<T: for<'de> Deserialize<'de>>(key: &str) -> Result<Option<T>, String> {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(value)) = storage.get_item(key) {
                    let deserialized = serde_json::from_str(&value)
                        .map_err(|e| format!("Deserialization error: {}", e))?;
                    return Ok(Some(deserialized));
                }
            }
        }
        Ok(None)
    }

    pub fn remove_item(key: &str) -> Result<(), String> {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage.remove_item(key)
                    .map_err(|e| format!("Storage error: {:?}", e))?;
                return Ok(());
            }
        }
        Err("Local storage not available".to_string())
    }

    pub fn clear() -> Result<(), String> {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                storage.clear()
                    .map_err(|e| format!("Storage error: {:?}", e))?;
                return Ok(());
            }
        }
        Err("Local storage not available".to_string())
    }
}