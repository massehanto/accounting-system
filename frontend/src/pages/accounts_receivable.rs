// frontend/src/pages/accounts_receivable.rs
use leptos::*;
use crate::{api, utils};

#[component]
pub fn AccountsReceivablePage() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("customers".to_string());
    
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Accounts Receivable"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Kelola pelanggan dan piutang yang akan diterima"</p>
                </div>
            </div>

            // Tab Navigation
            <div class="border-b border-gray-200">
                <nav class="-mb-px flex space-x-8">
                    <button 
                        class=move || {
                            let base = "py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap";
                            if active_tab.get() == "customers" {
                                format!("{} border-indigo-500 text-indigo-600", base)
                            } else {
                                format!("{} border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300", base)
                            }
                        }
                        on:click=move |_| set_active_tab.set("customers".to_string())
                    >
                        "👤 Pelanggan"
                    </button>
                    <button 
                        class=move || {
                            let base = "py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap";
                            if active_tab.get() == "invoices" {
                                format!("{} border-indigo-500 text-indigo-600", base)
                            } else {
                                format!("{} border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300", base)
                            }
                        }
                        on:click=move |_| set_active_tab.set("invoices".to_string())
                    >
                        "🧾 Faktur"
                    </button>
                    <button 
                        class=move || {
                            let base = "py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap";
                            if active_tab.get() == "collections" {
                                format!("{} border-indigo-500 text-indigo-600", base)
                            } else {
                                format!("{} border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300", base)
                            }
                        }
                        on:click=move |_| set_active_tab.set("collections".to_string())
                    >
                        "💰 Penagihan"
                    </button>
                    <button 
                        class=move || {
                            let base = "py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap";
                            if active_tab.get() == "aging" {
                                format!("{} border-indigo-500 text-indigo-600", base)
                            } else {
                                format!("{} border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300", base)
                            }
                        }
                        on:click=move |_| set_active_tab.set("aging".to_string())
                    >
                        "📊 Aging Report"
                    </button>
                </nav>
            </div>

            // Tab Content
            <div class="mt-6">
                <Show when=move || active_tab.get() == "customers">
                    <CustomerManagement/>
                </Show>
                <Show when=move || active_tab.get() == "invoices">
                    <CustomerInvoiceManagement/>
                </Show>
                <Show when=move || active_tab.get() == "collections">
                    <CollectionManagement/>
                </Show>
                <Show when=move || active_tab.get() == "aging">
                    <CustomerAgingReport/>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn CustomerManagement() -> impl IntoView {
    let (customers, set_customers) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = create_signal(false);

    let load_customers = create_action(|_: &()| async move {
        set_loading.set(true);
        match api::get_customers().await {
            Ok(customer_list) => {
                set_customers.set(customer_list);
                set_error.set(None);
            }
            Err(e) => {
                set_error.set(Some(utils::handle_api_error(&e)));
            }
        }
        set_loading.set(false);
    });

    // Load customers on mount
    create_effect(move |_| {
        load_customers.dispatch(());
    });

    view! {
        <div class="space-y-4">
            <div class="flex justify-between items-center">
                <h3 class="text-lg font-medium text-gray-900">"Manajemen Pelanggan"</h3>
                <button 
                    class="btn-primary"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "Tambah Pelanggan"
                </button>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                    <div class="text-sm text-red-700">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </div>
            </Show>

            <div class="bg-white shadow rounded-lg overflow-hidden">
                <Show 
                    when=move || loading.get()
                    fallback=move || view! {
                        <Show
                            when=move || !customers.get().is_empty()
                            fallback=|| view! {
                                <div class="p-8 text-center">
                                    <div class="text-gray-500">
                                        <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                        </svg>
                                        <h3 class="mt-2 text-sm font-medium text-gray-900">"Tidak ada pelanggan"</h3>
                                        <p class="mt-1 text-sm text-gray-500">"Mulai dengan menambahkan pelanggan pertama Anda."</p>
                                    </div>
                                    <div class="mt-6">
                                        <button 
                                            class="btn-primary"
                                            on:click=move |_| set_show_create_modal.set(true)
                                        >
                                            "Tambah Pelanggan"
                                        </button>
                                    </div>
                                </div>
                            }
                        >
                            <table class="min-w-full divide-y divide-gray-200">
                                <thead class="bg-gray-50">
                                    <tr>
                                        <th class="table-header">"Kode"</th>
                                        <th class="table-header">"Nama Pelanggan"</th>
                                        <th class="table-header">"NPWP"</th>
                                        <th class="table-header">"Limit Kredit"</th>
                                        <th class="table-header">"Status"</th>
                                        <th class="table-header">"Aksi"</th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white divide-y divide-gray-200">
                                    {move || customers.get().into_iter().map(|customer| {
                                        let code = customer.get("customer_code")
                                            .and_then(|c| c.as_str())
                                            .unwrap_or("");
                                        let name = customer.get("customer_name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        let npwp = customer.get("npwp")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        let credit_limit = customer.get("credit_limit")
                                            .and_then(|c| c.as_str())
                                            .and_then(|s| s.parse::<f64>().ok())
                                            .unwrap_or(0.0);
                                        let is_active = customer.get("is_active")
                                            .and_then(|a| a.as_bool())
                                            .unwrap_or(false);
                                        
                                        view! {
                                            <tr class="hover:bg-gray-50">
                                                <td class="table-cell font-mono">{code}</td>
                                                <td class="table-cell font-medium">{name}</td>
                                                <td class="table-cell font-mono">
                                                    {if npwp.is_empty() { "—" } else { &utils::format_npwp(npwp) }}
                                                </td>
                                                <td class="table-cell text-right">
                                                    {utils::format_currency(credit_limit)}
                                                </td>
                                                <td class="table-cell">
                                                    <span class={format!("badge {}", 
                                                        if is_active { "badge-success" } else { "badge-secondary" })}>
                                                        {if is_active { "Aktif" } else { "Tidak Aktif" }}
                                                    </span>
                                                </td>
                                                <td class="table-cell">
                                                    <div class="flex space-x-2">
                                                        <button class="text-indigo-600 hover:text-indigo-900 text-sm">
                                                            "Edit"
                                                        </button>
                                                        <button class="text-green-600 hover:text-green-900 text-sm">
                                                            "Lihat"
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </Show>
                    }
                >
                    <div class="p-8 text-center">
                        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                        <p class="mt-2 text-sm text-gray-600">"Memuat pelanggan..."</p>
                    </div>
                </Show>
            </div>

            // Create Customer Modal placeholder
            <Show when=move || show_create_modal.get()>
                <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
                    <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-md shadow-lg rounded-md bg-white">
                        <div class="text-center">
                            <h3 class="text-lg font-medium text-gray-900 mb-4">"Tambah Pelanggan"</h3>
                            <p class="text-sm text-gray-500 mb-4">"Fitur ini sedang dalam pengembangan."</p>
                            <button 
                                class="btn-secondary"
                                on:click=move |_| set_show_create_modal.set(false)
                            >
                                "Tutup"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn CustomerInvoiceManagement() -> impl IntoView {
    view! {
        <div class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">"Manajemen Faktur Pelanggan"</h3>
            <div class="text-center text-gray-500">
                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">"Belum ada faktur"</h3>
                <p class="mt-1 text-sm text-gray-500">"Faktur pelanggan akan muncul di sini."</p>
            </div>
        </div>
    }
}

#[component]
fn CollectionManagement() -> impl IntoView {
    view! {
        <div class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">"Manajemen Penagihan"</h3>
            <div class="text-center text-gray-500">
                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">"Belum ada penagihan"</h3>
                <p class="mt-1 text-sm text-gray-500">"Aktivitas penagihan akan ditampilkan di sini."</p>
            </div>
        </div>
    }
}

#[component]
fn CustomerAgingReport() -> impl IntoView {
    view! {
        <div class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">"Laporan Aging Accounts Receivable"</h3>
            <div class="text-center text-gray-500">
                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">"Laporan Aging"</h3>
                <p class="mt-1 text-sm text-gray-500">"Analisis umur piutang akan ditampilkan di sini."</p>
            </div>
        </div>
    }
}