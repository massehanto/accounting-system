// frontend/src/components/header.rs
use leptos::*;
use leptos_router::*;
use crate::stores::*;

// Update the Header component signature to accept the menu click handler:
#[component]
pub fn Header(
    #[prop(optional)] on_menu_click: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let logout = move |_| {
        crate::utils::remove_token();
        let navigate = use_navigate();
        navigate("/login", Default::default());
    };

    view! {
        <header class="bg-white shadow-sm border-b border-gray-200">
            <div class="flex items-center justify-between px-6 py-4">
                <div class="flex items-center">
                    // Mobile menu button
                    <Show when=move || on_menu_click.is_some()>
                        <button
                            class="mr-4 lg:hidden p-2 rounded-md hover:bg-gray-100"
                            on:click=move |_| {
                                if let Some(ref handler) = on_menu_click {
                                    handler();
                                }
                            }
                        >
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                            </svg>
                        </button>
                    </Show>
                    <h1 class="text-xl font-semibold text-gray-800">
                        "Indonesian Accounting System"
                    </h1>
                </div>
                
                <div class="flex items-center space-x-4">
                    <div class="relative">
                        <button class="flex items-center space-x-2 text-gray-600 hover:text-gray-800">
                            <span class="text-sm">"👤"</span>
                            <span class="text-sm font-medium">"User"</span>
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