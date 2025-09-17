// frontend/src/pages/journal_entries.rs
use leptos::*;
use crate::{api, utils};
use crate::api::{JournalEntry, JournalEntryStatus, JournalEntryWithLines, CreateJournalEntryRequest, CreateJournalEntryLineRequest};

#[component]
pub fn JournalEntriesPage() -> impl IntoView {
    let (journal_entries, set_journal_entries) = create_signal(Vec::<JournalEntry>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (status_filter, set_status_filter) = create_signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (selected_entry, set_selected_entry) = create_signal(Option::<String>::None);

    // Load journal entries
    let load_entries = create_action(|status: &Option<String>| {
        let status = status.clone();
        async move {
            set_loading.set(true);
            set_error.set(None);
            
            match api::get_journal_entries(status.as_deref()).await {
                Ok(entries) => {
                    set_journal_entries.set(entries);
                }
                Err(e) => {
                    set_error.set(Some(utils::handle_api_error(&e)));
                }
            }
            set_loading.set(false);
        }
    });

    // Load entries on mount
    create_effect(move |_| {
        load_entries.dispatch(status_filter.get());
    });

    let update_status = create_action(|(id, new_status): &(String, String)| {
        let id = id.clone();
        let new_status = new_status.clone();
        async move {
            match api::update_journal_entry_status(&id, &new_status).await {
                Ok(_) => {
                    // Reload entries
                    load_entries.dispatch(status_filter.get());
                }
                Err(e) => {
                    set_error.set(Some(utils::handle_api_error(&e)));
                }
            }
        }
    });

    let delete_entry = create_action(|id: &String| {
        let id = id.clone();
        async move {
            match api::delete_journal_entry(&id).await {
                Ok(_) => {
                    // Reload entries
                    load_entries.dispatch(status_filter.get());
                }
                Err(e) => {
                    set_error.set(Some(utils::handle_api_error(&e)));
                }
            }
        }
    });

    let get_status_badge_class = |status: &JournalEntryStatus| -> &'static str {
        match status {
            JournalEntryStatus::Draft => "bg-gray-100 text-gray-800",
            JournalEntryStatus::PendingApproval => "bg-yellow-100 text-yellow-800",
            JournalEntryStatus::Approved => "bg-blue-100 text-blue-800",
            JournalEntryStatus::Posted => "bg-green-100 text-green-800",
            JournalEntryStatus::Cancelled => "bg-red-100 text-red-800",
        }
    };

    let can_transition_to = |current: &JournalEntryStatus| -> Vec<(&'static str, &'static str)> {
        match current {
            JournalEntryStatus::Draft => vec![
                ("PENDING_APPROVAL", "Submit for Approval"),
                ("CANCELLED", "Cancel")
            ],
            JournalEntryStatus::PendingApproval => vec![
                ("APPROVED", "Approve"),
                ("DRAFT", "Reject to Draft"),
                ("CANCELLED", "Cancel")
            ],
            JournalEntryStatus::Approved => vec![
                ("POSTED", "Post Entry"),
                ("CANCELLED", "Cancel")
            ],
            JournalEntryStatus::Posted => vec![], // No transitions allowed
            JournalEntryStatus::Cancelled => vec![], // No transitions allowed
        }
    };

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Journal Entries"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage your general ledger journal entries"</p>
                </div>
                <div class="flex space-x-3">
                    <button 
                        class="btn-secondary"
                        on:click=move |_| {
                            load_entries.dispatch(status_filter.get());
                        }
                    >
                        "Refresh"
                    </button>
                    <button 
                        class="btn-primary"
                        on:click=move |_| set_show_create_modal.set(true)
                    >
                        "New Entry"
                    </button>
                </div>
            </div>

            // Filters
            <div class="bg-white p-4 rounded-lg shadow">
                <div class="flex items-center space-x-4">
                    <label class="text-sm font-medium text-gray-700">"Filter by Status:"</label>
                    <select 
                        class="form-select"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            let filter = if value.is_empty() { None } else { Some(value) };
                            set_status_filter.set(filter);
                        }
                    >
                        <option value="">"All Statuses"</option>
                        <option value="DRAFT">"Draft"</option>
                        <option value="PENDING_APPROVAL">"Pending Approval"</option>
                        <option value="APPROVED">"Approved"</option>
                        <option value="POSTED">"Posted"</option>
                        <option value="CANCELLED">"Cancelled"</option>
                    </select>
                </div>
            </div>

            // Error Display
            <Show when=move || error.get().is_some()>
                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                    <div class="flex">
                        <div class="text-sm text-red-700">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </div>
                </div>
            </Show>

            // Loading State
            <Show when=move || loading.get()>
                <div class="text-center py-8">
                    <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                    <p class="mt-2 text-sm text-gray-600">"Loading journal entries..."</p>
                </div>
            </Show>

            // Journal Entries Table
            <Show when=move || !loading.get()>
                <div class="bg-white shadow rounded-lg overflow-hidden">
                    <Show 
                        when=move || !journal_entries.get().is_empty()
                        fallback=|| view! {
                            <div class="p-8 text-center">
                                <div class="text-gray-500">
                                    <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                    </svg>
                                    <h3 class="mt-2 text-sm font-medium text-gray-900">"No journal entries"</h3>
                                    <p class="mt-1 text-sm text-gray-500">"Get started by creating a new journal entry."</p>
                                </div>
                                <div class="mt-6">
                                    <button 
                                        class="btn-primary"
                                        on:click=move |_| set_show_create_modal.set(true)
                                    >
                                        "New Entry"
                                    </button>
                                </div>
                            </div>
                        }
                    >
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th class="table-header">"Entry Number"</th>
                                    <th class="table-header">"Date"</th>
                                    <th class="table-header">"Description"</th>
                                    <th class="table-header">"Reference"</th>
                                    <th class="table-header">"Total Amount"</th>
                                    <th class="table-header">"Status"</th>
                                    <th class="table-header">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                {move || journal_entries.get().into_iter().map(|entry| {
                                    let entry_id = entry.id.clone();
                                    let status_transitions = can_transition_to(&entry.status);
                                    
                                    view! {
                                        <tr class="hover:bg-gray-50">
                                            <td class="table-cell">
                                                <div class="font-medium text-gray-900">
                                                    {entry.entry_number.clone()}
                                                </div>
                                                <div class="text-sm text-gray-500">
                                                    {entry.created_at.clone()}
                                                </div>
                                            </td>
                                            <td class="table-cell">
                                                {utils::format_date(&entry.entry_date)}
                                            </td>
                                            <td class="table-cell">
                                                <div class="max-w-xs truncate">
                                                    {entry.description.unwrap_or_else(|| "—".to_string())}
                                                </div>
                                            </td>
                                            <td class="table-cell">
                                                {entry.reference.unwrap_or_else(|| "—".to_string())}
                                            </td>
                                            <td class="table-cell">
                                                <div class="text-right">
                                                    {utils::format_currency(entry.total_debit)}
                                                </div>
                                            </td>
                                            <td class="table-cell">
                                                <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", 
                                                    get_status_badge_class(&entry.status))}>
                                                    {format!("{}", entry.status)}
                                                </span>
                                            </td>
                                            <td class="table-cell">
                                                <div class="flex items-center space-x-2">
                                                    // View button
                                                    <button 
                                                        class="text-indigo-600 hover:text-indigo-900 text-sm"
                                                        on:click={
                                                            let entry_id = entry_id.clone();
                                                            move |_| set_selected_entry.set(Some(entry_id.clone()))
                                                        }
                                                    >
                                                        "View"
                                                    </button>

                                                    // Status transition buttons
                                                    <Show when=move || !status_transitions.is_empty()>
                                                        <div class="relative inline-block text-left">
                                                            <select 
                                                                class="text-sm border border-gray-300 rounded px-2 py-1"
                                                                on:change={
                                                                    let entry_id = entry_id.clone();
                                                                    move |ev| {
                                                                        let new_status = event_target_value(&ev);
                                                                        if !new_status.is_empty() {
                                                                            update_status.dispatch((entry_id.clone(), new_status));
                                                                        }
                                                                    }
                                                                }
                                                            >
                                                                <option value="">"Change Status"</option>
                                                                {status_transitions.into_iter().map(|(status_code, label)| {
                                                                    view! {
                                                                        <option value=status_code>{label}</option>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </select>
                                                        </div>
                                                    </Show>

                                                    // Delete button (only for draft entries)
                                                    <Show when=move || matches!(entry.status, JournalEntryStatus::Draft)>
                                                        <button 
                                                            class="text-red-600 hover:text-red-900 text-sm"
                                                            on:click={
                                                                let entry_id = entry_id.clone();
                                                                let entry_number = entry.entry_number.clone();
                                                                move |_| {
                                                                    if window().confirm_with_message(&format!("Are you sure you want to delete journal entry {}?", entry_number)).unwrap_or(false) {
                                                                        delete_entry.dispatch(entry_id.clone());
                                                                    }
                                                                }
                                                            }
                                                        >
                                                            "Delete"
                                                        </button>
                                                    </Show>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </Show>
                </div>
            </Show>

            // Create Modal
            <Show when=move || show_create_modal.get()>
                <CreateJournalEntryModal 
                    show=show_create_modal
                    on_close=move |_| set_show_create_modal.set(false)
                    on_created=move |_| {
                        set_show_create_modal.set(false);
                        load_entries.dispatch(status_filter.get());
                    }
                />
            </Show>

            // View Modal
            <Show when=move || selected_entry.get().is_some()>
                <ViewJournalEntryModal 
                    entry_id=selected_entry
                    on_close=move |_| set_selected_entry.set(None)
                />
            </Show>
        </div>
    }
}

#[component]
fn CreateJournalEntryModal<F1, F2>(
    show: ReadSignal<bool>,
    on_close: F1,
    on_created: F2,
) -> impl IntoView 
where
    F1: Fn(()) + 'static + Copy,
    F2: Fn(()) + 'static + Copy,
{
    let (entry_date, set_entry_date) = create_signal(chrono::Local::now().format("%Y-%m-%d").to_string());
    let (description, set_description) = create_signal(String::new());
    let (reference, set_reference) = create_signal(String::new());
    let (lines, set_lines) = create_signal(vec![
        CreateJournalEntryLineRequest {
            account_id: String::new(),
            description: None,
            debit_amount: 0.0,
            credit_amount: 0.0,
        },
        CreateJournalEntryLineRequest {
            account_id: String::new(),
            description: None,
            debit_amount: 0.0,
            credit_amount: 0.0,
        }
    ]);
    let (accounts, set_accounts) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Load accounts on mount
    create_effect(move |_| {
        if show.get() {
            spawn_local(async move {
                match api::get_accounts().await {
                    Ok(accounts_list) => set_accounts.set(accounts_list),
                    Err(e) => set_error.set(Some(format!("Failed to load accounts: {}", e))),
                }
            });
        }
    });

    let add_line = move |_| {
        let mut current_lines = lines.get();
        current_lines.push(CreateJournalEntryLineRequest {
            account_id: String::new(),
            description: None,
            debit_amount: 0.0,
            credit_amount: 0.0,
        });
        set_lines.set(current_lines);
    };

    let remove_line = move |index: usize| {
        let mut current_lines = lines.get();
        if current_lines.len() > 2 { // Keep at least 2 lines
            current_lines.remove(index);
            set_lines.set(current_lines);
        }
    };

    let update_line = move |index: usize, field: String, value: String| {
        let mut current_lines = lines.get();
        if let Some(line) = current_lines.get_mut(index) {
            match field.as_str() {
                "account_id" => line.account_id = value,
                "description" => line.description = if value.is_empty() { None } else { Some(value) },
                "debit_amount" => line.debit_amount = value.parse().unwrap_or(0.0),
                "credit_amount" => line.credit_amount = value.parse().unwrap_or(0.0),
                _ => {}
            }
        }
        set_lines.set(current_lines);
    };

    let calculate_totals = move || {
        let current_lines = lines.get();
        let total_debits: f64 = current_lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: f64 = current_lines.iter().map(|l| l.credit_amount).sum();
        (total_debits, total_credits)
    };

    let is_balanced = move || {
        let (debits, credits) = calculate_totals();
        (debits - credits).abs() < 0.01 && debits > 0.0 && credits > 0.0
    };

    let submit_entry = move |_| {
        set_loading.set(true);
        set_error.set(None);

        let company_id = utils::get_company_id().unwrap_or_default();
        let entry = CreateJournalEntryRequest {
            company_id,
            entry_date: entry_date.get(),
            description: if description.get().is_empty() { None } else { Some(description.get()) },
            reference: if reference.get().is_empty() { None } else { Some(reference.get()) },
            lines: lines.get(),
        };

        spawn_local(async move {
            match api::create_journal_entry(entry).await {
                Ok(_) => {
                    on_created(());
                }
                Err(e) => {
                    set_error.set(Some(utils::handle_api_error(&e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
                <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-4xl shadow-lg rounded-md bg-white">
                    <div class="mt-3">
                        // Header
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-lg font-medium text-gray-900">"Create Journal Entry"</h3>
                            <button 
                                class="text-gray-400 hover:text-gray-600"
                                on:click=move |_| on_close(())
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>

                        // Error Display
                        <Show when=move || error.get().is_some()>
                            <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
                                <div class="text-sm text-red-700">
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </div>
                        </Show>

                        // Form
                        <form on:submit=move |ev| {
                            ev.prevent_default();
                            if is_balanced() {
                                submit_entry(());
                            }
                        }>
                            // Basic Information
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Entry Date" <span class="text-red-500">"*"</span>
                                    </label>
                                    <input
                                        type="date"
                                        class="form-input"
                                        required
                                        prop:value=move || entry_date.get()
                                        on:input=move |ev| set_entry_date.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Reference"
                                    </label>
                                    <input
                                        type="text"
                                        class="form-input"
                                        placeholder="Reference number or document"
                                        prop:value=move || reference.get()
                                        on:input=move |ev| set_reference.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <div class="mb-6">
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Description"
                                </label>
                                <textarea
                                    class="form-input"
                                    rows="3"
                                    placeholder="Enter transaction description"
                                    prop:value=move || description.get()
                                    on:input=move |ev| set_description.set(event_target_value(&ev))
                                ></textarea>
                            </div>

                            // Journal Entry Lines
                            <div class="mb-6">
                                <div class="flex justify-between items-center mb-4">
                                    <h4 class="text-md font-medium text-gray-900">"Journal Entry Lines"</h4>
                                    <button 
                                        type="button"
                                        class="btn-secondary text-sm"
                                        on:click=add_line
                                    >
                                        "Add Line"
                                    </button>
                                </div>

                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="table-header text-left">"Account"</th>
                                                <th class="table-header text-left">"Description"</th>
                                                <th class="table-header text-right">"Debit"</th>
                                                <th class="table-header text-right">"Credit"</th>
                                                <th class="table-header">"Action"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {move || lines.get().into_iter().enumerate().map(|(index, line)| {
                                                view! {
                                                    <tr>
                                                        <td class="table-cell">
                                                            <select 
                                                                class="form-select w-full"
                                                                required
                                                                prop:value=move || line.account_id.clone()
                                                                on:change={
                                                                    move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        update_line(index, "account_id".to_string(), value);
                                                                    }
                                                                }
                                                            >
                                                                <option value="">"Select Account"</option>
                                                                {move || accounts.get().into_iter().map(|account| {
                                                                    let account_id = account["id"].as_str().unwrap_or("");
                                                                    let account_code = account["account_code"].as_str().unwrap_or("");
                                                                    let account_name = account["account_name"].as_str().unwrap_or("");
                                                                    let display = format!("{} - {}", account_code, account_name);
                                                                    
                                                                    view! {
                                                                        <option value=account_id>{display}</option>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </select>
                                                        </td>
                                                        <td class="table-cell">
                                                            <input
                                                                type="text"
                                                                class="form-input w-full"
                                                                placeholder="Line description"
                                                                prop:value=move || line.description.clone().unwrap_or_default()
                                                                on:input={
                                                                    move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        update_line(index, "description".to_string(), value);
                                                                    }
                                                                }
                                                            />
                                                        </td>
                                                        <td class="table-cell">
                                                            <input
                                                                type="number"
                                                                step="0.01"
                                                                min="0"
                                                                class="form-input w-full text-right"
                                                                placeholder="0.00"
                                                                prop:value=move || {
                                                                    if line.debit_amount > 0.0 {
                                                                        line.debit_amount.to_string()
                                                                    } else {
                                                                        String::new()
                                                                    }
                                                                }
                                                                on:input={
                                                                    move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        update_line(index, "debit_amount".to_string(), value);
                                                                        // Clear credit when debit is entered
                                                                        if !value.is_empty() && value.parse::<f64>().unwrap_or(0.0) > 0.0 {
                                                                            update_line(index, "credit_amount".to_string(), "0".to_string());
                                                                        }
                                                                    }
                                                                }
                                                            />
                                                        </td>
                                                        <td class="table-cell">
                                                            <input
                                                                type="number"
                                                                step="0.01"
                                                                min="0"
                                                                class="form-input w-full text-right"
                                                                placeholder="0.00"
                                                                prop:value=move || {
                                                                    if line.credit_amount > 0.0 {
                                                                        line.credit_amount.to_string()
                                                                    } else {
                                                                        String::new()
                                                                    }
                                                                }
                                                                on:input={
                                                                    move |ev| {
                                                                        let value = event_target_value(&ev);
                                                                        update_line(index, "credit_amount".to_string(), value);
                                                                        // Clear debit when credit is entered
                                                                        if !value.is_empty() && value.parse::<f64>().unwrap_or(0.0) > 0.0 {
                                                                            update_line(index, "debit_amount".to_string(), "0".to_string());
                                                                        }
                                                                    }
                                                                }
                                                            />
                                                        </td>
                                                        <td class="table-cell text-center">
                                                            <Show when=move || lines.get().len() > 2>
                                                                <button 
                                                                    type="button"
                                                                    class="text-red-600 hover:text-red-800"
                                                                    on:click=move |_| remove_line(index)
                                                                >
                                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                                                                    </svg>
                                                                </button>
                                                            </Show>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                        <tfoot class="bg-gray-50">
                                            <tr>
                                                <td colspan="2" class="table-cell font-medium">"Totals:"</td>
                                                <td class="table-cell text-right font-medium">
                                                    {move || utils::format_currency(calculate_totals().0)}
                                                </td>
                                                <td class="table-cell text-right font-medium">
                                                    {move || utils::format_currency(calculate_totals().1)}
                                                </td>
                                                <td class="table-cell">
                                                    <Show 
                                                        when=move || is_balanced()
                                                        fallback=|| view! {
                                                            <span class="text-red-600 text-sm">"Not Balanced"</span>
                                                        }
                                                    >
                                                        <span class="text-green-600 text-sm">"Balanced ✓"</span>
                                                    </Show>
                                                </td>
                                            </tr>
                                        </tfoot>
                                    </table>
                                </div>
                            </div>

                            // Form Actions
                            <div class="flex justify-end space-x-3">
                                <button 
                                    type="button"
                                    class="btn-secondary"
                                    on:click=move |_| on_close(())
                                    disabled=loading
                                >
                                    "Cancel"
                                </button>
                                <button 
                                    type="submit"
                                    class="btn-primary"
                                    disabled=move || loading.get() || !is_balanced()
                                >
                                    <Show 
                                        when=move || loading.get()
                                        fallback=|| "Create Entry"
                                    >
                                        "Creating..."
                                    </Show>
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ViewJournalEntryModal<F>(
    entry_id: ReadSignal<Option<String>>,
    on_close: F,
) -> impl IntoView 
where
    F: Fn(()) + 'static + Copy,
{
    let (entry_data, set_entry_data) = create_signal(Option::<JournalEntryWithLines>::None);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Load entry data when ID changes
    create_effect(move |_| {
        if let Some(id) = entry_id.get() {
            set_loading.set(true);
            set_error.set(None);
            
            spawn_local(async move {
                match api::get_journal_entry_with_lines(&id).await {
                    Ok(data) => set_entry_data.set(Some(data)),
                    Err(e) => set_error.set(Some(utils::handle_api_error(&e))),
                }
                set_loading.set(false);
            });
        }
    });

    view! {
        <Show when=move || entry_id.get().is_some()>
            <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
                <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-4xl shadow-lg rounded-md bg-white">
                    <div class="mt-3">
                        // Header
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-lg font-medium text-gray-900">"Journal Entry Details"</h3>
                            <button 
                                class="text-gray-400 hover:text-gray-600"
                                on:click=move |_| on_close(())
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>

                        <Show when=move || loading.get()>
                            <div class="text-center py-8">
                                <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                                <p class="mt-2 text-sm text-gray-600">"Loading entry details..."</p>
                            </div>
                        </Show>

                        <Show when=move || error.get().is_some()>
                            <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
                                <div class="text-sm text-red-700">
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </div>
                        </Show>

                        <Show when=move || entry_data.get().is_some() && !loading.get()>
                            {move || {
                                if let Some(data) = entry_data.get() {
                                    let entry = &data.journal_entry;
                                    let lines = &data.lines;
                                    
                                    view! {
                                        <div class="space-y-6">
                                            // Entry Information
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">"Entry Number"</label>
                                                    <p class="mt-1 text-sm text-gray-900">{entry.entry_number.clone()}</p>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">"Date"</label>
                                                    <p class="mt-1 text-sm text-gray-900">{utils::format_date(&entry.entry_date)}</p>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">"Status"</label>
                                                    <span class={format!("mt-1 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", 
                                                        match entry.status {
                                                            JournalEntryStatus::Draft => "bg-gray-100 text-gray-800",
                                                            JournalEntryStatus::PendingApproval => "bg-yellow-100 text-yellow-800",
                                                            JournalEntryStatus::Approved => "bg-blue-100 text-blue-800",
                                                            JournalEntryStatus::Posted => "bg-green-100 text-green-800",
                                                            JournalEntryStatus::Cancelled => "bg-red-100 text-red-800",
                                                        })}>
                                                        {format!("{}", entry.status)}
                                                    </span>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">"Reference"</label>
                                                    <p class="mt-1 text-sm text-gray-900">
                                                        {entry.reference.clone().unwrap_or_else(|| "—".to_string())}
                                                    </p>
                                                </div>
                                            </div>

                                            <div>
                                                <label class="block text-sm font-medium text-gray-700">"Description"</label>
                                                <p class="mt-1 text-sm text-gray-900">
                                                    {entry.description.clone().unwrap_or_else(|| "—".to_string())}
                                                </p>
                                            </div>

                                            // Entry Lines
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-3">"Journal Entry Lines"</label>
                                                <div class="overflow-x-auto">
                                                    <table class="min-w-full divide-y divide-gray-200">
                                                        <thead class="bg-gray-50">
                                                            <tr>
                                                                <th class="table-header text-left">"Account"</th>
                                                                <th class="table-header text-left">"Description"</th>
                                                                <th class="table-header text-right">"Debit"</th>
                                                                <th class="table-header text-right">"Credit"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody class="bg-white divide-y divide-gray-200">
                                                            {lines.iter().map(|line| {
                                                                view! {
                                                                    <tr>
                                                                        <td class="table-cell">
                                                                            <div>
                                                                                <div class="font-medium text-gray-900">
                                                                                    {line.account_code.clone().unwrap_or_default()}
                                                                                </div>
                                                                                <div class="text-sm text-gray-500">
                                                                                    {line.account_name.clone().unwrap_or_default()}
                                                                                </div>
                                                                            </div>
                                                                        </td>
                                                                        <td class="table-cell">
                                                                            {line.description.clone().unwrap_or_else(|| "—".to_string())}
                                                                        </td>
                                                                        <td class="table-cell text-right">
                                                                            {if line.debit_amount > 0.0 {
                                                                                utils::format_currency(line.debit_amount)
                                                                            } else {
                                                                                "—".to_string()
                                                                            }}
                                                                        </td>
                                                                        <td class="table-cell text-right">
                                                                            {if line.credit_amount > 0.0 {
                                                                                utils::format_currency(line.credit_amount)
                                                                            } else {
                                                                                "—".to_string()
                                                                            }}
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </tbody>
                                                        <tfoot class="bg-gray-50">
                                                            <tr>
                                                                <td colspan="2" class="table-cell font-medium">"Totals:"</td>
                                                                <td class="table-cell text-right font-medium">
                                                                    {utils::format_currency(entry.total_debit)}
                                                                </td>
                                                                <td class="table-cell text-right font-medium">
                                                                    {utils::format_currency(entry.total_credit)}
                                                                </td>
                                                            </tr>
                                                        </tfoot>
                                                    </table>
                                                </div>
                                            </div>

                                            // Audit Information
                                            <div class="border-t pt-4">
                                                <h4 class="text-sm font-medium text-gray-700 mb-3">"Audit Information"</h4>
                                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                                                    <div>
                                                        <span class="text-gray-500">"Created: "</span>
                                                        <span class="text-gray-900">{utils::format_date(&entry.created_at)}</span>
                                                    </div>
                                                    <Show when=move || entry.approved_at.is_some()>
                                                        <div>
                                                            <span class="text-gray-500">"Approved: "</span>
                                                            <span class="text-gray-900">
                                                                {entry.approved_at.clone().map(|d| utils::format_date(&d)).unwrap_or_default()}
                                                            </span>
                                                        </div>
                                                    </Show>
                                                    <Show when=move || entry.posted_at.is_some()>
                                                        <div>
                                                            <span class="text-gray-500">"Posted: "</span>
                                                            <span class="text-gray-900">
                                                                {entry.posted_at.clone().map(|d| utils::format_date(&d)).unwrap_or_default()}
                                                            </span>
                                                        </div>
                                                    </Show>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    view! { <div></div> }
                                }
                            }}
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// Helper function to access window object
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}