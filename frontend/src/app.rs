use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::layout::AppLayout;
use crate::components::features::OfflineIndicator;
use crate::pages::*;
use crate::stores::auth::{AuthStore, provide_auth_store};
use crate::hooks::use_offline::use_offline;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_auth_store();
    
    let offline_state = use_offline();
    provide_context(offline_state);

    view! {
        <Html lang="id"/>
        <Title text="Sistem Akuntansi Indonesia"/>
        <Meta name="description" content="Sistem akuntansi lengkap untuk perusahaan Indonesia"/>
        <Meta name="theme-color" content="#2563eb"/>
        <Link rel="manifest" href="/manifest.json"/>
        
        <Router>
            <main class="min-h-screen bg-gray-50">
                <OfflineIndicator/>
                <Routes>
                    <Route path="/login" view=LoginPage/>
                    <Route path="/*any" view=AuthenticatedApp/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn AuthenticatedApp() -> impl IntoView {
    let auth_store = expect_context::<AuthStore>();
    let navigate = use_navigate();
    
    create_effect(move |_| {
        if !auth_store.is_authenticated() {
            navigate("/login", NavigateOptions::default());
        }
    });

    view! {
        <Show 
            when=move || auth_store.is_authenticated()
            fallback=move || view! { 
                <div class="min-h-screen flex items-center justify-center">
                    <div class="text-center">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
                        <p class="mt-4 text-gray-600">"Redirecting to login..."</p>
                    </div>
                </div>
            }
        >
            <AppLayout>
                <Routes>
                    <Route path="/" view=DashboardPage/>
                    <Route path="/companies" view=CompaniesPage/>
                    <Route path="/accounts" view=AccountsPage/>
                    <Route path="/journal-entries" view=JournalEntriesPage/>
                    <Route path="/vendors" view=VendorsPage/>
                    <Route path="/customers" view=CustomersPage/>
                    <Route path="/inventory" view=InventoryPage/>
                    <Route path="/tax" view=TaxPage/>
                    <Route path="/reports" view=ReportsPage/>
                </Routes>
            </AppLayout>
        </Show>
    }
}