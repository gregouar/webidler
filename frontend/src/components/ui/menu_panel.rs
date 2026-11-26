use leptos::{ev::KeyboardEvent, html::*, prelude::*};

use crate::components::{
    events::keyboard_event_key, settings::SettingsContext, ui::tooltip::DynamicTooltipContext,
};

#[component]
pub fn MenuPanel(
    open: RwSignal<bool>,
    children: ChildrenFn,
    #[prop(default = true)] w_full: bool,
    #[prop(default = true)] h_full: bool,
    #[prop(default = true)] center: bool,
    #[prop(default = false)] always_mounted: bool,
) -> impl IntoView {
    let panel_ref = NodeRef::<Div>::new();
    let tooltip_context = use_context::<DynamicTooltipContext>();
    let settings = expect_context::<SettingsContext>();

    Effect::new(move |_| {
        if open.try_get().unwrap_or_default()
            && let Some(el) = panel_ref.get_untracked()
        {
            _ = el.focus();
        }
    });

    let handle_key = move |e: KeyboardEvent| {
        if keyboard_event_key(&e).as_deref() == Some("Escape") {
            open.try_set(false);
        }
    };

    let (is_visible, set_is_visible) = signal(false);
    let (is_animating, set_is_animating) = signal(false);
    Effect::new(move |_| {
        if open.try_get().unwrap_or_default() {
            set_is_visible.set(true);
            set_is_animating.set(settings.animations_enabled());
        } else {
            if let Some(tooltip_context) = tooltip_context {
                tooltip_context.clear();
            }
            if !settings.animations_enabled() {
                set_is_animating.set(false);
                set_is_visible.set(open.try_get_untracked().unwrap_or_default());
            } else {
                set_is_animating.set(true);
            }
        }
    });

    let children = StoredValue::new(children);

    view! {
        <style>
            "@keyframes dropDown {
                from { transform: translate3d(0, -50%, 0); }
                to { transform: translate3d(0, 0, 0); }
            }
            @keyframes pullUp {
                from { transform: translate3d(0, 0, 0); }
                to { transform: translate3d(0, -50%, 0); }
            }
            @keyframes fadeIn {
                from { opacity: 0; }
                to { opacity: 1; }
            }
            @keyframes fadeOut {
                from { opacity: 1; }
                to { opacity: 0; }
            }"
        </style>

        <Show when=move || is_visible.get() || always_mounted>
            <div
                class="absolute inset-0 bg-black/70 z-40 flex flex-col p-1 xl:p-4 items-center"
                class:will-change-opacity=move || is_animating.get()
                style=move || {
                    if is_animating.get() && open.try_get().unwrap_or_default() {
                        "animation: fadeIn 0.3s ease-out forwards;"
                    } else if is_animating.get() {
                        "animation: fadeOut 0.3s ease-out forwards;"
                    } else if open.try_get().unwrap_or_default() {
                        "opacity: 1;"
                    } else {
                        "opacity: 0;"
                    }
                }
                on:animationend=move |event| {
                    match event.animation_name().as_str() {
                        "fadeIn" if open.try_get_untracked().unwrap_or_default() => {
                            set_is_animating.set(false);
                        }
                        "fadeOut" if !open.try_get_untracked().unwrap_or_default() => {
                            set_is_animating.set(false);
                            set_is_visible.set(false);
                        }
                        _ => {}
                    }
                }
                on:click=move |_| {
                    open.try_set(false);
                }
                on:keydown=handle_key
                tabindex="0"
                class:hidden=move || !is_visible.get()
            >
                <div
                    class="z-41 w-fit mx-auto max-h-full flex flex-col"
                    class:will-change-transform=move || is_animating.get()
                    class:w-full=w_full
                    class:h-full=h_full
                    class:my-auto=center
                    style=move || {
                        if is_animating.get() && open.try_get().unwrap_or_default() {
                            "animation: dropDown 0.3s ease-out forwards;"
                        } else if is_animating.get() {
                            "animation: pullUp 0.3s ease-out forwards;"
                        } else if open.try_get().unwrap_or_default() {
                            "transform: translate3d(0, 0, 0);"
                        } else {
                            "transform: translate3d(0, -50%, 0);"
                        }
                    }
                    on:click=|e| e.stop_propagation()
                >
                    {children.read_value()()}
                </div>
            </div>
        </Show>
    }
}
