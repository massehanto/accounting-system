use leptos::*;
use crate::api::companies;
use crate::types::common::Company;
use crate::components::common::{Button, LoadingSpinner, Modal};
use crate::components::forms::Input;

#[component]
pub fn CompaniesPage() -> impl IntoView {
    let companies = create_rw_signal(Vec::<Company>::new());
    let loading = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);
    let show_edit_modal = create_rw_signal(false);
    let selected_company = create_rw_signal(None::<Company>);

    let load_companies = create_action(|_: &()| async move {
        loading.set(true);
        match companies::get_companies().await {
            Ok(data) => {
                companies.set(data);
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
        load_companies.dispatch(());
    });

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Company Management"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage your company information"</p>
                </div>
                <Button
                    on_click=Some(Callback::new(move |_| {
                        if let Some(company) = companies.get().first() {
                            selected_company.set(Some(company.clone()));
                            show_edit_modal.set(true);
                        }
                    }))
                >
                    "Edit Company"
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
                <div class="px-6 py-4 border-b border-gray-200">
                    <h3 class="text-lg leading-6 font-medium text-gray-900">
                        "Company Information"
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
                                        "No company information found."
                                    </div>
                                }
                            >
                                <CompanyInfo companies=companies/>
                            </Show>
                        }
                    >
                        <div class="flex justify-center py-8">
                            <LoadingSpinner message=Some("Loading company information...".to_string())/>
                        </div>
                    </Show>
                </div>
            </div>

            // Edit Modal
            <Modal
                show=show_edit_modal.into()
                on_close=Callback::new(move |_| {
                    show_edit_modal.set(false);
                    selected_company.set(None);
                })
                title=Some("Edit Company".to_string())
                size=Some("lg".to_string())
            >
                <Show when=move || selected_company.get().is_some()>
                    <EditCompanyForm 
                        company=selected_company.get().unwrap()
                        on_success=Callback::new(move |_| {
                            show_edit_modal.set(false);
                            load_companies.dispatch(());
                        })
                    />
                </Show>
            </Modal>
        </div>
    }
}

#[component]
fn CompanyInfo(companies: RwSignal<Vec<Company>>) -> impl IntoView {
    view! {
        {move || {
            if let Some(company) = companies.get().first() {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Company Name"</label>
                            <p class="mt-1 text-sm text-gray-900">{company.name.clone()}</p>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700">"NPWP"</label>
                            <p class="mt-1 text-sm text-gray-900 font-mono">
                                {crate::utils::formatting::format_npwp(&company.npwp)}
                            </p>
                        </div>
                        <div class="md:col-span-2">
                            <label class="block text-sm font-medium text-gray-700">"Address"</label>
                            <p class="mt-1 text-sm text-gray-900">{company.address.clone()}</p>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Phone"</label>
                            <p class="mt-1 text-sm text-gray-900">
                                {company.phone.clone().unwrap_or_else(|| "—".to_string())}
                            </p>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Email"</label>
                            <p class="mt-1 text-sm text-gray-900">
                                {company.email.clone().unwrap_or_else(|| "—".to_string())}
                            </p>
                        </div>
                    </div>
                }
            } else {
                view! { <div></div> }
            }
        }}
    }
}

#[component]
fn EditCompanyForm(company: Company, on_success: Callback<()>) -> impl IntoView {
    let name = create_rw_signal(company.name.clone());
    let address = create_rw_signal(company.address.clone());
    let phone = create_rw_signal(company.phone.unwrap_or_default());
    let email = create_rw_signal(company.email.unwrap_or_default());
    let loading = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);

    let submit_form = move |_| {
        loading.set(true);
        error.set(None);

        let update_data = serde_json::json!({
            "name": name.get(),
            "address": address.get(),
            "phone": if phone.get().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(phone.get()) },
            "email": if email.get().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(email.get()) }
        });

        let company_id = company.id.clone();
        spawn_local(async move {
            match companies::update_company(&company_id, update_data).await {
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
                    label="Company Name"
                    input_type="text"
                    value=name
                    required=Some(true)
                />
                
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        "Address" <span class="text-red-500">"*"</span>
                    </label>
                    <textarea
                        class="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                        rows="3"
                        required
                        prop:value=move || address.get()
                        on:input=move |ev| address.set(event_target_value(&ev))
                    ></textarea>
                </div>
                
                <Input
                    label="Phone"
                    input_type="tel"
                    value=phone
                />
                
                <Input
                    label="Email"
                    input_type="email"
                    value=email
                />
                
                <Show when=move || error.get().is_some()>
                    <div class="text-red-600 text-sm">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>
                
                <div class="flex justify-end space-x-3 pt-4">
                    <Button
                        variant=Some(crate::components::common::ButtonVariant::Secondary)
                        disabled=Some(loading.get())
                    >
                        "Cancel"
                    </Button>
                    <Button
                        loading=Some(loading.get())
                        disabled=Some(loading.get())
                    >
                        "Save Changes"
                    </Button>
                </div>
            </div>
        </form>
    }
}
