// frontend/src/components/header.rs
use leptos::*;
use leptos_router::use_navigate;
use crate::utils;

#[component]
pub fn Header() -> impl IntoView {
    let logout = move |_| {
        utils::remove_token();
        let navigate = use_navigate();
        navigate("/login", Default::default());
    };

    view! {
        <header class="bg-white shadow-sm border-b border-gray-200">
            <div class="flex items-center justify-between px-6 py-4">
                <div class="flex items-center">
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