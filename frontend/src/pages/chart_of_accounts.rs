use leptos::*;
use crate::{api, utils};

#[component]
pub fn ChartOfAccountsPage() -> impl IntoView {
    let (accounts, set_accounts) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Load accounts on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match api::get_accounts().await {
                Ok(account_list) => {
                    set_accounts.set(account_list);
                }
                Err(e) => {
                    set_error.set(Some(utils::handle_api_error(&e)));
                }
            }
            set_loading.set(false);
        });
    });

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-2xl font-bold text-gray-900">"Chart of Accounts"</h1>
                <button class="btn-primary">
                    "Add Account"
                </button>
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
                                    <div class="text-center text-gray-500">
                                        "No accounts found. Create your first account to get started."
                                    </div>
                                }
                            >
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="table-header">"Code"</th>
                                                <th class="table-header">"Name"</th>
                                                <th class="table-header">"Type"</th>
                                                <th class="table-header">"Status"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {move || accounts.get().into_iter().map(|account| {
                                                let code = account.get("account_code")
                                                    .and_then(|c| c.as_str())
                                                    .unwrap_or("");
                                                let name = account.get("account_name")
                                                    .and_then(|n| n.as_str())
                                                    .unwrap_or("");
                                                let account_type = account.get("account_type")
                                                    .and_then(|t| t.as_str())
                                                    .unwrap_or("");
                                                let is_active = account.get("is_active")
                                                    .and_then(|a| a.as_bool())
                                                    .unwrap_or(false);
                                                
                                                view! {
                                                    <tr class="hover:bg-gray-50">
                                                        <td class="table-cell font-mono">{code}</td>
                                                        <td class="table-cell">{name}</td>
                                                        <td class="table-cell">{account_type}</td>
                                                        <td class="table-cell">
                                                            <span class={format!("badge {}", 
                                                                if is_active { "badge-success" } else { "badge-secondary" })}>
                                                                {if is_active { "Active" } else { "Inactive" }}
                                                            </span>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            </Show>
                        }
                    >
                        <div class="text-center">
                            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                            <p class="mt-2 text-sm text-gray-600">"Loading accounts..."</p>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}