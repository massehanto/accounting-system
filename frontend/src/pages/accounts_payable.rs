// frontend/src/pages/accounts_payable.rs
use leptos::*;
use crate::{api, utils};

#[component]
pub fn AccountsPayablePage() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("vendors".to_string());
    
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Accounts Payable"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Kelola vendor dan tagihan yang harus dibayar"</p>
                </div>
            </div>

            // Tab Navigation
            <div class="border-b border-gray-200">
                <nav class="-mb-px flex space-x-8">
                    <button 
                        class=move || {
                            let base = "py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap";
                            if active_tab.get() == "vendors" {
                                format!("{} border-indigo-500 text-indigo-600", base)
                            } else {
                                format!("{} border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300", base)
                            }
                        }
                        on:click=move |_| set_active_tab.set("vendors".to_string())
                    >
                        "👥 Vendor"
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
                        "🧾 Tagihan"
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
                <Show when=move || active_tab.get() == "vendors">
                    <VendorManagement/>
                </Show>
                <Show when=move || active_tab.get() == "invoices">
                    <VendorInvoiceManagement/>
                </Show>
                <Show when=move || active_tab.get() == "aging">
                    <AgingReportView/>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn VendorManagement() -> impl IntoView {
    let (vendors, set_vendors) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = create_signal(false);

    let load_vendors = create_action(|_: &()| async move {
        set_loading.set(true);
        match api::get_vendors().await {
            Ok(vendor_list) => {
                set_vendors.set(vendor_list);
                set_error.set(None);
            }
            Err(e) => {
                set_error.set(Some(utils::handle_api_error(&e)));
            }
        }
        set_loading.set(false);
    });

    // Load vendors on mount
    create_effect(move |_| {
        load_vendors.dispatch(());
    });

    view! {
        <div class="space-y-4">
            <div class="flex justify-between items-center">
                <h3 class="text-lg font-medium text-gray-900">"Manajemen Vendor"</h3>
                <button 
                    class="btn-primary"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "Tambah Vendor"
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
                            when=move || !vendors.get().is_empty()
                            fallback=|| view! {
                                <div class="p-8 text-center">
                                    <div class="text-gray-500">
                                        <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                        </svg>
                                        <h3 class="mt-2 text-sm font-medium text-gray-900">"Tidak ada vendor"</h3>
                                        <p class="mt-1 text-sm text-gray-500">"Mulai dengan menambahkan vendor pertama Anda."</p>
                                    </div>
                                    <div class="mt-6">
                                        <button 
                                            class="btn-primary"
                                            on:click=move |_| set_show_create_modal.set(true)
                                        >
                                            "Tambah Vendor"
                                        </button>
                                    </div>
                                </div>
                            }
                        >
                            <table class="min-w-full divide-y divide-gray-200">
                                <thead class="bg-gray-50">
                                    <tr>
                                        <th class="table-header">"Kode"</th>
                                        <th class="table-header">"Nama Vendor"</th>
                                        <th class="table-header">"NPWP"</th>
                                        <th class="table-header">"Telepon"</th>
                                        <th class="table-header">"Status"</th>
                                        <th class="table-header">"Aksi"</th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white divide-y divide-gray-200">
                                    {move || vendors.get().into_iter().map(|vendor| {
                                        let code = vendor.get("vendor_code")
                                            .and_then(|c| c.as_str())
                                            .unwrap_or("");
                                        let name = vendor.get("vendor_name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        let npwp = vendor.get("npwp")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        let phone = vendor.get("phone")
                                            .and_then(|p| p.as_str())
                                            .unwrap_or("");
                                        let is_active = vendor.get("is_active")
                                            .and_then(|a| a.as_bool())
                                            .unwrap_or(false);
                                        
                                        view! {
                                            <tr class="hover:bg-gray-50">
                                                <td class="table-cell font-mono">{code}</td>
                                                <td class="table-cell font-medium">{name}</td>
                                                <td class="table-cell font-mono">
                                                    {if npwp.is_empty() { "—" } else { &utils::format_npwp(npwp) }}
                                                </td>
                                                <td class="table-cell">
                                                    {if phone.is_empty() { "—" } else { phone }}
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
                        <p class="mt-2 text-sm text-gray-600">"Memuat vendor..."</p>
                    </div>
                </Show>
            </div>

            // Create Vendor Modal placeholder
            <Show when=move || show_create_modal.get()>
                <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
                    <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-md shadow-lg rounded-md bg-white">
                        <div class="text-center">
                            <h3 class="text-lg font-medium text-gray-900 mb-4">"Tambah Vendor"</h3>
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
fn VendorInvoiceManagement() -> impl IntoView {
    view! {
        <div class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">"Manajemen Tagihan Vendor"</h3>
            <div class="text-center text-gray-500">
                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">"Belum ada tagihan"</h3>
                <p class="mt-1 text-sm text-gray-500">"Tagihan vendor akan muncul di sini."</p>
            </div>
        </div>
    }
}

#[component]
fn AgingReportView() -> impl IntoView {
    view! {
        <div class="bg-white shadow rounded-lg p-6">
            <h3 class="text-lg font-medium text-gray-900 mb-4">"Laporan Aging Accounts Payable"</h3>
            <div class="text-center text-gray-500">
                <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">"Laporan Aging"</h3>
                <p class="mt-1 text-sm text-gray-500">"Analisis umur hutang akan ditampilkan di sini."</p>
            </div>
        </div>
    }
}