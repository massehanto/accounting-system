use leptos::*;
use crate::components::common::{Button, LoadingSpinner};

#[component]
pub fn InventoryPage() -> impl IntoView {
    let active_tab = create_rw_signal("items".to_string());

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Inventory Management"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage your inventory items and stock levels"</p>
                </div>
                <Button>
                    "Add Item"
                </Button>
            </div>

            // Tab Navigation
            <div class="border-b border-gray-200">
                <nav class="-mb-px flex space-x-8">
                    <TabButton
                        label="📦 Items"
                        tab_name="items"
                        active_tab=active_tab
                    />
                    <TabButton
                        label="📋 Transactions"
                        tab_name="transactions"
                        active_tab=active_tab
                    />
                    <TabButton
                        label="📊 Reports"
                        tab_name="reports"
                        active_tab=active_tab
                    />
                </nav>
            </div>

            // Tab Content
            <div class="bg-white shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                    <Show when=move || active_tab.get() == "items">
                        <InventoryItemsTab/>
                    </Show>
                    <Show when=move || active_tab.get() == "transactions">
                        <InventoryTransactionsTab/>
                    </Show>
                    <Show when=move || active_tab.get() == "reports">
                        <InventoryReportsTab/>
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TabButton(
    #[prop(into)] label: String,
    #[prop(into)] tab_name: String,
    active_tab: RwSignal<String>,
) -> impl IntoView {
    let is_active = move || active_tab.get() == tab_name;
    
    view! {
        <button
            class=move || {
                let base = "py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap";
                if is_active() {
                    format!("{} border-blue-500 text-blue-600", base)
                } else {
                    format!("{} border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300", base)
                }
            }
            on:click=move |_| active_tab.set(tab_name.clone())
        >
            {label}
        </button>
    }
}

#[component]
fn InventoryItemsTab() -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <div class="text-gray-500">
                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2 2v-5m16 0h-2M4 13h2m13-8V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v1M7 8h2M7 12h2" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">"No inventory items"</h3>
                <p class="mt-1 text-sm text-gray-500">"Get started by adding your first inventory item."</p>
            </div>
            <div class="mt-6">
                <Button>
                    "Add Item"
                </Button>
            </div>
        </div>
    }
}

#[component]
fn InventoryTransactionsTab() -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <h3 class="text-sm font-medium text-gray-900">"Inventory Transactions"</h3>
            <p class="mt-1 text-sm text-gray-500">"Track stock movements and adjustments."</p>
        </div>
    }
}

#[component]
fn InventoryReportsTab() -> impl IntoView {
    view! {
        <div class="text-center py-8">
            <h3 class="text-sm font-medium text-gray-900">"Inventory Reports"</h3>
            <p class="mt-1 text-sm text-gray-500">"View stock levels and valuation reports."</p>
        </div>
    }
}