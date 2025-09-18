use leptos::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-2xl font-bold text-gray-900">"Dashboard"</h1>
            </div>
            
            // Stats Cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatCard
                    title="Total Revenue"
                    value="Rp 0"
                    icon="💰"
                    color="green"
                />
                <StatCard
                    title="Total Expenses" 
                    value="Rp 0"
                    icon="💸"
                    color="red"
                />
                <StatCard
                    title="Net Income"
                    value="Rp 0"
                    icon="📊"
                    color="blue"
                />
                <StatCard
                    title="Tax Liability"
                    value="Rp 0"
                    icon="🧾"
                    color="yellow"
                />
            </div>

            // Recent Activity
            <div class="bg-white shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                    <h3 class="text-lg leading-6 font-medium text-gray-900 mb-4">
                        "Recent Transactions"
                    </h3>
                    <div class="text-center text-gray-500 py-8">
                        <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                        <h3 class="mt-2 text-sm font-medium text-gray-900">"No transactions yet"</h3>
                        <p class="mt-1 text-sm text-gray-500">"Start by creating your first journal entry."</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: String,
    #[prop(into)] color: String,
) -> impl IntoView {
    let color_classes = match color.as_str() {
        "green" => "text-green-600",
        "red" => "text-red-600", 
        "blue" => "text-blue-600",
        "yellow" => "text-yellow-600",
        _ => "text-gray-600",
    };

    view! {
        <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="p-5">
                <div class="flex items-center">
                    <div class="flex-shrink-0">
                        <span class="text-2xl">{icon}</span>
                    </div>
                    <div class="ml-5 w-0 flex-1">
                        <dl>
                            <dt class="text-sm font-medium text-gray-500 truncate">
                                {title}
                            </dt>
                            <dd class=format!("text-lg font-medium {}", color_classes)>
                                {value}
                            </dd>
                        </dl>
                    </div>
                </div>
            </div>
        </div>
    }
}