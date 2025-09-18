use leptos::*;
use crate::components::common::{Button, LoadingSpinner};

#[component]
pub fn VendorsPage() -> impl IntoView {
    let loading = create_rw_signal(false);
    let vendors = create_rw_signal(Vec::<serde_json::Value>::new());

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Vendor Management"</h1>
                    <p class="text-sm text-gray-600 mt-1">"Manage your vendor relationships"</p>
                </div>
                <Button>
                    "Add Vendor"
                </Button>
            </div>

            <div class="bg-white shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                    <div class="text-center py-8">
                        <div class="text-gray-500">
                            <svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                            </svg>
                            <h3 class="mt-2 text-sm font-medium text-gray-900">"No vendors"</h3>
                            <p class="mt-1 text-sm text-gray-500">"Get started by adding your first vendor."</p>
                        </div>
                        <div class="mt-6">
                            <Button>
                                "Add Vendor"
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}