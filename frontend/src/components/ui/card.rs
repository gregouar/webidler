use leptos::{html::*, prelude::*};

use crate::components::ui::buttons::CloseButton;

#[component]
pub fn Card(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = true)] gap: bool,
    #[prop(default = true)] pad: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "max-h-full flex flex-col
            bg-zinc-800 rounded-md shadow-xl/30
            relative
            border-1 
            border-b-zinc-950 border-r-zinc-900
            border-t-zinc-600 border-l-zinc-700
            {} {} {}",
            class.unwrap_or_default(),
            if gap { "gap-1 xl:gap-2" } else { "" },
            if pad { "p-1 xl:p-3" } else { "" },
        )>
            // <div class="
            // absolute inset-0 rounded-md pointer-events-none
            // ring-1 ring-white/10
            // translate-x-[0.5px] translate-y-[0.5px]
            // "></div>

            // <div class="
            // absolute inset-0 rounded-md pointer-events-none
            // ring-1 ring-black/40
            // -translate-x-[0.5px] -translate-y-[0.5px]
            // "></div>

            {children()}
        </div>
    }
}

#[component]
pub fn CardTitle(children: Children) -> impl IntoView {
    view! {
        <span class="
        text-shadow-lg/30 shadow-gray-950 text-amber-200 font-semibold
        text-base xl:text-xl 
        ">{children()}</span>
    }
}

#[component]
pub fn CardHeader(
    title: &'static str,
    on_close: impl Fn() + Send + Sync + 'static,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="px-4 relative z-10 flex items-center justify-between">
            <CardTitle>{title}</CardTitle>
            {children.map(|children| children())}
            <CloseButton on:click=move |_| on_close() />
        </div>
    }
}

#[component]
pub fn CardInset(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = true)] gap: bool,
    #[prop(default = true)] pad: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "flex flex-col
            bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]
            ring-1  ring-zinc-950
            overflow-y-auto
            {} {} {}",
            class.unwrap_or_default(),
            if gap { "gap-1 xl:gap-2" } else { "" },
            if pad { "p-1 xl:p-3" } else { "" },
        )>{children()}</div>
    }
}
