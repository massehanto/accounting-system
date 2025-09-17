// frontend/src/app.rs
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::*;
use crate::pages::*;
use crate::{api, utils};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/accounting-frontend.css"/>
        <Title text="Indonesian Accounting System"/>
        <Router>
            <main class="min-h-screen bg-gray-50">
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
    let (is_authenticated, set_is_authenticated) = create_signal(false);
    let (loading, set_loading) = create_signal(true);
    
    // Check authentication on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            
            if let Some(token) = utils::get_token() {
                match api::verify_token(&token).await {
                    Ok(_) => {
                        set_is_authenticated.set(true);
                    },
                    Err(_) => {
                        utils::clear_user_data();
                        let navigate = use_navigate();
                        navigate("/login", Default::default());
                    }
                }
            } else {
                let navigate = use_navigate();
                navigate("/login", Default::default());
            }
            
            set_loading.set(false);
        });
    });

    view! {
        <Show 
            when=move || !loading.get()
            fallback=|| view! { <LoadingSpinner/> }
        >
            <Show 
                when=move || is_authenticated.get() 
                fallback=|| view! { <LoginRedirect/> }
            >
                <div class="flex h-screen bg-gray-100">
                    <Sidebar/>
                    <div class="flex-1 flex flex-col overflow-hidden">
                        <Header/>
                        <main class="flex-1 overflow-x-hidden overflow-y-auto bg-gray-50 p-6">
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
                        </main>
                    </div>
                </div>
            </Show>
        </Show>
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
            <div class="text-center">
                <LoadingSpinner/>
                <p class="mt-4 text-gray-600">"Redirecting to login..."</p>
            </div>
        </div>
    }
}

#[component]
fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex justify-center items-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
        </div>
    }
}