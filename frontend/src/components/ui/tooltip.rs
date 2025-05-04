use leptos::html::*;
use leptos::prelude::*;
use leptos_use::use_mouse;

#[derive(Clone, Debug)]
pub struct TooltipContext {
    pub content: RwSignal<Option<ChildrenFn>>,
    pub position: RwSignal<TooltipPosition>,
}

#[derive(Debug, Clone, Copy)]
pub enum TooltipPosition {
    BottomRight,
}

// TODO: position
#[component]
pub fn DynamicTooltip() -> impl IntoView {
    let tooltip_context = TooltipContext {
        content: RwSignal::new(None),
        position: RwSignal::new(TooltipPosition::BottomRight),
    };
    provide_context(tooltip_context.clone());

    let show_tooltip = Signal::derive({
        let tooltip_context = tooltip_context.clone();
        move || tooltip_context.content.read().is_some()
    });

    view! {
        <DynamicTooltipContent show=show_tooltip>
            {move || {
                tooltip_context
                    .content
                    .get()
                    .map(|x| {
                        view! { {x()} }
                    })
            }}
        </DynamicTooltipContent>
    }
}

// TODO: position
#[component]
fn DynamicTooltipContent(#[prop(into)] show: Signal<bool>, children: ChildrenFn) -> impl IntoView {
    let mouse = use_mouse();

    // See: https://book.leptos.dev/interlude_projecting_children.html
    let children = StoredValue::new(children);

    view! {
        <Show when=move || {
            show.get()
        }>
            {move || {
                view! {
                    <div
                        class="fixed z-50 pointer-events-none transition-opacity duration-150"
                        style=format!(
                            "top: {}px; left: {}px;",
                            mouse.y.get() + 16.0,
                            mouse.x.get() + 16.0,
                        )
                    >
                        {children.read_value()()}
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
