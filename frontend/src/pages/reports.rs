use leptos::*;
use crate::components::common::Button;

#[component]
pub fn ReportsPage() -> impl IntoView {
    let period_start = create_rw_signal(String::new());
    let period_end = create_rw_signal(String::new());
    let loading = create_rw_signal(false);

    // Set default dates
    create_effect(move |_| {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let year_start = chrono::Local::now().format("%Y-01-01").to_string();
        
        if period_end.get().is_empty() {
            period_end.set(today);
        }
        if period_start.get().is_empty() {
            period_start.set(year_start);
        }
    });

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Financial Reports"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Generate comprehensive financial reports"</p>
                </div>
                <Button
                    variant=Some(crate::components::common::ButtonVariant::Secondary)
                    disabled=Some(loading.get())
                >
                    "🔄 Refresh"
                </Button>
            </div>

            // Report Period Selection
            <div class="bg-white p-4 rounded-lg shadow">
                <h3 class="text-lg font-medium text-gray-900 mb-4">"Report Period"</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">"Period Start"</label>
                        <input 
                            type="date" 
                            class="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                            prop:value=move || period_start.get()
                            on:input=move |ev| period_start.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">"Period End"</label>
                        <input 
                            type="date" 
                            class="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                            prop:value=move || period_end.get()
                            on:input=move |ev| period_end.set(event_target_value(&ev))
                        />
                    </div>
                </div>
            </div>

            // Report Selection Grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <ReportCard
                    title="Balance Sheet"
                    subtitle="Laporan Posisi Keuangan"
                    description="View your company's assets, liabilities, and equity"
                    icon="⚖️"
                    loading=loading
                />
                <ReportCard
                    title="Income Statement"
                    subtitle="Laporan Laba Rugi"
                    description="Analyze revenues, expenses, and profit"
                    icon="📊"
                    loading=loading
                />
                <ReportCard
                    title="Cash Flow Statement"
                    subtitle="Laporan Arus Kas"
                    description="Track cash inflows and outflows"
                    icon="💰"
                    loading=loading
                />
                <ReportCard
                    title="Trial Balance"
                    subtitle="Neraca Saldo"
                    description="Verify that your books are balanced"
                    icon="📋"
                    loading=loading
                />
                <ReportCard
                    title="AP Aging Report"
                    subtitle="Laporan Umur Hutang"
                    description="Analyze outstanding vendor invoices"
                    icon="💸"
                    loading=loading
                />
                <ReportCard
                    title="AR Aging Report"
                    subtitle="Laporan Umur Piutang"
                    description="Track outstanding customer invoices"
                    icon="💳"
                    loading=loading
                />
            </div>
        </div>
    }
}

#[component]
fn ReportCard(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    #[prop(into)] description: String,
    #[prop(into)] icon: String,
    loading: RwSignal<bool>,
) -> impl IntoView {
    let generate_report = move |_| {
        loading.set(true);
        // Simulate report generation
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(2000).await;
            loading.set(false);
        });
    };

    view! {
        <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow">
            <div class="p-6">
                <div class="flex items-center mb-4">
                    <div class="flex-shrink-0">
                        <span class="text-3xl">{icon}</span>
                    </div>
                    <div class="ml-4">
                        <h3 class="text-lg font-medium text-gray-900">{title}</h3>
                        <p class="text-sm text-gray-500">{subtitle}</p>
                    </div>
                </div>
                <p class="text-sm text-gray-600 mb-4">{description}</p>
                <Button
                    full_width=Some(true)
                    disabled=Some(loading.get())
                    loading=Some(loading.get())
                    on_click=Some(Callback::new(generate_report))
                >
                    <Show 
                        when=move || loading.get()
                        fallback=|| format!("Generate {}", title)
                    >
                        "Generating..."
                    </Show>
                </Button>
            </div>
        </div>
    }
}