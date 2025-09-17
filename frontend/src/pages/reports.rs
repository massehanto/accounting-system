// frontend/src/pages/reports.rs
use leptos::*;
use crate::{api, utils};

#[component]
pub fn ReportsPage() -> impl IntoView {
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (active_report, set_active_report) = create_signal("".to_string());
    let (report_data, set_report_data) = create_signal(Option::<serde_json::Value>::None);
    
    // Report generation parameters
    let (period_start, set_period_start) = create_signal(String::new());
    let (period_end, set_period_end) = create_signal(String::new());

    let generate_report = move |report_type: &str| {
        let report_type = report_type.to_string();
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            set_active_report.set(report_type.clone());
            
            let result = match report_type.as_str() {
                "balance_sheet" => api::get_balance_sheet(&period_end.get()).await,
                "income_statement" => api::get_income_statement(&period_start.get(), &period_end.get()).await,
                "cash_flow" => api::get_cash_flow_statement(&period_start.get(), &period_end.get()).await,
                "trial_balance" => api::get_trial_balance(Some(&period_end.get())).await,
                _ => Err("Unknown report type".to_string()),
            };
            
            match result {
                Ok(data) => set_report_data.set(Some(data)),
                Err(e) => set_error.set(Some(utils::handle_api_error(&e))),
            }
            
            set_loading.set(false);
        });
    };

    // Set default dates
    create_effect(move |_| {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let year_start = chrono::Local::now().format("%Y-01-01").to_string();
        
        if period_end.get().is_empty() {
            set_period_end.set(today);
        }
        if period_start.get().is_empty() {
            set_period_start.set(year_start);
        }
    });

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Financial Reports"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Generate comprehensive financial reports for your business"</p>
                </div>
                <div class="flex space-x-3">
                    <button 
                        class="btn-secondary"
                        disabled=loading
                        on:click=move |_| {
                            if !active_report.get().is_empty() {
                                generate_report(&active_report.get());
                            }
                        }
                    >
                        "🔄 Refresh"
                    </button>
                </div>
            </div>

            // Report Period Selection
            <div class="bg-white p-4 rounded-lg shadow">
                <h3 class="text-lg font-medium text-gray-900 mb-4">"Report Period"</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">"Period Start"</label>
                        <input 
                            type="date" 
                            class="form-input"
                            prop:value=move || period_start.get()
                            on:input=move |ev| set_period_start.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">"Period End"</label>
                        <input 
                            type="date" 
                            class="form-input"
                            prop:value=move || period_end.get()
                            on:input=move |ev| set_period_end.set(event_target_value(&ev))
                        />
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

            // Report Selection Grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                // Balance Sheet
                <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow">
                    <div class="p-6">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <span class="text-3xl">"⚖️"</span>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-medium text-gray-900">"Balance Sheet"</h3>
                                <p class="text-sm text-gray-500">"Laporan Posisi Keuangan"</p>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm text-gray-600">"View your company's assets, liabilities, and equity as of a specific date."</p>
                        </div>
                        <div class="mt-6">
                            <button 
                                class="btn-primary w-full"
                                disabled=loading
                                on:click=move |_| generate_report("balance_sheet")
                            >
                                <Show 
                                    when=move || loading.get() && active_report.get() == "balance_sheet"
                                    fallback=|| "Generate Balance Sheet"
                                >
                                    "Generating..."
                                </Show>
                            </button>
                        </div>
                    </div>
                </div>

                // Income Statement
                <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow">
                    <div class="p-6">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <span class="text-3xl">"📊"</span>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-medium text-gray-900">"Income Statement"</h3>
                                <p class="text-sm text-gray-500">"Laporan Laba Rugi"</p>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm text-gray-600">"Analyze your company's revenues, expenses, and profit for a specific period."</p>
                        </div>
                        <div class="mt-6">
                            <button 
                                class="btn-primary w-full"
                                disabled=loading
                                on:click=move |_| generate_report("income_statement")
                            >
                                <Show 
                                    when=move || loading.get() && active_report.get() == "income_statement"
                                    fallback=|| "Generate Income Statement"
                                >
                                    "Generating..."
                                </Show>
                            </button>
                        </div>
                    </div>
                </div>

                // Cash Flow Statement
                <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow">
                    <div class="p-6">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <span class="text-3xl">"💰"</span>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-medium text-gray-900">"Cash Flow Statement"</h3>
                                <p class="text-sm text-gray-500">"Laporan Arus Kas"</p>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm text-gray-600">"Track cash inflows and outflows from operating, investing, and financing activities."</p>
                        </div>
                        <div class="mt-6">
                            <button 
                                class="btn-primary w-full"
                                disabled=loading
                                on:click=move |_| generate_report("cash_flow")
                            >
                                <Show 
                                    when=move || loading.get() && active_report.get() == "cash_flow"
                                    fallback=|| "Generate Cash Flow"
                                >
                                    "Generating..."
                                </Show>
                            </button>
                        </div>
                    </div>
                </div>

                // Trial Balance
                <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow">
                    <div class="p-6">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <span class="text-3xl">"📋"</span>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-medium text-gray-900">"Trial Balance"</h3>
                                <p class="text-sm text-gray-500">"Neraca Saldo"</p>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm text-gray-600">"Verify that your books are balanced with a complete list of all accounts."</p>
                        </div>
                        <div class="mt-6">
                            <button 
                                class="btn-primary w-full"
                                disabled=loading
                                on:click=move |_| generate_report("trial_balance")
                            >
                                <Show 
                                    when=move || loading.get() && active_report.get() == "trial_balance"
                                    fallback=|| "Generate Trial Balance"
                                >
                                    "Generating..."
                                </Show>
                            </button>
                        </div>
                    </div>
                </div>

                // Accounts Payable Report
                <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow">
                    <div class="p-6">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <span class="text-3xl">"💸"</span>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-medium text-gray-900">"AP Aging Report"</h3>
                                <p class="text-sm text-gray-500">"Laporan Umur Hutang"</p>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm text-gray-600">"Analyze outstanding vendor invoices by age categories."</p>
                        </div>
                        <div class="mt-6">
                            <button 
                                class="btn-primary w-full"
                                disabled=loading
                                on:click=move |_| {
                                    // Would implement AP aging report generation
                                    utils::show_info_toast("AP Aging report coming soon!");
                                }
                            >
                                "Generate AP Report"
                            </button>
                        </div>
                    </div>
                </div>

                // Accounts Receivable Report
                <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow">
                    <div class="p-6">
                        <div class="flex items-center">
                            <div class="flex-shrink-0">
                                <span class="text-3xl">"💰"</span>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-medium text-gray-900">"AR Aging Report"</h3>
                                <p class="text-sm text-gray-500">"Laporan Umur Piutang"</p>
                            </div>
                        </div>
                        <div class="mt-4">
                            <p class="text-sm text-gray-600">"Track outstanding customer invoices and collection status."</p>
                        </div>
                        <div class="mt-6">
                            <button 
                                class="btn-primary w-full"
                                disabled=loading
                                on:click=move |_| {
                                    // Would implement AR aging report generation
                                    utils::show_info_toast("AR Aging report coming soon!");
                                }
                            >
                                "Generate AR Report"
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            // Report Display Area
            <Show when=move || report_data.get().is_some()>
                <div class="bg-white shadow rounded-lg">
                    <div class="px-4 py-5 sm:p-6">
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-lg font-medium text-gray-900">
                                {move || match active_report.get().as_str() {
                                    "balance_sheet" => "Balance Sheet Report",
                                    "income_statement" => "Income Statement Report", 
                                    "cash_flow" => "Cash Flow Statement Report",
                                    "trial_balance" => "Trial Balance Report",
                                    _ => "Financial Report",
                                }}
                            </h3>
                            <div class="flex space-x-2">
                                <button class="btn-secondary">"📄 Export PDF"</button>
                                <button class="btn-secondary">"📊 Export Excel"</button>
                                <button class="btn-secondary">"🖨️ Print"</button>
                            </div>
                        </div>
                        
                        {move || {
                            if let Some(data) = report_data.get() {
                                match active_report.get().as_str() {
                                    "trial_balance" => {
                                        if let Some(accounts) = data.get("accounts").and_then(|a| a.as_array()) {
                                            let total_debits = data.get("total_debits")
                                                .and_then(|t| t.as_f64())
                                                .unwrap_or(0.0);
                                            let total_credits = data.get("total_credits")
                                                .and_then(|t| t.as_f64())
                                                .unwrap_or(0.0);
                                            let is_balanced = data.get("is_balanced")
                                                .and_then(|b| b.as_bool())
                                                .unwrap_or(false);
                                            
                                            view! {
                                                <div class="space-y-4">
                                                    <div class="text-center">
                                                        <h4 class="text-md font-medium text-gray-900">
                                                            "Trial Balance as of " {period_end.get()}
                                                        </h4>
                                                    </div>
                                                    
                                                    <div class="overflow-x-auto">
                                                        <table class="min-w-full divide-y divide-gray-200">
                                                            <thead class="bg-gray-50">
                                                                <tr>
                                                                    <th class="table-header text-left">"Account Code"</th>
                                                                    <th class="table-header text-left">"Account Name"</th>
                                                                    <th class="table-header text-left">"Account Type"</th>
                                                                    <th class="table-header text-right">"Debit"</th>
                                                                    <th class="table-header text-right">"Credit"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody class="bg-white divide-y divide-gray-200">
                                                                {accounts.iter().map(|account| {
                                                                    let account_code = account.get("account_code")
                                                                        .and_then(|c| c.as_str())
                                                                        .unwrap_or("");
                                                                    let account_name = account.get("account_name")
                                                                        .and_then(|n| n.as_str())
                                                                        .unwrap_or("");
                                                                    let account_type = account.get("account_type")
                                                                        .and_then(|t| t.as_str())
                                                                        .unwrap_or("");
                                                                    let debit_balance = account.get("debit_balance")
                                                                        .and_then(|d| d.as_f64())
                                                                        .unwrap_or(0.0);
                                                                    let credit_balance = account.get("credit_balance")
                                                                        .and_then(|c| c.as_f64())
                                                                        .unwrap_or(0.0);
                                                                    
                                                                    view! {
                                                                        <tr>
                                                                            <td class="table-cell font-mono">{account_code}</td>
                                                                            <td class="table-cell">{account_name}</td>
                                                                            <td class="table-cell">{account_type}</td>
                                                                            <td class="table-cell text-right">
                                                                                {if debit_balance > 0.0 {
                                                                                    utils::format_currency(debit_balance)
                                                                                } else {
                                                                                    "—".to_string()
                                                                                }}
                                                                            </td>
                                                                            <td class="table-cell text-right">
                                                                                {if credit_balance > 0.0 {
                                                                                    utils::format_currency(credit_balance)
                                                                                } else {
                                                                                    "—".to_string()
                                                                                }}
                                                                            </td>
                                                                        </tr>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </tbody>
                                                            <tfoot class="bg-gray-50">
                                                                <tr class="font-medium">
                                                                    <td class="table-cell" colspan="3">"Total"</td>
                                                                    <td class="table-cell text-right">{utils::format_currency(total_debits)}</td>
                                                                    <td class="table-cell text-right">{utils::format_currency(total_credits)}</td>
                                                                </tr>
                                                            </tfoot>
                                                        </table>
                                                    </div>
                                                    
                                                    <div class="text-center">
                                                        <span class={format!("inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {}", 
                                                            if is_balanced { "bg-green-100 text-green-800" } else { "bg-red-100 text-red-800" })}>
                                                            {if is_balanced { "✅ Books are balanced" } else { "❌ Books are not balanced" }}
                                                        </span>
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            view! {
                                                <div class="text-center text-gray-500">
                                                    "No trial balance data available"
                                                </div>
                                            }
                                        }
                                    },
                                    _ => {
                                        view! {
                                            <div class="text-center text-gray-500">
                                                <p>"Report data structure not yet implemented for " {active_report.get()}</p>
                                                <pre class="mt-4 text-left bg-gray-100 p-4 rounded text-xs overflow-auto">
                                                    {serde_json::to_string_pretty(&data).unwrap_or_default()}
                                                </pre>
                                            </div>
                                        }
                                    }
                                }
                            } else {
                                view! {
                                    <div class="text-center text-gray-500">
                                        "No report data available"
                                    </div>
                                }
                            }
                        }}
                    </div>
                </div>
            </Show>
        </div>
    }
}