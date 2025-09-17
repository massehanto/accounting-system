// frontend/src/pages/inventory.rs
use leptos::*;
use crate::{api, utils};

#[component]
pub fn InventoryPage() -> impl IntoView {
    let (items, set_items) = create_signal(Vec::<serde_json::Value>::new());
    let (transactions, set_transactions) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (active_tab, set_active_tab) = create_signal("items".to_string());
    let (show_create_item_modal, set_show_create_item_modal) = create_signal(false);
    let (show_transaction_modal, set_show_transaction_modal) = create_signal(false);
    let (show_adjustment_modal, set_show_adjustment_modal) = create_signal(false);
    let (stock_report, set_stock_report) = create_signal(Option::<serde_json::Value>::None);

    // Load inventory items on mount
    create_effect(move |_| {
        if active_tab.get() == "items" {
            spawn_local(async move {
                set_loading.set(true);
                match api::get_inventory_items().await {
                    Ok(items_list) => {
                        set_items.set(items_list);
                    }
                    Err(e) => {
                        set_error.set(Some(utils::handle_api_error(&e)));
                    }
                }
                set_loading.set(false);
            });
        }
    });

    // Load transactions when tab changes
    create_effect(move |_| {
        if active_tab.get() == "transactions" {
            spawn_local(async move {
                set_loading.set(true);
                // This would call a hypothetical get_inventory_transactions API
                // For now, we'll use empty transactions
                set_transactions.set(Vec::new());
                set_loading.set(false);
            });
        }
    });

    // Load stock report when tab changes
    create_effect(move |_| {
        if active_tab.get() == "reports" {
            spawn_local(async move {
                set_loading.set(true);
                // This would call a hypothetical get_stock_report API
                // For now, we'll create a mock report
                let mock_report = serde_json::json!({
                    "summary": {
                        "total_items": 0,
                        "total_stock_value": 0,
                        "out_of_stock_items": 0,
                        "low_stock_items": 0
                    },
                    "alerts": {
                        "low_stock_alerts": [],
                        "reorder_suggestions": []
                    }
                });
                set_stock_report.set(Some(mock_report));
                set_loading.set(false);
            });
        }
    });

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Inventory Management"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage your inventory items, stock levels, and transactions"</p>
                </div>
                <div class="flex space-x-3">
                    <button 
                        class="btn-secondary"
                        on:click=move |_| {
                            match active_tab.get().as_str() {
                                "items" => spawn_local(async move {
                                    set_loading.set(true);
                                    match api::get_inventory_items().await {
                                        Ok(items_list) => set_items.set(items_list),
                                        Err(e) => set_error.set(Some(utils::handle_api_error(&e))),
                                    }
                                    set_loading.set(false);
                                }),
                                "transactions" => set_transactions.set(Vec::new()),
                                _ => {}
                            }
                        }
                    >
                        "Refresh"
                    </button>
                    <Show when=move || active_tab.get() == "items">
                        <button 
                            class="btn-primary"
                            on:click=move |_| set_show_create_item_modal.set(true)
                        >
                            "Add Item"
                        </button>
                    </Show>
                    <Show when=move || active_tab.get() == "transactions">
                        <div class="flex space-x-2">
                            <button 
                                class="btn-primary"
                                on:click=move |_| set_show_transaction_modal.set(true)
                            >
                                "New Transaction"
                            </button>
                            <button 
                                class="btn-secondary"
                                on:click=move |_| set_show_adjustment_modal.set(true)
                            >
                                "Stock Adjustment"
                            </button>
                        </div>
                    </Show>
                </div>
            </div>

            // Tab Navigation
            <div class="border-b border-gray-200">
                <nav class="-mb-px flex space-x-8">
                    <button
                        class={move || if active_tab.get() == "items" {
                            "border-primary-500 text-primary-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        } else {
                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        }}
                        on:click=move |_| set_active_tab.set("items".to_string())
                    >
                        "📦 Items"
                    </button>
                    <button
                        class={move || if active_tab.get() == "transactions" {
                            "border-primary-500 text-primary-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        } else {
                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        }}
                        on:click=move |_| set_active_tab.set("transactions".to_string())
                    >
                        "📋 Transactions"
                    </button>
                    <button
                        class={move || if active_tab.get() == "reports" {
                            "border-primary-500 text-primary-600 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        } else {
                            "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm"
                        }}
                        on:click=move |_| set_active_tab.set("reports".to_string())
                    >
                        "📊 Stock Reports"
                    </button>
                </nav>
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
                // Items Tab
                <Show when=move || active_tab.get() == "items">
                    <div class="px-4 py-5 sm:p-6">
                        <Show 
                            when=move || loading.get()
                            fallback=move || view! {
                                <Show
                                    when=move || !items.get().is_empty()
                                    fallback=|| view! {
                                        <div class="text-center py-8">
                                            <div class="text-gray-500">
                                                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2 2v-5m16 0h-2M4 13h2m13-8V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v1M7 8h2M7 12h2" />
                                                </svg>
                                                <h3 class="mt-2 text-sm font-medium text-gray-900">"No inventory items"</h3>
                                                <p class="mt-1 text-sm text-gray-500">"Get started by adding your first inventory item."</p>
                                            </div>
                                            <div class="mt-6">
                                                <button 
                                                    class="btn-primary"
                                                    on:click=move |_| set_show_create_item_modal.set(true)
                                                >
                                                    "Add Item"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                >
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="table-header">"Item Code"</th>
                                                <th class="table-header">"Item Name"</th>
                                                <th class="table-header">"Type"</th>
                                                <th class="table-header">"Unit Cost"</th>
                                                <th class="table-header">"Selling Price"</th>
                                                <th class="table-header">"Stock Level"</th>
                                                <th class="table-header">"Status"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {move || items.get().into_iter().map(|item| {
                                                let item_code = item.get("item_code")
                                                    .and_then(|c| c.as_str())
                                                    .unwrap_or("");
                                                let item_name = item.get("item_name")
                                                    .and_then(|n| n.as_str())
                                                    .unwrap_or("");
                                                let item_type = item.get("item_type")
                                                    .and_then(|t| t.as_str())
                                                    .unwrap_or("");
                                                let unit_cost = item.get("unit_cost")
                                                    .and_then(|c| c.as_f64())
                                                    .unwrap_or(0.0);
                                                let selling_price = item.get("selling_price")
                                                    .and_then(|p| p.as_f64())
                                                    .unwrap_or(0.0);
                                                let quantity_on_hand = item.get("quantity_on_hand")
                                                    .and_then(|q| q.as_f64())
                                                    .unwrap_or(0.0);
                                                let reorder_level = item.get("reorder_level")
                                                    .and_then(|r| r.as_f64())
                                                    .unwrap_or(0.0);
                                                let is_active = item.get("is_active")
                                                    .and_then(|a| a.as_bool())
                                                    .unwrap_or(false);
                                                
                                                let stock_status = if quantity_on_hand <= 0.0 {
                                                    ("Out of Stock", "badge-danger")
                                                } else if quantity_on_hand <= reorder_level {
                                                    ("Low Stock", "badge-warning")
                                                } else {
                                                    ("In Stock", "badge-success")
                                                };
                                                
                                                view! {
                                                    <tr class="hover:bg-gray-50">
                                                        <td class="table-cell font-mono">{item_code}</td>
                                                        <td class="table-cell">{item_name}</td>
                                                        <td class="table-cell">{item_type}</td>
                                                        <td class="table-cell">{utils::format_currency(unit_cost)}</td>
                                                        <td class="table-cell">{utils::format_currency(selling_price)}</td>
                                                        <td class="table-cell">{format!("{:.2}", quantity_on_hand)}</td>
                                                        <td class="table-cell">
                                                            <span class={format!("badge {}", stock_status.1)}>
                                                                {stock_status.0}
                                                            </span>
                                                        </td>
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
                                <p class="mt-2 text-sm text-gray-600">"Loading inventory items..."</p>
                            </div>
                        </Show>
                    </div>
                </Show>

                // Transactions Tab
                <Show when=move || active_tab.get() == "transactions">
                    <div class="px-4 py-5 sm:p-6">
                        <div class="text-center py-8">
                            <div class="text-gray-500">
                                <h3 class="text-sm font-medium text-gray-900">"Inventory Transactions"</h3>
                                <p class="mt-1 text-sm text-gray-500">"Track stock movements and adjustments."</p>
                            </div>
                        </div>
                    </div>
                </Show>

                // Reports Tab
                <Show when=move || active_tab.get() == "reports">
                    <div class="px-4 py-5 sm:p-6">
                        <Show when=move || stock_report.get().is_some()>
                            {move || {
                                if let Some(report) = stock_report.get() {
                                    let summary = report.get("summary").cloned().unwrap_or_default();
                                    let total_items = summary.get("total_items")
                                        .and_then(|t| t.as_i64())
                                        .unwrap_or(0);
                                    let total_value = summary.get("total_stock_value")
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(0.0);
                                    let out_of_stock = summary.get("out_of_stock_items")
                                        .and_then(|o| o.as_i64())
                                        .unwrap_or(0);
                                    let low_stock = summary.get("low_stock_items")
                                        .and_then(|l| l.as_i64())
                                        .unwrap_or(0);
                                    
                                    view! {
                                        <div class="space-y-6">
                                            <h3 class="text-lg font-medium text-gray-900">"Stock Report Summary"</h3>
                                            
                                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                                                <div class="bg-blue-50 p-4 rounded-lg">
                                                    <div class="text-2xl font-bold text-blue-600">{total_items}</div>
                                                    <div class="text-sm text-blue-600">"Total Items"</div>
                                                </div>
                                                <div class="bg-green-50 p-4 rounded-lg">
                                                    <div class="text-2xl font-bold text-green-600">{utils::format_currency(total_value)}</div>
                                                    <div class="text-sm text-green-600">"Total Value"</div>
                                                </div>
                                                <div class="bg-yellow-50 p-4 rounded-lg">
                                                    <div class="text-2xl font-bold text-yellow-600">{low_stock}</div>
                                                    <div class="text-sm text-yellow-600">"Low Stock Items"</div>
                                                </div>
                                                <div class="bg-red-50 p-4 rounded-lg">
                                                    <div class="text-2xl font-bold text-red-600">{out_of_stock}</div>
                                                    <div class="text-sm text-red-600">"Out of Stock"</div>
                                                </div>
                                            </div>
                                            
                                            <div class="text-center text-gray-500">
                                                <p>"Detailed stock analysis and reorder recommendations will appear here."</p>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    view! { <div></div> }
                                }
                            }}
                        </Show>
                    </div>
                </Show>
            </div>
        </div>
    }
}