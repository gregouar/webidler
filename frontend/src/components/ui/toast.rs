use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use leptos::{portal::Portal, prelude::*};

use super::buttons::CloseButton;

const DEFAULT_TOAST_DURATION: Duration = Duration::from_secs(5);
const TOAST_EXIT_DURATION: Duration = Duration::from_millis(240);
const STACK_PREVIEW_LIMIT: usize = 5;
static NEXT_TOAST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug)]
pub struct Toasts {
    entries: RwSignal<Vec<ToastData>>,
}

#[derive(Clone, Debug)]
struct ToastData {
    id: ToastId,
    message: String,
    variant: ToastVariant,
    is_exiting: RwSignal<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(u64);

impl Default for ToastId {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastId {
    pub fn new() -> Self {
        Self(NEXT_TOAST_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Toasts {
    pub fn show(self, message: impl Into<String>, variant: ToastVariant) {
        let toast = ToastData {
            id: ToastId::new(),
            message: message.into(),
            variant,
            is_exiting: RwSignal::new(false),
        };
        let toast_id = toast.id;

        self.entries.update(|entries| {
            if entries.len() >= STACK_PREVIEW_LIMIT {
                entries.remove(0);
            }
            entries.push(toast);
        });

        set_timeout(move || self.dismiss(toast_id), DEFAULT_TOAST_DURATION);
    }

    pub fn dismiss(self, toast_id: ToastId) {
        let should_schedule = self.entries.with_untracked(|entries| {
            entries
                .iter()
                .find(|toast| toast.id == toast_id)
                .map(|toast| !toast.is_exiting.get_untracked())
                .unwrap_or(false)
        });

        if !should_schedule {
            return;
        }

        self.entries.update(|entries| {
            if let Some(toast) = entries.iter_mut().find(|toast| toast.id == toast_id) {
                toast.is_exiting.set(true);
            }
        });

        set_timeout(move || self.remove(toast_id), TOAST_EXIT_DURATION);
    }

    fn remove(self, toast_id: ToastId) {
        self.entries
            .update(|entries| entries.retain(|toast| toast.id != toast_id));
    }
}

pub fn provide_toasts() -> Toasts {
    let toasts = Toasts {
        entries: RwSignal::new(Vec::new()),
    };
    provide_context(toasts);
    toasts
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToasterPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastVariant {
    Normal,
    Success,
    Info,
    Warning,
    Error,
}

#[component]
fn ToastIcon(variant: ToastVariant) -> impl IntoView {
    let badge_class = match variant {
        ToastVariant::Normal => "text-stone-200",
        ToastVariant::Success => "text-emerald-300",
        ToastVariant::Info => "text-sky-300",
        ToastVariant::Warning => "text-amber-300",
        ToastVariant::Error => "text-rose-300",
    };

    view! {
        <div class=format!(
            "relative flex h-9 w-9 shrink-0 items-center justify-center rounded-[6px]
            border border-[#6c5734]/55
            bg-[linear-gradient(180deg,#3d3941,#26232a)]
            shadow-[inset_1px_1px_0_rgba(255,255,255,0.04),inset_-1px_-1px_0_rgba(0,0,0,0.22)] {}",
            badge_class,
        )>
            <div class="pointer-events-none absolute inset-[1px] rounded-[5px] border border-white/6"></div>
            {match variant {
                ToastVariant::Normal => {
                    view! {
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="h-5 w-5"
                        >
                            <path d="M8 10h8" />
                            <path d="M8 14h5" />
                            <path d="M7.2 19.2L4 20l.8-3.2A7 7 0 1 1 19 12a7 7 0 0 1-11.8 7.2Z" />
                        </svg>
                    }
                        .into_any()
                }
                ToastVariant::Success => {
                    view! {
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="h-5 w-5"
                        >
                            <path d="m20 6-11 11-5-5" />
                        </svg>
                    }
                        .into_any()
                }
                ToastVariant::Info => {
                    view! {
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="h-5 w-5"
                        >
                            <circle cx="12" cy="12" r="9" />
                            <path d="M12 10v6" />
                            <path d="M12 7.25h.01" />
                        </svg>
                    }
                        .into_any()
                }
                ToastVariant::Warning => {
                    view! {
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="h-5 w-5"
                        >
                            <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.72 3h16.92a2 2 0 0 0 1.72-3L13.71 3.86a2 2 0 0 0-3.42 0Z" />
                            <path d="M12 9v4.5" />
                            <path d="M12 17h.01" />
                        </svg>
                    }
                        .into_any()
                }
                ToastVariant::Error => {
                    view! {
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.8"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="h-5 w-5"
                        >
                            <circle cx="12" cy="12" r="9" />
                            <path d="M15 9 9 15" />
                            <path d="m9 9 6 6" />
                        </svg>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ToastView(
    message: String,
    variant: ToastVariant,
    is_exiting: RwSignal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (accent_class, label_class, text_class, title) = match variant {
        ToastVariant::Normal => (
            "bg-stone-200/60",
            "text-stone-300/85",
            "text-stone-100",
            "Notice",
        ),
        ToastVariant::Success => (
            "bg-emerald-300/70",
            "text-emerald-300/90",
            "text-stone-100",
            "Success",
        ),
        ToastVariant::Info => ("bg-sky-300/70", "text-sky-300/90", "text-stone-100", "Info"),
        ToastVariant::Warning => (
            "bg-amber-300/75",
            "text-amber-300/95",
            "text-stone-100",
            "Warning",
        ),
        ToastVariant::Error => (
            "bg-rose-300/75",
            "text-rose-300/95",
            "text-stone-100",
            "Error",
        ),
    };

    view! {
        <div
            role="status"
            class=move || {
                format!(
                    "group relative overflow-hidden rounded-[8px]
                    border border-[#6c5734]/55
                    bg-[linear-gradient(180deg,#433f47,#27242b)]
                    shadow-[0_8px_18px_rgba(0,0,0,0.3),inset_1px_1px_0_rgba(255,255,255,0.04),inset_-1px_-1px_0_rgba(0,0,0,0.22)]
                    transition-[opacity,transform] duration-200 will-change-transform
                    {}",
                    if is_exiting.get() {
                        "animate-[toast-slide-out_240ms_cubic-bezier(0.55,0,1,0.45)_forwards]"
                    } else {
                        "animate-[toast-slide-in_260ms_cubic-bezier(0.22,1,0.36,1)]"
                    },
                )
            }
        >
            <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-white/6"></div>
            <div class="pointer-events-none absolute inset-x-4 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/30 to-transparent"></div>
            <div class=format!(
                "pointer-events-none absolute inset-y-2 left-[3px] w-[2px] rounded-full {}",
                accent_class,
            )></div>
            <div class="relative flex items-start gap-3 px-4 py-3">
                <ToastIcon variant />
                <div class="min-w-0 flex-1">
                    <div class=format!(
                        "text-[10px] font-semibold uppercase tracking-[0.18em] {}",
                        label_class,
                    )>
                        {title}
                    </div>
                    <div class=format!("mt-0.5 text-sm font-medium leading-5 {}", text_class)>
                        {message}
                    </div>
                </div>
                <div class="shrink-0 pt-0.5 opacity-70 transition-opacity duration-150 group-hover:opacity-100">
                    <CloseButton on:click=move |_| on_close.run(()) />
                </div>
            </div>
        </div>
    }
}

fn collapsed_stack_item_class(preview_rank: usize) -> &'static str {
    if preview_rank >= STACK_PREVIEW_LIMIT {
        return "opacity-0 pointer-events-none z-0 overflow-hidden";
    }

    match preview_rank {
        0 => "z-50",
        1 => "z-40",
        2 => "z-30",
        3 => "z-20",
        _ => "z-10",
    }
}

fn stack_item_style(
    stack_index: usize,
    preview_rank: usize,
    is_hovered: bool,
    is_top: bool,
) -> String {
    if is_hovered {
        let margin_top = if stack_index == 0 { 0.0 } else { 8.0 };
        return format!(
            "margin-top:{margin_top}px; max-height:12rem; transform: translate3d(0,0,0) scale(1);"
        );
    }

    if preview_rank >= STACK_PREVIEW_LIMIT {
        let direction = if is_top { 1.0 } else { -1.0 };
        let margin_top = if stack_index == 0 { 0.0 } else { -60.0 };
        return format!(
            "margin-top:{margin_top}px; max-height:0; transform: translate3d(0,{}px,0) scale(0.96);",
            20.0 * direction
        );
    }

    let direction = if is_top { 1.0 } else { -1.0 };
    let margin_top = if stack_index == 0 { 0.0 } else { -60.0 };
    let (shift_x, shift_y, scale) = match preview_rank {
        0 => (0.0, 0.0, 1.0),
        1 => (4.0, 7.0 * direction, 0.992),
        2 => (8.0, 13.0 * direction, 0.984),
        3 => (12.0, 18.0 * direction, 0.976),
        _ => (16.0, 22.0 * direction, 0.968),
    };

    format!(
        "margin-top:{margin_top}px; max-height:12rem; transform: translate3d({shift_x}px,{shift_y}px,0) scale({scale});"
    )
}

#[component]
pub fn Toaster(#[prop(optional)] position: Option<ToasterPosition>) -> impl IntoView {
    let toasts = expect_context::<Toasts>();
    let is_hovered = RwSignal::new(false);
    let position = position.unwrap_or_default();
    let is_top = matches!(
        position,
        ToasterPosition::TopLeft | ToasterPosition::TopRight
    );
    let (container_class, stack_class) = match position {
        ToasterPosition::TopLeft => ("top-4 left-4 items-start", "items-start"),
        ToasterPosition::TopRight => ("top-4 right-4 items-end", "items-end"),
        ToasterPosition::BottomLeft => ("bottom-4 left-4 items-start", "items-start"),
        ToasterPosition::BottomCenter => ("inset-x-0 bottom-4 items-center", "items-center"),
        ToasterPosition::BottomRight => ("bottom-4 right-4 items-end", "items-end"),
    };

    view! {
        <Portal>
            <style>
                "
                @keyframes toast-slide-in {
                    0% {
                        opacity: 0;
                        transform: translate3d(0, 14px, 0) scale(0.98);
                    }
                    100% {
                        opacity: 1;
                        transform: translate3d(0, 0, 0) scale(1);
                    }
                }

                @keyframes toast-slide-out {
                    0% {
                        opacity: 1;
                        transform: translate3d(0, 0, 0) scale(1);
                    }
                    100% {
                        opacity: 0;
                        transform: translate3d(0, 12px, 0) scale(0.98);
                    }
                }
                "
            </style>
            <div class=format!(
                "pointer-events-none fixed z-60 flex max-w-full flex-col px-4 {}",
                container_class,
            )>
                <div
                    class=format!("pointer-events-auto flex w-full max-w-sm flex-col {}", stack_class)
                    aria-live="polite"
                    on:mouseenter=move |_| is_hovered.set(true)
                    on:mouseleave=move |_| is_hovered.set(false)
                >
                    <For
                        each=move || {
                            let entries = toasts.entries.get();
                            let ordered = if is_top {
                                entries.into_iter().rev().collect::<Vec<_>>()
                            } else {
                                entries
                            };
                            let total = ordered.len();

                            ordered
                                .into_iter()
                                .enumerate()
                                .map(|(index, toast)| {
                                    let preview_rank = if is_top { index } else { total - index - 1 };
                                    (toast, index, preview_rank)
                                })
                                .collect::<Vec<_>>()
                        }
                        key=|item| item.0.id
                        children=move |(toast, stack_index, preview_rank)| {
                            let toast_id = toast.id;
                            let exit_signal = toast.is_exiting;
                            let origin_class = if is_top { "origin-top" } else { "origin-bottom" };
                            view! {
                                <div
                                    class=move || {
                                        let expanded_class = if is_hovered.get() {
                                            "z-0 opacity-100"
                                        } else {
                                            collapsed_stack_item_class(preview_rank)
                                        };

                                        format!(
                                            "relative {} transition-[margin-top,max-height,opacity,transform] duration-200
                                            ease-[cubic-bezier(0.22,1,0.36,1)] {}",
                                            origin_class,
                                            expanded_class,
                                        )
                                    }
                                    style=move || {
                                        stack_item_style(stack_index, preview_rank, is_hovered.get(), is_top)
                                    }
                                >
                                    <ToastView
                                        message=toast.message
                                        variant=toast.variant
                                        is_exiting=exit_signal
                                        on_close=Callback::new(move |_| toasts.dismiss(toast_id))
                                    />
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </Portal>
    }
}

pub fn show_toast(toaster: Toasts, message: impl Into<String>, variant: ToastVariant) {
    toaster.show(message, variant);
}
