// frontend/src/components/layout.rs
use leptos::*;
use crate::components::{Header, Sidebar};

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let (sidebar_open, set_sidebar_open) = create_signal(true);
    
    view! {
        <div class="flex h-screen bg-gray-100 overflow-hidden">
            // Mobile sidebar backdrop
            <Show when=move || sidebar_open.get()>
                <div 
                    class="fixed inset-0 bg-black bg-opacity-50 z-40 lg:hidden"
                    on:click=move |_| set_sidebar_open.set(false)
                ></div>
            </Show>
            
            // Sidebar
            <div class={move || format!(
                "fixed inset-y-0 left-0 z-50 w-64 transform transition-transform duration-300 ease-in-out lg:translate-x-0 lg:static lg:inset-0 {}",
                if sidebar_open.get() { "translate-x-0" } else { "-translate-x-full" }
            )}>
                <Sidebar/>
            </div>
            
            // Main content area
            <div class="flex-1 flex flex-col overflow-hidden">
                <Header 
                    on_menu_click=move |_| set_sidebar_open.update(|open| *open = !*open)
                />
                <main class="flex-1 overflow-x-hidden overflow-y-auto bg-gray-50 p-4 lg:p-6">
                    {children()}
                </main>
            </div>
        </div>
    }
}