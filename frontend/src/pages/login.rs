use leptos::*;
use leptos_router::*;
use crate::api::auth;
use crate::stores::auth::AuthStore;

#[component]
pub fn LoginPage() -> impl IntoView {
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error = create_rw_signal(None::<String>);
    let loading = create_rw_signal(false);

    let auth_store = expect_context::<AuthStore>();
    let navigate = use_navigate();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        error.set(None);

        let email_val = email.get();
        let password_val = password.get();

        spawn_local(async move {
            match auth::login(&email_val, &password_val).await {
                Ok(response) => {
                    // Set user in auth store
                    navigate("/", NavigateOptions::default());
                }
                Err(e) => {
                    error.set(Some(format!("Login failed: {}", e)));
                }
            }
            loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
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
                    <div class="space-y-4">
                        <div>
                            <input
                                type="email"
                                required
                                class="relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                placeholder="Email address"
                                prop:value=move || email.get()
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <input
                                type="password"
                                required
                                class="relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                placeholder="Password"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
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
                            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
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