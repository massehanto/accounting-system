use leptos::*;

#[component]
pub fn StatusBadge(
    status: String,
    #[prop(optional)] variant: Option<String>, // "success", "warning", "danger", "info"
) -> impl IntoView {
    let badge_class = match variant.as_deref().unwrap_or("info") {
        "success" => "badge-success",
        "warning" => "badge-warning", 
        "danger" => "badge-danger",
        _ => "badge-info",
    };

    view! {
        <span class=format!("badge {}", badge_class)>
            {status}
        </span>
    }
}

#[component] 
pub fn JournalStatusBadge(status: crate::types::JournalEntryStatus) -> impl IntoView {
    let (variant, text) = match status {
        crate::types::JournalEntryStatus::Draft => ("secondary", "Draft"),
        crate::types::JournalEntryStatus::PendingApproval => ("warning", "Pending Approval"),
        crate::types::JournalEntryStatus::Approved => ("info", "Approved"),
        crate::types::JournalEntryStatus::Posted => ("success", "Posted"),
        crate::types::JournalEntryStatus::Cancelled => ("danger", "Cancelled"),
    };

    view! {
        <StatusBadge status=text.to_string() variant=Some(variant.to_string())/>
    }
}