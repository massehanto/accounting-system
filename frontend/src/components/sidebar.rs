// frontend/src/components/sidebar.rs
use leptos::*;
use leptos_router::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    
    let menu_items = vec![
        ("Dashboard", "/", "📊"),
        ("Companies", "/companies", "🏢"),
        ("Chart of Accounts", "/chart-of-accounts", "📋"),
        ("Journal Entries", "/journal-entries", "📝"),
        ("Accounts Payable", "/accounts-payable", "💸"),
        ("Accounts Receivable", "/accounts-receivable", "💰"),
        ("Inventory", "/inventory", "📦"),
        ("Tax Management", "/tax", "🧾"),
        ("Reports", "/reports", "📊"),
    ];

    view! {
        <div class="bg-gray-800 text-white w-64 space-y-6 py-7 px-2 absolute inset-y-0 left-0 transform -translate-x-full md:relative md:translate-x-0 transition duration-200 ease-in-out">
            <div class="text-white flex items-center space-x-2 px-4">
                <span class="text-2xl font-extrabold">"🧮"</span>
                <span class="text-2xl font-extrabold">"AccSys"</span>
            </div>
            
            <nav class="space-y-2">
                {menu_items.into_iter().map(|(name, path, icon)| {
                    let is_active = move || location.pathname.get() == path;
                    view! {
                        <A href=path class=move || {
                            let base = "flex items-center space-x-2 py-2.5 px-4 rounded transition duration-200";
                            if is_active() {
                                format!("{} bg-gray-700 text-white", base)
                            } else {
                                format!("{} text-gray-400 hover:bg-gray-700 hover:text-white", base)
                            }
                        }>
                            <span class="text-lg">{icon}</span>
                            <span>{name}</span>
                        </A>
                    }
                }).collect::<Vec<_>>()}
            </nav>
        </div>
    }
}