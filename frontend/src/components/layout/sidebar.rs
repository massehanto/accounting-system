use leptos::*;
use leptos_router::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    
    let menu_items = vec![
        ("Dashboard", "/", "📊"),
        ("Companies", "/companies", "🏢"),
        ("Accounts", "/accounts", "📋"),
        ("Journal Entries", "/journal-entries", "📝"),
        ("Vendors", "/vendors", "💸"),
        ("Customers", "/customers", "💰"),
        ("Inventory", "/inventory", "📦"),
        ("Tax Management", "/tax", "🧾"),
        ("Reports", "/reports", "📊"),
    ];

    view! {
        <div class="bg-gray-800 text-white w-64 h-full flex flex-col">
            <div class="flex items-center space-x-2 px-6 py-4 border-b border-gray-700">
                <span class="text-2xl font-extrabold">"🧮"</span>
                <span class="text-xl font-bold">"AccSys"</span>
            </div>
            <nav class="flex-1 px-4 py-6 space-y-2">
                {menu_items.into_iter().map(|(name, path, icon)| {
                    let is_active = move || location.pathname.get() == path;
                    view! {
                        <A href=path class=move || {
                            let base = "flex items-center space-x-3 py-2.5 px-3 rounded-lg transition duration-200";
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