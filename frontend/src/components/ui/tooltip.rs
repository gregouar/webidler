use std::sync::Arc;

use leptos::{ev, html::Div, portal::Portal, prelude::*, web_sys};
use leptos_use::{use_mouse, use_window_size};

#[derive(Clone, Debug, Copy)]
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
    AutoLeft,
}

#[component]
pub fn DynamicTooltip() -> impl IntoView {
    let tooltip_context = DynamicTooltipContext {
        content: RwSignal::new(None),
        position: RwSignal::new(DynamicTooltipPosition::BottomRight),
    };
    provide_context(tooltip_context);

    let show_tooltip = { move || tooltip_context.content.read().is_some() };

    let tooltip_size = RwSignal::new((0.0, 0.0));
    let tooltip_ref: NodeRef<Div> = NodeRef::new();

    Effect::new(move |_| {
        let _ = tooltip_context.content.read();
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
            DynamicTooltipPosition::AutoLeft => {
                if mouse_y < window.height.get() / 2.0 {
                    DynamicTooltipPosition::BottomLeft
                } else {
                    DynamicTooltipPosition::TopLeft
                }
            }
            x => x,
        };

        let window_height = web_sys::window()
            .unwrap()
            .inner_height()
            .unwrap()
            .as_f64()
            .unwrap();

        let window_width = web_sys::window()
            .unwrap()
            .inner_width()
            .unwrap()
            .as_f64()
            .unwrap();

        let (left, top) = match position {
            DynamicTooltipPosition::BottomLeft => (mouse_x - width, mouse_y),
            DynamicTooltipPosition::BottomRight => (mouse_x, mouse_y),
            DynamicTooltipPosition::TopLeft => (mouse_x - width, mouse_y - height),
            DynamicTooltipPosition::TopRight => (mouse_x, mouse_y - height),
            DynamicTooltipPosition::AutoLeft => (mouse_x - width, 0.0),
            DynamicTooltipPosition::Auto => (0.0, 0.0),
        };

        let left = left.clamp(0.0, (window_width - width).max(0.0));
        let top = top.clamp(0.0, (window_height - height).max(0.0));

        format!("transform: translate3d({left}px, {top}px, 0);")
    };

    let handle = window_event_listener(ev::touchend, {
        move |ev| {
            if ev.touches().length() == 0 {
                tooltip_context.content.set(None)
            }
        }
    });

    on_cleanup(move || handle.remove());

    let handle = window_event_listener(ev::touchcancel, {
        move |ev| {
            if ev.touches().length() == 0 {
                tooltip_context.content.set(None)
            }
        }
    });

    on_cleanup(move || handle.remove());

    view! {
        <Show when=show_tooltip>
            {move || {
                view! {
                    <div
                        class="fixed z-60 pointer-events-none transition-opacity duration-150 p-2 will-change-transform"
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
    F: Fn() -> IV + Send + Sync + Clone + 'static,
    IV: IntoView + 'static,
{
    let is_open = RwSignal::new(false);

    let container_ref = NodeRef::<Div>::new();
    let tooltip_ref = NodeRef::<Div>::new();
    let _ = leptos_use::on_click_outside(container_ref, move |_| is_open.set(false));

    let tooltip_pos = RwSignal::new(Default::default());

    Effect::new(move |_| {
        if !is_open.get() {
            return;
        }

        let Some(trigger) = container_ref.get() else {
            return;
        };
        let Some(tip) = tooltip_ref.get() else {
            return;
        };

        let rect = trigger.get_bounding_client_rect();
        let tip_rect = tip.get_bounding_client_rect();

        let (x, y) = match position {
            StaticTooltipPosition::Top => (rect.left() + rect.width() / 2.0, rect.top()),
            StaticTooltipPosition::Bottom => (rect.left() + rect.width() / 2.0, rect.bottom()),
            StaticTooltipPosition::Left => (rect.left(), rect.top() + rect.height() / 2.0),
            StaticTooltipPosition::Right => (rect.right(), rect.top() + rect.height() / 2.0),
        };

        // Viewport size
        let window = web_sys::window().unwrap();
        let vw = window.inner_width().unwrap().as_f64().unwrap();
        let vh = window.inner_height().unwrap().as_f64().unwrap();

        // Compute actual rendered box depending on transform
        let (mut left, mut top) = match position {
            StaticTooltipPosition::Top => (x - tip_rect.width() / 2.0, y - tip_rect.height()),
            StaticTooltipPosition::Bottom => (x - tip_rect.width() / 2.0, y),
            StaticTooltipPosition::Left => (x - tip_rect.width(), y - tip_rect.height() / 2.0),
            StaticTooltipPosition::Right => (x, y - tip_rect.height() / 2.0),
        };

        // Clamp inside viewport with 8px padding
        let padding = 8.0;

        left = left.clamp(padding, vw - tip_rect.width() - padding);
        top = top.clamp(padding, vh - tip_rect.height() - padding);

        tooltip_pos.set((left, top));
    });

    // Effect::new(move |_| {
    //     if is_open.get() {
    //         if let Some(el) = container_ref.get() {
    //             let rect: DomRect = el.get_bounding_client_rect();

    //             tooltip_pos.set(match position {
    //                 StaticTooltipPosition::Top => (rect.left() + rect.width() / 2.0, rect.top()),
    //                 StaticTooltipPosition::Bottom => {
    //                     (rect.left() + rect.width() / 2.0, rect.bottom())
    //                 }
    //                 StaticTooltipPosition::Left => (rect.left(), rect.top() + rect.height() / 2.0),
    //                 StaticTooltipPosition::Right => {
    //                     (rect.right(), rect.top() + rect.height() / 2.0)
    //                 }
    //             });
    //         }
    //     }
    // });

    // let transform = match position {
    //     StaticTooltipPosition::Top => "translate(-50%, -100%)",
    //     StaticTooltipPosition::Bottom => "translate(-50%, 0)",
    //     StaticTooltipPosition::Left => "translate(-100%, -50%)",
    //     StaticTooltipPosition::Right => "translate(0, -50%)",
    // };

    let handle = window_event_listener(ev::touchend, {
        move |ev| {
            if ev.touches().length() == 0 {
                is_open.set(false)
            }
        }
    });

    on_cleanup(move || handle.remove());

    view! {
        <div
            class="inline-block"
            on:touchstart=move |_| is_open.set(true)
            on:mouseenter=move |_| is_open.set(true)
            on:mouseleave=move |_| is_open.set(false)
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }
            node_ref=container_ref
        >
            {children()}
        </div>

        <Show when=move || {
            is_open.get()
        }>
            {
                let tooltip = tooltip.clone();
                view! {
                    <Portal>
                        <div
                            node_ref=tooltip_ref
                            class="p-2  fixed z-50"
                            style=move || {
                                let (x, y) = tooltip_pos.get();
                                format!("position: fixed; left:{}px; top:{}px; ", x, y)
                            }
                        >
                            <div class="
                            px-2 py-1 xl:px-3 xl:py-1
                            text-xs xl:text-sm text-white font-normal
                            bg-zinc-900 border border-neutral-200
                            rounded shadow-lg/30 whitespace-nowrap
                            select-none text-center
                            ">
                                {
                                    let tooltip = tooltip.clone();
                                    move || tooltip()
                                }
                            </div>
                        </div>
                    </Portal>
                }
            }
        </Show>
    }
}
