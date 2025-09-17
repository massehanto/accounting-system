// frontend/src/services/offline.rs
use leptos::*;
use serde::{Serialize, Deserialize};
use crate::services::storage::LocalStorageService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineAction {
    pub id: String,
    pub action_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
    pub retry_count: u32,
}

pub struct OfflineManager;

impl OfflineManager {
    const OFFLINE_ACTIONS_KEY: &'static str = "offline_actions";
    const MAX_RETRIES: u32 = 3;

    pub fn queue_action(action: OfflineAction) -> Result<(), String> {
        let mut actions = Self::get_queued_actions()?;
        actions.push(action);
        LocalStorageService::set_item(Self::OFFLINE_ACTIONS_KEY, &actions)
    }

    pub fn get_queued_actions() -> Result<Vec<OfflineAction>, String> {
        LocalStorageService::get_item(Self::OFFLINE_ACTIONS_KEY)
            .map(|actions| actions.unwrap_or_default())
    }

    pub fn remove_action(action_id: &str) -> Result<(), String> {
        let mut actions = Self::get_queued_actions()?;
        actions.retain(|a| a.id != action_id);
        LocalStorageService::set_item(Self::OFFLINE_ACTIONS_KEY, &actions)
    }

    pub fn clear_all_actions() -> Result<(), String> {
        LocalStorageService::remove_item(Self::OFFLINE_ACTIONS_KEY)
    }

    pub async fn sync_offline_actions() -> Result<(), String> {
        let actions = Self::get_queued_actions()?;
        
        for action in actions {
            match Self::execute_action(&action).await {
                Ok(_) => {
                    // Remove successful action
                    Self::remove_action(&action.id)?;
                }
                Err(_) => {
                    // Increment retry count
                    if action.retry_count < Self::MAX_RETRIES {
                        let mut updated_action = action.clone();
                        updated_action.retry_count += 1;
                        
                        // Remove old and add updated
                        Self::remove_action(&action.id)?;
                        Self::queue_action(updated_action)?;
                    } else {
                        // Max retries reached, remove action
                        Self::remove_action(&action.id)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn execute_action(action: &OfflineAction) -> Result<(), String> {
        // This would contain the actual API call logic
        match action.action_type.as_str() {
            "create_journal_entry" => {
                // Execute the journal entry creation
                Ok(())
            }
            "update_journal_entry" => {
                // Execute the journal entry update
                Ok(())
            }
            _ => Err("Unknown action type".to_string()),
        }
    }
}

// React hook for offline functionality
#[derive(Clone)]
pub struct OfflineHook {
    pub is_online: ReadSignal<bool>,
    pub pending_actions_count: ReadSignal<usize>,
    pub sync_actions: Box<dyn Fn()>,
}

pub fn use_offline() -> OfflineHook {
    let (is_online, set_is_online) = create_signal(true);
    let (pending_count, set_pending_count) = create_signal(0);

    // Monitor online status
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            set_is_online.set(navigator.on_line());
        }
    });

    // Update pending actions count
    let update_pending_count = move || {
        if let Ok(actions) = OfflineManager::get_queued_actions() {
            set_pending_count.set(actions.len());
        }
    };

    let sync_actions = Box::new(move || {
        spawn_local(async move {
            if let Ok(_) = OfflineManager::sync_offline_actions().await {
                update_pending_count();
            }
        });
    });

    OfflineHook {
        is_online,
        pending_actions_count: pending_count,
        sync_actions,
    }
}