use leptos::*;
use crate::api::journal;
use crate::types::journal::{JournalEntry, JournalEntryStatus};
use crate::components::common::{Button, LoadingSpinner, Modal};
use crate::components::forms::Input;
use crate::utils::formatting::format_currency;

#[component]
pub fn JournalEntriesPage() -> impl IntoView {
    let entries = create_rw_signal(Vec::<JournalEntry>::new());
    let loading = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);
    let show_create_modal = create_rw_signal(false);

    let load_entries = create_action(|_: &()| async move {
        loading.set(true);
        match journal::get_journal_entries(None).await {
            Ok(data) => {
                entries.set(data);
                error.set(None);
            }
            Err(e) => {
                error.set(Some(e));
            }
        }
        loading.set(false);
    });

    // Load on mount
    create_effect(move |_| {
        load_entries.dispatch(());
    });

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Journal Entries"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage your general ledger journal entries"</p>
                </div>
                <div class="flex space-x-3">
                    <Button
                        variant=Some(crate::components::common::ButtonVariant::Secondary)
                        on_click=Some(Callback::new(move |_| {
                            load_entries.dispatch(());
                        }))
                    >
                        "Refresh"
                    </Button>
                    <Button
                        on_click=Some(Callback::new(move |_| {
                            show_create_modal.set(true);
                        }))
                    >
                        "New Entry"
                    </Button>
                </div>
            </div>

            // Error Display
            <Show when=move || error.get().is_some()>
                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                    <div class="text-sm text-red-700">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </div>
            </Show>

            // Loading State
            <Show when=move || loading.get()>
                <div class="flex justify-center py-8">
                    <LoadingSpinner message=Some("Loading journal entries...".to_string())/>
                </div>
            </Show>

            // Entries Table
            <Show when=move || !loading.get()>
                <div class="bg-white shadow rounded-lg overflow-hidden">
                    <Show 
                        when=move || !entries.get().is_empty()
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
                                    <Button
                                        on_click=Some(Callback::new(move |_| {
                                            show_create_modal.set(true);
                                        }))
                                    >
                                        "New Entry"
                                    </Button>
                                </div>
                            </div>
                        }
                    >
                        <JournalEntriesTable entries=entries/>
                    </Show>
                </div>
            </Show>

            // Create Modal
            <Modal
                show=show_create_modal.into()
                on_close=Callback::new(move |_| show_create_modal.set(false))
                title=Some("Create Journal Entry".to_string())
                size=Some("xl".to_string())
            >
                <CreateJournalEntryForm 
                    on_success=Callback::new(move |_| {
                        show_create_modal.set(false);
                        load_entries.dispatch(());
                    })
                />
            </Modal>
        </div>
    }
}

#[component]
fn JournalEntriesTable(entries: RwSignal<Vec<JournalEntry>>) -> impl IntoView {
    view! {
        <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
                <tr>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        "Entry Number"
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        "Date"
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        "Description"
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        "Amount"
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        "Status"
                    </th>
                    <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                        "Actions"
                    </th>
                </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
                {move || entries.get().into_iter().map(|entry| {
                    let status_color = match entry.status {
                        JournalEntryStatus::Draft => "bg-gray-100 text-gray-800",
                        JournalEntryStatus::PendingApproval => "bg-yellow-100 text-yellow-800",
                        JournalEntryStatus::Approved => "bg-blue-100 text-blue-800",
                        JournalEntryStatus::Posted => "bg-green-100 text-green-800",
                        JournalEntryStatus::Cancelled => "bg-red-100 text-red-800",
                    };
                    
                    view! {
                        <tr class="hover:bg-gray-50">
                            <td class="px-6 py-4 whitespace-nowrap">
                                <div class="font-medium text-gray-900">
                                    {entry.entry_number.clone()}
                                </div>
                            </td>
                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                {entry.entry_date.clone()}
                            </td>
                            <td class="px-6 py-4 text-sm text-gray-900">
                                <div class="max-w-xs truncate">
                                    {entry.description.unwrap_or_else(|| "—".to_string())}
                                </div>
                            </td>
                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                {format_currency(entry.total_debit)}
                            </td>
                            <td class="px-6 py-4 whitespace-nowrap">
                                <span class=format!("inline-flex px-2 py-1 text-xs font-semibold rounded-full {}", status_color)>
                                    {format!("{}", entry.status)}
                                </span>
                            </td>
                            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                <button class="text-blue-600 hover:text-blue-900 mr-3">
                                    "Edit"
                                </button>
                                <button class="text-red-600 hover:text-red-900">
                                    "Delete"
                                </button>
                            </td>
                        </tr>
                    }
                }).collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}

#[component]
fn CreateJournalEntryForm(on_success: Callback<()>) -> impl IntoView {
    let entry_date = create_rw_signal(chrono::Local::now().format("%Y-%m-%d").to_string());
    let description = create_rw_signal(String::new());
    let reference = create_rw_signal(String::new());
    let loading = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);

    let submit_form = move |_| {
        loading.set(true);
        error.set(None);

        let entry_data = serde_json::json!({
            "entry_date": entry_date.get(),
            "description": description.get(),
            "reference": reference.get(),
            "lines": []
        });

        spawn_local(async move {
            match journal::create_journal_entry(entry_data).await {
                Ok(_) => {
                    on_success.call(());
                }
                Err(e) => {
                    error.set(Some(e));
                }
            }
            loading.set(false);
        });
    };

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            submit_form(());
        }>
            <div class="space-y-4">
                <Input
                    label="Entry Date"
                    input_type="date"
                    value=entry_date
                    required=Some(true)
                />
                
                <Input
                    label="Description"
                    input_type="text"
                    value=description
                    placeholder=Some("Enter transaction description".to_string())
                />
                
                <Input
                    label="Reference"
                    input_type="text"
                    value=reference
                    placeholder=Some("Reference number or document".to_string())
                />
                
                <Show when=move || error.get().is_some()>
                    <div class="text-red-600 text-sm">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>
                
                <div class="flex justify-end space-x-3 pt-4">
                    <Button
                        variant=Some(crate::components::common::ButtonVariant::Secondary)
                        on_click=Some(Callback::new(move |_| {
                            // Close modal logic would go here
                        }))
                    >
                        "Cancel"
                    </Button>
                    <Button
                        loading=Some(loading.get())
                        disabled=Some(loading.get())
                    >
                        "Create Entry"
                    </Button>
                </div>
            </div>
        </form>
    }
}