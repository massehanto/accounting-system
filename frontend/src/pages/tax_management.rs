// frontend/src/pages/tax_management.rs
use leptos::*;
use crate::{api, utils};

#[component]
pub fn TaxManagementPage() -> impl IntoView {
    let (tax_configurations, set_tax_configurations) = create_signal(Vec::<serde_json::Value>::new());
    let (tax_transactions, set_tax_transactions) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (active_tab, set_active_tab) = create_signal("configurations".to_string());
    let (show_config_modal, set_show_config_modal) = create_signal(false);
    let (show_transaction_modal, set_show_transaction_modal) = create_signal(false);
    let (show_calculator_modal, set_show_calculator_modal) = create_signal(false);

    // Load tax configurations on mount
    create_effect(move |_| {
        if active_tab.get() == "configurations" {
            spawn_local(async move {
                set_loading.set(true);
                match api::get_tax_configurations().await {
                    Ok(configs) => {
                        set_tax_configurations.set(configs);
                    }
                    Err(e) => {
                        set_error.set(Some(utils::handle_api_error(&e)));
                    }
                }
                set_loading.set(false);
            });
        }
    });

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Indonesian Tax Management"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage PPN, PPh21, PPh22, PPh23, and other Indonesian tax obligations"</p>
                </div>
                <div class="flex space-x-3">
                    <button 
                        class="btn-secondary"
                        on:click=move |_| {
                            spawn_local(async move {
                                set_loading.set(true);
                                match api::get_tax_configurations().await {
                                    Ok(configs) => set_tax_configurations.set(configs),
                                    Err(e) => set_error.set(Some(utils::handle_api_error(&e))),
                                }
                                set_loading.set(false);
                            });
                        }
                    >
                        "Refresh"
                    </button>
                    <button 
                        class="btn-secondary"
                        on:click=move |_| set_show_calculator_modal.set(true)
                    >
                        "🧮 Tax Calculator"
                    </button>
                    <Show when=move || active_tab.get() == "configurations">
                        <button 
                            class="btn-primary"
                            on:click=move |_| set_show_config_modal.set(true)
                        >
                            "Add Tax Configuration"
                        </button>
                    </Show>
                    <Show when=move || active_tab.get() == "transactions">
                        <button 
                            class="btn-primary"
                            on:click=move |_| set_show_transaction_modal.set(true)
                        >
                            "Record Tax Transaction"
                        </button>
                    </Show>
                </div>
            </div>

            // Tab Navigation
            <div class="border-b border-gray-200">
                <nav class="-mb-px flex space-x-8">
                    <button
                        class={move || if active_tab.get() == "configurations" {
                            "border-primary-500 text-primary-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        } else {
                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        }}
                        on:click=move |_| set_active_tab.set("configurations".to_string())
                    >
                        "⚙️ Tax Configurations"
                    </button>
                    <button
                        class={move || if active_tab.get() == "transactions" {
                            "border-primary-500 text-primary-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        } else {
                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        }}
                        on:click=move |_| set_active_tab.set("transactions".to_string())
                    >
                        "📋 Tax Transactions"
                    </button>
                    <button
                        class={move || if active_tab.get() == "reports" {
                            "border-primary-500 text-primary-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        } else {
                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        }}
                        on:click=move |_| set_active_tab.set("reports".to_string())
                    >
                        "📊 Tax Reports"
                    </button>
                </nav>
            </div>

            // Indonesian Tax Information Panel
            <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <div class="flex items-start">
                    <div class="flex-shrink-0">
                        <span class="text-2xl">🇮🇩</span>
                    </div>
                    <div class="ml-3">
                        <h3 class="text-sm font-medium text-blue-800">
                            "Indonesian Tax Rates (2024)"
                        </h3>
                        <div class="mt-2 text-sm text-blue-700 grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div>
                                <strong>"PPN:"</strong> " 11% (Value Added Tax)"
                            </div>
                            <div>
                                <strong>"PPh 21:"</strong> " Progressive rates (Employee income tax)"
                            </div>
                            <div>
                                <strong>"PPh 22:"</strong> " 1.5% (Import/purchase withholding)"
                            </div>
                            <div>
                                <strong>"PPh 23:"</strong> " 2% services, 10% rent (Service withholding)"
                            </div>
                            <div>
                                <strong>"PTKP Single:"</strong> " Rp 54,000,000/year"
                            </div>
                            <div>
                                <strong>"PTKP Married:"</strong> " Rp 58,500,000/year"
                            </div>
                        </div>
                    </div>
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

            // Tab Content
            <div class="bg-white shadow rounded-lg">
                // Configurations Tab
                <Show when=move || active_tab.get() == "configurations">
                    <div class="px-4 py-5 sm:p-6">
                        <Show 
                            when=move || loading.get()
                            fallback=move || view! {
                                <Show
                                    when=move || !tax_configurations.get().is_empty()
                                    fallback=|| view! {
                                        <div class="text-center py-8">
                                            <div class="text-gray-500">
                                                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                                                </svg>
                                                <h3 class="mt-2 text-sm font-medium text-gray-900">"No tax configurations"</h3>
                                                <p class="mt-1 text-sm text-gray-500">"Set up tax rates for Indonesian compliance."</p>
                                            </div>
                                            <div class="mt-6">
                                                <button 
                                                    class="btn-primary"
                                                    on:click=move |_| set_show_config_modal.set(true)
                                                >
                                                    "Add Configuration"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                >
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="table-header">"Tax Type"</th>
                                                <th class="table-header">"Tax Rate"</th>
                                                <th class="table-header">"Effective Date"</th>
                                                <th class="table-header">"End Date"</th>
                                                <th class="table-header">"Status"</th>
                                                <th class="table-header">"Description"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {move || tax_configurations.get().into_iter().map(|config| {
                                                let tax_type = config.get("tax_type")
                                                    .and_then(|t| t.as_str())
                                                    .unwrap_or("");
                                                let tax_rate = config.get("tax_rate")
                                                    .and_then(|r| r.as_f64())
                                                    .unwrap_or(0.0);
                                                let effective_date = config.get("effective_date")
                                                    .and_then(|d| d.as_str())
                                                    .unwrap_or("");
                                                let end_date = config.get("end_date")
                                                    .and_then(|d| d.as_str())
                                                    .unwrap_or("—");
                                                let is_active = config.get("is_active")
                                                    .and_then(|a| a.as_bool())
                                                    .unwrap_or(false);
                                                let description = config.get("description")
                                                    .and_then(|d| d.as_str())
                                                    .unwrap_or("—");
                                                
                                                view! {
                                                    <tr class="hover:bg-gray-50">
                                                        <td class="table-cell">
                                                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-primary-100 text-primary-800">
                                                                {tax_type}
                                                            </span>
                                                        </td>
                                                        <td class="table-cell font-mono">{format!("{:.2}%", tax_rate)}</td>
                                                        <td class="table-cell">{utils::format_date(effective_date)}</td>
                                                        <td class="table-cell">{if end_date == "—" { end_date.to_string() } else { utils::format_date(end_date) }}</td>
                                                        <td class="table-cell">
                                                            <span class={format!("badge {}", 
                                                                if is_active { "badge-success" } else { "badge-secondary" })}>
                                                                {if is_active { "Active" } else { "Inactive" }}
                                                            </span>
                                                        </td>
                                                        <td class="table-cell">{description}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </Show>
                            }
                        >
                            <div class="text-center py-8">
                                <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
                                <p class="mt-2 text-sm text-gray-600">"Loading tax configurations..."</p>
                            </div>
                        </Show>
                    </div>
                </Show>

                // Transactions Tab
                <Show when=move || active_tab.get() == "transactions">
                    <div class="px-4 py-5 sm:p-6">
                        <div class="text-center py-8">
                            <div class="text-gray-500">
                                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                </svg>
                                <h3 class="mt-2 text-sm font-medium text-gray-900">"Tax Transactions"</h3>
                                <p class="mt-1 text-sm text-gray-500">"Record and track tax withholdings and payments."</p>
                            </div>
                        </div>
                    </div>
                </Show>

                // Reports Tab
                <Show when=move || active_tab.get() == "reports">
                    <div class="px-4 py-5 sm:p-6">
                        <div class="space-y-6">
                            <h3 class="text-lg font-medium text-gray-900">"Indonesian Tax Reports"</h3>
                            
                            // Quick Report Access
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                                <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <span class="text-2xl">"🧾"</span>
                                        </div>
                                        <div class="ml-4">
                                            <h4 class="text-sm font-medium text-gray-900">"SPT Masa PPN"</h4>
                                            <p class="text-sm text-gray-500">"Monthly VAT return"</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <span class="text-2xl">"💼"</span>
                                        </div>
                                        <div class="ml-4">
                                            <h4 class="text-sm font-medium text-gray-900">"PPh 21 Report"</h4>
                                            <p class="text-sm text-gray-500">"Employee tax withholding"</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <span class="text-2xl">"🔧"</span>
                                        </div>
                                        <div class="ml-4">
                                            <h4 class="text-sm font-medium text-gray-900">"PPh 23 Report"</h4>
                                            <p class="text-sm text-gray-500">"Service tax withholding"</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <span class="text-2xl">"🚛"</span>
                                        </div>
                                        <div class="ml-4">
                                            <h4 class="text-sm font-medium text-gray-900">"PPh 22 Report"</h4>
                                            <p class="text-sm text-gray-500">"Import/purchase withholding"</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <span class="text-2xl">"📅"</span>
                                        </div>
                                        <div class="ml-4">
                                            <h4 class="text-sm font-medium text-gray-900">"SPT Tahunan"</h4>
                                            <p class="text-sm text-gray-500">"Annual tax return"</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <span class="text-2xl">"📊"</span>
                                        </div>
                                        <div class="ml-4">
                                            <h4 class="text-sm font-medium text-gray-900">"Tax Summary"</h4>
                                            <p class="text-sm text-gray-500">"Comprehensive tax overview"</p>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // Report Generation Form
                            <div class="border-t pt-6">
                                <h4 class="text-md font-medium text-gray-900 mb-4">"Generate Custom Tax Report"</h4>
                                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-2">"Tax Type"</label>
                                        <select class="form-select">
                                            <option value="">"Select tax type"</option>
                                            <option value="PPN">"PPN (VAT)"</option>
                                            <option value="PPH21">"PPh 21"</option>
                                            <option value="PPH22">"PPh 22"</option>
                                            <option value="PPH23">"PPh 23"</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-2">"Period Start"</label>
                                        <input type="date" class="form-input"/>
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-2">"Period End"</label>
                                        <input type="date" class="form-input"/>
                                    </div>
                                    <div class="flex items-end">
                                        <button class="btn-primary w-full">"Generate Report"</button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}