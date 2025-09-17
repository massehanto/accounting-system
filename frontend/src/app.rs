// frontend/src/app.rs
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::*;
use crate::pages::*;
use crate::stores::*;
use crate::hooks::*;
use crate::{api, utils};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    
    // Global state providers
    let (auth_state, set_auth_state) = create_auth_store();
    let (offline_state, _) = use_offline();
    
    provide_context(auth_state);
    provide_context(set_auth_state);
    provide_context(offline_state);

    // SEO and PWA meta tags
    view! {
        <Html lang="id"/>
        <Title text="Sistem Akuntansi Indonesia"/>
        <Meta name="description" content="Sistem akuntansi lengkap untuk perusahaan Indonesia dengan fitur jurnal, laporan keuangan, dan manajemen pajak"/>
        <Meta name="theme-color" content="#2563eb"/>
        <Meta name="apple-mobile-web-app-capable" content="yes"/>
        <Meta name="apple-mobile-web-app-status-bar-style" content="default"/>
        <Link rel="apple-touch-icon" href="/public/icons/icon-192x192.png"/>
        <Link rel="manifest" href="/public/manifest.json"/>
        
        <Router>
            <main class="min-h-screen bg-gray-50">
                // Global offline indicator
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
fn OfflineIndicator() -> impl IntoView {
    let offline_state = use_context::<ReadSignal<super::hooks::OfflineState>>()
        .expect("OfflineState not provided");

    view! {
        <Show when=move || !offline_state.get().is_online>
            <div class="fixed top-0 left-0 right-0 bg-red-600 text-white px-4 py-2 text-sm text-center z-50">
                "🔴 Tidak ada koneksi internet. Beberapa fitur mungkin terbatas."
            </div>
        </Show>
    }
}

#[component]
fn AuthenticatedApp() -> impl IntoView {
    let auth_state = use_context::<ReadSignal<AuthState>>()
        .expect("AuthState not provided");
    let set_auth_state = use_context::<WriteSignal<AuthState>>()
        .expect("AuthState setter not provided");
    
    // Check authentication on mount with better error handling
    create_effect(move |_| {
        spawn_local(async move {
            set_auth_state.update(|state| state.is_loading = true);
            
            if let Some(token) = utils::get_token() {
                match api::verify_token(&token).await {
                    Ok(_) => {
                        set_auth_state.update(|state| {
                            state.is_authenticated = true;
                            state.token = Some(token);
                            state.is_loading = false;
                        });
                    },
                    Err(_) => {
                        utils::clear_user_data();
                        set_auth_state.update(|state| {
                            *state = AuthState::default();
                        });
                        let navigate = use_navigate();
                        navigate("/login", Default::default());
                    }
                }
            } else {
                set_auth_state.update(|state| state.is_loading = false);
                let navigate = use_navigate();
                navigate("/login", Default::default());
            }
        });
    });

    view! {
        <Suspense fallback=move || view! { <LoadingSpinner message="Memuat aplikasi...".to_string()/> }>
            <Show 
                when=move || !auth_state.get().is_loading
                fallback=|| view! { <LoadingSpinner message="Memverifikasi autentikasi...".to_string()/> }
            >
                <Show 
                    when=move || auth_state.get().is_authenticated 
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
            <LoadingSpinner message="Mengalihkan ke halaman login...".to_string()/>
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