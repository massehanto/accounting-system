// frontend/src/pages/login.rs
use leptos::*;
use leptos_router::*;
use crate::{api, utils};

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);

    let navigate = use_navigate();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let password_val = password.get();

        spawn_local(async move {
            match api::login(&email_val, &password_val).await {
                Ok(response) => {
                    utils::set_token(&response.token);
                    navigate("/", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(format!("Login failed: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="max-w-md w-full space-y-8">
                <div>
                    <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
                        "Sign in to your account"
                    </h2>
                    <p class="mt-2 text-center text-sm text-gray-600">
                        "Indonesian Accounting System"
                    </p>
                </div>
                <form class="mt-8 space-y-6" on:submit=on_submit>
                    <div class="rounded-md shadow-sm -space-y-px">
                        <div>
                            <input
                                type="email"
                                required
                                class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                                placeholder="Email address"
                                prop:value=move || email.get()
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <input
                                type="password"
                                required
                                class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
                                placeholder="Password"
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    <Show when=move || error.get().is_some()>
                        <div class="text-red-600 text-sm text-center">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <div>
                        <button
                            type="submit"
                            disabled=move || loading.get()
                            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                        >
                            <Show when=move || loading.get() fallback=|| "Sign in">
                                "Signing in..."
                            </Show>
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}