use leptos::*;

#[component]
pub fn DataTable(
    headers: Vec<String>,
    rows: ReadSignal<Vec<Vec<String>>>,
) -> impl IntoView {
    view! {
        <div class="table-container">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        {headers.into_iter().map(|header| {
                            view! {
                                <th class="table-header">{header}</th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    {move || rows.get().into_iter().enumerate().map(|(index, row)| {
                        view! {
                            <tr class="table-row">
                                {row.into_iter().map(|cell| {
                                    view! {
                                        <td class="table-cell">{cell}</td>
                                    }
                                }).collect::<Vec<_>>()}
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
pub fn Pagination(
    current_page: ReadSignal<usize>,
    total_pages: ReadSignal<usize>,
    on_page_change: Callback<usize>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between px-4 py-3 bg-white border-t border-gray-200 sm:px-6">
            <div class="flex-1 flex justify-between sm:hidden">
                <button 
                    class="btn-secondary"
                    disabled=move || current_page.get() <= 1
                    on:click=move |_| {
                        if current_page.get() > 1 {
                            on_page_change.call(current_page.get() - 1);
                        }
                    }
                >
                    "Previous"
                </button>
                <button 
                    class="btn-secondary ml-3"
                    disabled=move || current_page.get() >= total_pages.get()
                    on:click=move |_| {
                        if current_page.get() < total_pages.get() {
                            on_page_change.call(current_page.get() + 1);
                        }
                    }
                >
                    "Next"
                </button>
            </div>
            <div class="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
                <div>
                    <p class="text-sm text-gray-700">
                        "Page " 
                        <span class="font-medium">{move || current_page.get()}</span>
                        " of "
                        <span class="font-medium">{move || total_pages.get()}</span>
                    </p>
                </div>
                <div>
                    <nav class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px">
                        <button 
                            class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                            disabled=move || current_page.get() <= 1
                            on:click=move |_| {
                                if current_page.get() > 1 {
                                    on_page_change.call(current_page.get() - 1);
                                }
                            }
                        >
                            "Previous"
                        </button>
                        
                        {move || (1..=total_pages.get()).map(|page| {
                            let is_current = page == current_page.get();
                            view! {
                                <button 
                                    class=format!("relative inline-flex items-center px-4 py-2 border text-sm font-medium {}",
                                        if is_current {
                                            "z-10 bg-indigo-50 border-indigo-500 text-indigo-600"
                                        } else {
                                            "bg-white border-gray-300 text-gray-500 hover:bg-gray-50"
                                        }
                                    )
                                    on:click=move |_| on_page_change.call(page)
                                >
                                    {page.to_string()}
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                        
                        <button 
                            class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50"
                            disabled=move || current_page.get() >= total_pages.get()
                            on:click=move |_| {
                                if current_page.get() < total_pages.get() {
                                    on_page_change.call(current_page.get() + 1);
                                }
                            }
                        >
                            "Next"
                        </button>
                    </nav>
                </div>
            </div>
        </div>
    }
}