use leptos::*;
use crate::components::common::{Button, LoadingSpinner};

#[component]
pub fn CustomersPage() -> impl IntoView {
    let loading = create_rw_signal(false);
    let customers = create_rw_signal(Vec::<serde_json::Value>::new());

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Customer Management"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage your customer relationships"</p>
                </div>
                <Button>
                    "Add Customer"
                </Button>
            </div>

            <div class="bg-white shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                    <div class="text-center py-8">
                        <div class="text-gray-500">
                            <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                            </svg>
                            <h3 class="mt-2 text-sm font-medium text-gray-900">"No customers"</h3>
                            <p class="mt-1 text-sm text-gray-500">"Get started by adding your first customer."</p>
                        </div>
                        <div class="mt-6">
                            <Button>
                                "Add Customer"
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}