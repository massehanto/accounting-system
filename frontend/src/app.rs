use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::layout::AppLayout;
use crate::components::features::OfflineIndicator;
use crate::pages::*;
use crate::stores::{AuthStore, create_auth_store};
use crate::hooks::use_offline;
use crate::utils::storage;
use crate::api::auth;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    
    let (auth_state, set_auth_state) = create_auth_store();
    let (offline_state, _) = use_offline();
    
    provide_context(AuthStore {
        state: auth_state,
        set_state: set_auth_state,
    });
    provide_context(offline_state);

    view! {
        <Html lang="id"/>
        <Title text="Sistem Akuntansi Indonesia"/>
        <Meta name="description" content="Sistem akuntansi lengkap untuk perusahaan Indonesia dengan fitur jurnal, laporan keuangan, dan manajemen pajak"/>
        <Meta name="theme-color" content="#2563eb"/>
        <Meta name="apple-mobile-web-app-capable" content="yes"/>
        <Meta name="apple-mobile-web-app-status-bar-style" content="default"/>
        <Link rel="apple-touch-icon" href="/icons/icon-192x192.png"/>
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
    let auth_store = use_context::<AuthStore>()
        .expect("AuthStore not provided");
    
    let navigate = use_navigate();
    
    create_effect(move |_| {
        spawn_local(async move {
            auth_store.set_state.update(|state| state.is_loading = true);
            
            if let Some(token) = storage::get_token() {
                match auth::verify_token(&token).await {
                    Ok(_) => {
                        if let (Some(_user_id), Some(company_id)) = (storage::get_user_id(), storage::get_company_id()) {
                            auth_store.set_state.update(|state| {
                                state.is_authenticated = true;
                                state.token = Some(token);
                                state.company_id = Some(company_id);
                                state.is_loading = false;
                            });
                        } else {
                            storage::clear_user_data();
                            auth_store.set_state.update(|state| *state = Default::default());
                            navigate("/login", Default::default());
                        }
                    },
                    Err(_) => {
                        storage::clear_user_data();
                        auth_store.set_state.update(|state| *state = Default::default());
                        navigate("/login", Default::default());
                    }
                }
            } else {
                auth_store.set_state.update(|state| state.is_loading = false);
                navigate("/login", Default::default());
            }
        });
    });

    view! {
        <Suspense fallback=move || view! { <LoadingSpinner message="Memuat aplikasi..."/> }>
            <Show 
                when=move || !auth_store.state.get().is_loading
                fallback=|| view! { <LoadingSpinner message="Memverifikasi autentikasi..."/> }
            >
                <Show 
                    when=move || auth_store.state.get().is_authenticated 
                    fallback=|| view! { <LoginRedirect/> }
                >
                    <AppLayout>
                        <Routes>
                            <Route path="/" view=DashboardPage/>
                            <Route path="/companies" view=CompanyManagementPage/>
                            <Route path="/chart-of-accounts" view=ChartOfAccountsPage/>
                            <Route path="/journal-entries" view=JournalEntriesPage/>
                            <Route path="/accounts-payable" view=AccountsPayablePage/>
                            <Route path="/accounts-receivable" view=AccountsReceivablePage/>
                            <Route path="/inventory" view=InventoryPage/>
                            <Route path="/tax" view=TaxManagementPage/>
                            <Route path="/reports" view=ReportsPage/>
                        </Routes>
                    </AppLayout>
                </Show>
            </Show>
        </Suspense>
    }
}

#[component]
fn LoginRedirect() -> impl IntoView {
    let navigate = use_navigate();
    
    create_effect(move |_| {
        navigate("/login", Default::default());
    });

    view! {
        <div class="min-h-screen flex items-center justify-center">
            <LoadingSpinner message="Mengalihkan ke halaman login..."/>
        </div>
    }
}

#[component]
pub fn LoadingSpinner(message: String) -> impl IntoView {
    view! {
        <div class="flex flex-col justify-center items-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mb-4"></div>
            <p class="text-gray-600 text-sm">{message}</p>
        </div>
    }
}