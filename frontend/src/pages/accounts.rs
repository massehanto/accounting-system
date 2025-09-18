use leptos::*;
use crate::api::accounts;
use crate::types::common::Account;
use crate::components::common::{Button, LoadingSpinner};

#[component]
pub fn AccountsPage() -> impl IntoView {
    let accounts = create_rw_signal(Vec::<Account>::new());
    let loading = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);

    let load_accounts = create_action(|_: &()| async move {
        loading.set(true);
        match accounts::get_accounts().await {
            Ok(data) => {
                accounts.set(data);
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
        load_accounts.dispatch(());
    });

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-2xl font-bold text-gray-900">"Chart of Accounts"</h1>
                <Button>
                    "Add Account"
                </Button>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                    <div class="text-sm text-red-700">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </div>
            </Show>

            <div class="bg-white shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                    <Show 
                        when=move || loading.get()
                        fallback=move || view! {
                            <Show
                                when=move || !accounts.get().is_empty()
                                fallback=|| view! {
                                    <div class="text-center text-gray-500 py-8">
                                        "No accounts found. Create your first account to get started."
                                    </div>
                                }
                            >
                                <AccountsTable accounts=accounts/>
                            </Show>
                        }
                    >
                        <div class="flex justify-center py-8">
                            <LoadingSpinner message=Some("Loading accounts...".to_string())/>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AccountsTable(accounts: RwSignal<Vec<Account>>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Code"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Name"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Type"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {move || accounts.get().into_iter().map(|account| {
                        view! {
                            <tr class="hover:bg-gray-50">
                                <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">{account.account_code}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{account.account_name}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{account.account_type}</td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class={format!("inline-flex px-2 py-1 text-xs font-semibold rounded-full {}", 
                                        if account.is_active { "bg-green-100 text-green-800" } else { "bg-gray-100 text-gray-800" })}>
                                        {if account.is_active { "Active" } else { "Inactive" }}
                                    </span>
                                </td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}