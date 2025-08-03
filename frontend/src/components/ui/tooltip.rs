use std::sync::Arc;

use leptos::{html::*, prelude::*};
use leptos_use::{use_mouse, use_window_size};

#[derive(Clone, Debug)]
pub struct DynamicTooltipContext {
    content: RwSignal<Option<ChildrenFn>>,
    position: RwSignal<DynamicTooltipPosition>,
}

impl DynamicTooltipContext {
    pub fn set_content(
        &self,
        content: impl Fn() -> AnyView + Send + Sync + 'static,
        position: DynamicTooltipPosition,
    ) {
        self.content.set(Some(Arc::new(content)));
        self.position.set(position);
    }

    pub fn hide(&self) {
        self.content.set(None);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DynamicTooltipPosition {
    Auto,
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

#[component]
pub fn DynamicTooltip() -> impl IntoView {
    let tooltip_context = DynamicTooltipContext {
        content: RwSignal::new(None),
        position: RwSignal::new(DynamicTooltipPosition::BottomRight),
    };
    provide_context(tooltip_context.clone());

    let show_tooltip = {
        let tooltip_context = tooltip_context.clone();
        move || tooltip_context.content.read().is_some()
    };

    let tooltip_size = RwSignal::new((0.0, 0.0));
    let tooltip_ref: NodeRef<Div> = NodeRef::new();

    Effect::new(move |_| {
        if let Some(el) = tooltip_ref.get() {
            let rect = el.get_bounding_client_rect();
            tooltip_size.set((rect.width(), rect.height()));
        }
    });

    let mouse = use_mouse();
    let window = use_window_size();

    let style = move || {
        let mouse_x = mouse.x.get();
        let mouse_y = mouse.y.get();
        let (width, height) = tooltip_size.get();

        let position = match tooltip_context.position.get() {
            DynamicTooltipPosition::Auto => {
                match (
                    mouse_x < window.width.get() / 2.0,
                    mouse_y < window.height.get() / 2.0,
                ) {
                    (true, true) => DynamicTooltipPosition::BottomRight,
                    (false, true) => DynamicTooltipPosition::BottomLeft,
                    (true, false) => DynamicTooltipPosition::TopRight,
                    (false, false) => DynamicTooltipPosition::TopLeft,
                }
            }
            x => x,
        };

        let (left, top) = match position {
            DynamicTooltipPosition::BottomLeft => (mouse_x - width, mouse_y),
            DynamicTooltipPosition::BottomRight => (mouse_x, mouse_y),
            DynamicTooltipPosition::TopLeft => (mouse_x - width, mouse_y - height),
            DynamicTooltipPosition::TopRight => (mouse_x, mouse_y - height),
            _ => (0.0, 0.0),
        };

        format!("top: {top}px; left: {left}px;")
    };

    view! {
        <Show when=show_tooltip>
            {move || {
                view! {
                    <div
                        class="fixed z-50 pointer-events-none transition-opacity duration-150 p-2 {}"

                        node_ref=tooltip_ref
                        style=style
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

#[derive(Debug, Clone, Copy)]
pub enum StaticTooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
}

// #[component]
// pub fn StaticTooltip(
//     tooltip: Signal<String>,
//     position: StaticTooltipPosition,
//     children: Children,
// ) -> impl IntoView {
//     let position_classes = match position {
//         StaticTooltipPosition::Top => "bottom-full left-1/2 -translate-x-1/2 mb-2",
//         StaticTooltipPosition::Bottom => "top-full left-1/2 -translate-x-1/2 mt-2",
//         StaticTooltipPosition::Left => "right-full top-1/2 -translate-y-1/2 mr-2",
//         StaticTooltipPosition::Right => "left-full top-1/2 -translate-y-1/2 ml-2",
//     };

//     view! {
//         <div class="relative group inline-block">
//             {children()}
//             <div class=format!(
//                 "absolute hidden group-hover:block px-3 py-1 text-sm text-white \
//              bg-zinc-800 border border-neutral-900 rounded shadow-lg \
//              whitespace-nowrap  whitespace-pre-line z-50 {}",
//                 position_classes,
//             )>{tooltip}</div>
//         </div>
//     }
// }

#[component]
pub fn StaticTooltip<F, IV>(
    children: Children,
    position: StaticTooltipPosition,
    tooltip: F,
) -> impl IntoView
where
    F: Fn() -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    let position_classes = match position {
        StaticTooltipPosition::Top => "bottom-full left-1/2 -translate-x-1/2 mb-2",
        StaticTooltipPosition::Bottom => "top-full left-1/2 -translate-x-1/2 mt-2",
        StaticTooltipPosition::Left => "right-full top-1/2 -translate-y-1/2 mr-2",
        StaticTooltipPosition::Right => "left-full top-1/2 -translate-y-1/2 ml-2",
    };
    view! {
        <div class="relative group inline-block">
            {children()}
            <div class=format!(
                "
                absolute hidden group-hover:block
                px-3 py-1 text-sm text-white
                bg-zinc-800 border border-neutral-900
                rounded shadow-lg whitespace-nowrap z-50
                select-none
                {}",
                position_classes,
            )>{move || tooltip()}</div>
        </div>
    }
}
