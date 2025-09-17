// frontend/src/pages/company_management.rs
use leptos::*;
use crate::{api, utils};

#[component]
pub fn CompanyManagementPage() -> impl IntoView {
    let (companies, set_companies) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (selected_company, set_selected_company) = create_signal(Option::<serde_json::Value>::None);

    // Load companies on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match api::get_companies().await {
                Ok(company_list) => {
                    set_companies.set(company_list);
                    set_error.set(None);
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
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Manajemen Perusahaan"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Kelola informasi profil perusahaan Anda"</p>
                </div>
                <button 
                    class="btn-primary"
                    on:click=move |_| {
                        if let Some(company) = companies.get().first() {
                            set_selected_company.set(Some(company.clone()));
                            set_show_edit_modal.set(true);
                        }
                    }
                >
                    "Edit Perusahaan"
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
                <div class="px-6 py-4 border-b border-gray-200">
                    <h3 class="text-lg leading-6 font-medium text-gray-900">
                        "Informasi Perusahaan"
                    </h3>
                </div>
                <div class="px-6 py-5">
                    <Show 
                        when=move || loading.get()
                        fallback=move || view! {
                            <Show
                                when=move || !companies.get().is_empty()
                                fallback=|| view! {
                                    <div class="text-center text-gray-500">
                                        "Tidak ada data perusahaan ditemukan."
                                    </div>
                                }
                            >
                                {move || {
                                    if let Some(company) = companies.get().first() {
                                        let name = company.get("name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        let npwp = company.get("npwp")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("");
                                        let address = company.get("address")
                                            .and_then(|a| a.as_str())
                                            .unwrap_or("");
                                        let phone = company.get("phone")
                                            .and_then(|p| p.as_str())
                                            .unwrap_or("");
                                        let email = company.get("email")
                                            .and_then(|e| e.as_str())
                                            .unwrap_or("");
                                        let business_type = company.get("business_type")
                                            .and_then(|b| b.as_str())
                                            .unwrap_or("");

                                        view! {
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">
                                                        "Nama Perusahaan"
                                                    </label>
                                                    <p class="mt-1 text-sm text-gray-900">{name}</p>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">
                                                        "NPWP"
                                                    </label>
                                                    <p class="mt-1 text-sm text-gray-900 font-mono">
                                                        {utils::format_npwp(npwp)}
                                                    </p>
                                                </div>
                                                <div class="md:col-span-2">
                                                    <label class="block text-sm font-medium text-gray-700">
                                                        "Alamat"
                                                    </label>
                                                    <p class="mt-1 text-sm text-gray-900">{address}</p>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">
                                                        "Telepon"
                                                    </label>
                                                    <p class="mt-1 text-sm text-gray-900">
                                                        {if phone.is_empty() { "—" } else { phone }}
                                                    </p>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700">
                                                        "Email"
                                                    </label>
                                                    <p class="mt-1 text-sm text-gray-900">
                                                        {if email.is_empty() { "—" } else { email }}
                                                    </p>
                                                </div>
                                                <div class="md:col-span-2">
                                                    <label class="block text-sm font-medium text-gray-700">
                                                        "Jenis Usaha"
                                                    </label>
                                                    <p class="mt-1 text-sm text-gray-900">
                                                        {if business_type.is_empty() { "—" } else { business_type }}
                                                    </p>
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        view! { <div></div> }
                                    }
                                }}
                            </Show>
                        }
                    >
                        <div class="text-center">
                            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                            <p class="mt-2 text-sm text-gray-600">"Memuat data perusahaan..."</p>
                        </div>
                    </Show>
                </div>
            </div>

            // Edit Company Modal
            <Show when=move || show_edit_modal.get()>
                <EditCompanyModal 
                    show=show_edit_modal
                    company=selected_company
                    on_close=move |_| {
                        set_show_edit_modal.set(false);
                        set_selected_company.set(None);
                    }
                    on_updated=move |_| {
                        set_show_edit_modal.set(false);
                        // Reload companies
                        spawn_local(async move {
                            match api::get_companies().await {
                                Ok(company_list) => set_companies.set(company_list),
                                Err(e) => set_error.set(Some(utils::handle_api_error(&e))),
                            }
                        });
                    }
                />
            </Show>
        </div>
    }
}

#[component]
fn EditCompanyModal<F1, F2>(
    show: ReadSignal<bool>,
    company: ReadSignal<Option<serde_json::Value>>,
    on_close: F1,
    on_updated: F2,
) -> impl IntoView 
where
    F1: Fn(()) + 'static + Copy,
    F2: Fn(()) + 'static + Copy,
{
    let (name, set_name) = create_signal(String::new());
    let (address, set_address) = create_signal(String::new());
    let (phone, set_phone) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (business_type, set_business_type) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Initialize form when company changes
    create_effect(move |_| {
        if let Some(comp) = company.get() {
            set_name.set(comp.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string());
            set_address.set(comp.get("address").and_then(|a| a.as_str()).unwrap_or("").to_string());
            set_phone.set(comp.get("phone").and_then(|p| p.as_str()).unwrap_or("").to_string());
            set_email.set(comp.get("email").and_then(|e| e.as_str()).unwrap_or("").to_string());
            set_business_type.set(comp.get("business_type").and_then(|b| b.as_str()).unwrap_or("").to_string());
        }
    });

    let submit_form = move |_| {
        set_loading.set(true);
        set_error.set(None);

        if let Some(comp) = company.get() {
            if let Some(id_str) = comp.get("id").and_then(|id| id.as_str()) {
                let update_data = serde_json::json!({
                    "name": name.get(),
                    "address": address.get(),
                    "phone": if phone.get().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(phone.get()) },
                    "email": if email.get().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(email.get()) },
                    "business_type": if business_type.get().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(business_type.get()) }
                });

                spawn_local(async move {
                    match reqwest::Client::new()
                        .put(&format!("/api/companies/{}", id_str))
                        .json(&update_data)
                        .send()
                        .await
                    {
                        Ok(response) if response.status().is_success() => {
                            on_updated(());
                        }
                        Ok(response) => {
                            set_error.set(Some(format!("Gagal memperbarui: {}", response.status())));
                        }
                        Err(e) => {
                            set_error.set(Some(format!("Error: {}", e)));
                        }
                    }
                    set_loading.set(false);
                });
            }
        }
    };

    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
                <div class="relative top-20 mx-auto p-5 border w-11/12 max-w-2xl shadow-lg rounded-md bg-white">
                    <div class="mt-3">
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-lg font-medium text-gray-900">"Edit Perusahaan"</h3>
                            <button 
                                class="text-gray-400 hover:text-gray-600"
                                on:click=move |_| on_close(())
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>

                        <Show when=move || error.get().is_some()>
                            <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
                                <div class="text-sm text-red-700">
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </div>
                        </Show>

                        <form on:submit=move |ev| {
                            ev.prevent_default();
                            submit_form(());
                        }>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                                <div class="md:col-span-2">
                                    <label class="form-label">"Nama Perusahaan *"</label>
                                    <input
                                        type="text"
                                        class="form-input"
                                        required
                                        prop:value=move || name.get()
                                        on:input=move |ev| set_name.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="md:col-span-2">
                                    <label class="form-label">"Alamat *"</label>
                                    <textarea
                                        class="form-input"
                                        rows="3"
                                        required
                                        prop:value=move || address.get()
                                        on:input=move |ev| set_address.set(event_target_value(&ev))
                                    ></textarea>
                                </div>
                                <div>
                                    <label class="form-label">"Telepon"</label>
                                    <input
                                        type="tel"
                                        class="form-input"
                                        prop:value=move || phone.get()
                                        on:input=move |ev| set_phone.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="form-label">"Email"</label>
                                    <input
                                        type="email"
                                        class="form-input"
                                        prop:value=move || email.get()
                                        on:input=move |ev| set_email.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="md:col-span-2">
                                    <label class="form-label">"Jenis Usaha"</label>
                                    <input
                                        type="text"
                                        class="form-input"
                                        prop:value=move || business_type.get()
                                        on:input=move |ev| set_business_type.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <div class="flex justify-end space-x-3">
                                <button 
                                    type="button"
                                    class="btn-secondary"
                                    on:click=move |_| on_close(())
                                    disabled=loading
                                >
                                    "Batal"
                                </button>
                                <button 
                                    type="submit"
                                    class="btn-primary"
                                    disabled=loading
                                >
                                    <Show 
                                        when=move || loading.get()
                                        fallback=|| "Simpan Perubahan"
                                    >
                                        "Menyimpan..."
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