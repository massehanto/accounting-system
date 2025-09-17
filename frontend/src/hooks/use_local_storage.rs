// frontend/src/hooks/use_local_storage.rs
use leptos::*;
use serde::{Serialize, Deserialize};
use crate::services::storage::LocalStorageService;

pub fn use_local_storage<T>(key: &'static str, initial_value: T) -> (ReadSignal<T>, WriteSignal<T>) 
where 
    T: Clone + Serialize + for<'de> Deserialize<'de> + 'static,
{
    let stored_value = LocalStorageService::get_item(key)
        .unwrap_or(None)
        .unwrap_or_else(|| initial_value.clone());
    
    let (value, set_value) = create_signal(stored_value);
    
    let setter = create_memo(move |_| {
        let current_value = value.get();
        let _ = LocalStorageService::set_item(key, &current_value);
    });
    
    // Run the memo to ensure storage is updated
    create_effect(move |_| {
        setter.get();
    });
    
    (value.into(), set_value.into())
}