use leptos::*;
use leptos_router::*;
use crate::stores::auth::AuthStore;

#[component]
pub fn Header(
    #[prop(optional)] on_menu_click: Option<Callback<()>>,
) -> impl IntoView {
    let auth_store = expect_context::<AuthStore>();
    let navigate = use_navigate();

    let logout = move |_| {
        auth_store.logout();
        navigate("/login", NavigateOptions::default());
    };

    view! {
        <header class="bg-white shadow-sm border-b border-gray-200">
            <div class="flex items-center justify-between px-6 py-4">
                <div class="flex items-center">
                    <Show when=move || on_menu_click.is_some()>
                        <button
                            class="mr-4 lg:hidden p-2 rounded-md hover:bg-gray-100"
                            on:click=move |_| {
                                if let Some(handler) = on_menu_click {
                                    handler.call(());
                                }
                            }
                        >
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                            </svg>
                        </button>
                    </Show>
                    <h1 class="text-xl font-semibold text-gray-800">
                        "Sistem Akuntansi Indonesia"
                    </h1>
                </div>
                
                <div class="flex items-center space-x-4">
                    <div class="relative">
                        <button class="flex items-center space-x-2 text-gray-600 hover:text-gray-800">
                            <span class="text-sm">"👤"</span>
                            <span class="text-sm font-medium">
                                {move || auth_store.get_user().map(|u| u.full_name).unwrap_or_else(|| "User".to_string())}
                            </span>
                        </button>
                    </div>
                    
                    <button 
                        on:click=logout
                        class="text-sm text-gray-600 hover:text-gray-800 px-3 py-1 border border-gray-300 rounded hover:bg-gray-50"
                    >
                        "Logout"
                    </button>
                </div>
            </div>
        </header>
    }
}