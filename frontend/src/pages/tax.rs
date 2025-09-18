use leptos::*;
use crate::components::common::Button;

#[component]
pub fn TaxPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Indonesian Tax Management"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage PPN, PPh21, PPh22, PPh23, and other tax obligations"</p>
                </div>
                <Button>
                    "🧮 Tax Calculator"
                </Button>
            </div>

            // Indonesian Tax Information Panel
            <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <div class="flex items-start">
                    <div class="flex-shrink-0">
                        <span class="text-2xl">🇮🇩</span>
                    </div>
                    <div class="ml-3">
                        <h3 class="text-sm font-medium text-blue-800">
                            "Indonesian Tax Rates (2024)"
                        </h3>
                        <div class="mt-2 text-sm text-blue-700 grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div><strong>"PPN:"</strong> " 11% (Value Added Tax)"</div>
                            <div><strong>"PPh 21:"</strong> " Progressive rates (Employee income tax)"</div>
                            <div><strong>"PPh 22:"</strong> " 1.5% (Import/purchase withholding)"</div>
                            <div><strong>"PPh 23:"</strong> " 2% services, 10% rent (Service withholding)"</div>
                            <div><strong>"PTKP Single:"</strong> " Rp 54,000,000/year"</div>
                            <div><strong>"PTKP Married:"</strong> " Rp 58,500,000/year"</div>
                        </div>
                    </div>
                </div>
            </div>

            // Tax Management Grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <TaxCard
                    title="Tax Configurations"
                    description="Set up tax rates for Indonesian compliance"
                    icon="⚙️"
                />
                <TaxCard
                    title="Tax Transactions"
                    description="Record and track tax withholdings"
                    icon="📋"
                />
                <TaxCard
                    title="PPN Reports"
                    description="Generate monthly VAT returns"
                    icon="🧾"
                />
                <TaxCard
                    title="PPh 21 Reports"
                    description="Employee tax withholding reports"
                    icon="💼"
                />
                <TaxCard
                    title="PPh 23 Reports"
                    description="Service tax withholding reports"
                    icon="🔧"
                />
                <TaxCard
                    title="Annual Tax Return"
                    description="SPT Tahunan preparation"
                    icon="📅"
                />
            </div>
        </div>
    }
}

#[component]
fn TaxCard(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(into)] icon: String,
) -> impl IntoView {
    view! {
        <div class="bg-white border border-gray-200 rounded-lg shadow hover:shadow-md transition-shadow cursor-pointer">
            <div class="p-6">
                <div class="flex items-center">
                    <div class="flex-shrink-0">
                        <span class="text-3xl">{icon}</span>
                    </div>
                    <div class="ml-4">
                        <h3 class="text-lg font-medium text-gray-900">{title}</h3>
                        <p class="text-sm text-gray-500">{description}</p>
                    </div>
                </div>
            </div>
        </div>
    }
}