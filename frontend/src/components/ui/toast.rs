use leptos::html::*;
use leptos::prelude::*;
use leptos_toaster::*;

pub use leptos_toaster::Toasts;

#[derive(PartialEq, Clone, Copy)]
pub enum ToastVariant {
    Normal,
    Success,
    Info,
    Warning,
    Error,
}

#[component]
pub fn ToastView(message: String, variant: ToastVariant, toast_id: ToastId) -> impl IntoView {
    let (bg_color, border_color, text_color, icon) = match variant {
        ToastVariant::Normal => ("bg-gray-800", "border-gray-400/40", "text-white", "💬"),
        ToastVariant::Success => ("bg-gray-900", "border-green-400/40", "text-green-300", "✅"),
        ToastVariant::Info => ("bg-gray-900", "border-blue-400/40", "text-blue-300", "ℹ️"),
        ToastVariant::Warning => (
            "bg-gray-800",
            "border-yellow-400/40",
            "text-yellow-300",
            "⚠️",
        ),
        ToastVariant::Error => ("bg-gray-900", "border-red-500/40", "text-red-200", "❌"),
    };

    view! {
        <div class=format!(
            "flex items-start gap-3 w-full max-w-sm p-4 rounded-xl border shadow-lg  {} {}",
            bg_color,
            border_color,
        )>
            <div class="text-xl leading-none">{icon}</div>
            <div class=format!(
                "flex-1 text-sm font-medium leading-snug {}",
                text_color,
            )>{message}</div>
            <button
                class="ml-2 text-white hover:text-gray-400 transition-colors"
                on:click=move |_| dismiss_toast(&toast_id)
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
            </button>
        </div>
    }
}

pub fn show_toast(toaster: Toasts, message: impl Into<String>, variant: ToastVariant) {
    let toast_id = ToastId::new();
    let message: String = message.into();
    toaster.toast(
        move || view! { <ToastView message=message.clone() variant=variant toast_id=toast_id /> },
        Some(toast_id),
        None,
    );
}
