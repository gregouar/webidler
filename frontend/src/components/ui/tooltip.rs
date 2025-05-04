use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;
use leptos_use::use_mouse;

#[derive(Clone, Debug)]
pub struct TooltipContext {
    content: RwSignal<Option<ChildrenFn>>,
    position: RwSignal<TooltipPosition>,
}

impl TooltipContext {
    pub fn set_content(
        &self,
        content: impl Fn() -> AnyView + Send + Sync + 'static,
        position: TooltipPosition,
    ) {
        self.content.set(Some(Arc::new(content)));
        self.position.set(position);
    }

    pub fn hide(&self) {
        self.content.set(None);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TooltipPosition {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

#[component]
pub fn DynamicTooltip() -> impl IntoView {
    let tooltip_context = TooltipContext {
        content: RwSignal::new(None),
        position: RwSignal::new(TooltipPosition::BottomRight),
    };
    provide_context(tooltip_context.clone());

    let show_tooltip = {
        let tooltip_context = tooltip_context.clone();
        move || tooltip_context.content.read().is_some()
    };

    let mouse = use_mouse();

    let origin = move || match tooltip_context.position.get() {
        TooltipPosition::BottomLeft => "transform -translate-x-full",
        TooltipPosition::BottomRight => "",
        TooltipPosition::TopLeft => "transform -translate-y-full -translate-x-full",
        TooltipPosition::TopRight => "transform -translate-y-full",
    };

    view! {
        <Show when=show_tooltip>
            {move || {
                view! {
                    <div
                        class=move || {
                            format!(
                                "fixed z-50 pointer-events-none transition-opacity duration-150 p-2 {}",
                                origin(),
                            )
                        }
                        style=move || {
                            format!("top: {}px; left: {}px;", mouse.y.get(), mouse.x.get())
                        }
                    >
                        {move || {
                            tooltip_context
                                .content
                                .get()
                                .map(|x| {
                                    view! { {x()} }
                                })
                        }}
                    </div>
                }
            }}
        </Show>
    }
}

#[component]
pub fn StaticTooltip(tooltip: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="relative group inline-block">
            {children()}
            <div class="
            absolute bottom-full left-1/2 -translate-x-1/2 mb-2
            hidden group-hover:block
            px-3 py-1
            text-sm text-white
            bg-zinc-800 border border-neutral-900
            rounded shadow-lg
            whitespace-nowrap
            z-50
            ">{tooltip}</div>
        </div>
    }
}
