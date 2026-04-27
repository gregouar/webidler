use leptos::prelude::*;
use leptos_use::on_click_outside;
use std::sync::Arc;

use crate::components::ui::{
    buttons::{MenuButton, MenuButtonRed},
    card::CardTitle,
};

type ArcFn = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct ConfirmContext {
    pub confirm: Arc<dyn Fn(String, ArcFn) + Send + Sync>,
}

pub fn provide_confirm_context() -> RwSignal<Option<(String, ArcFn)>> {
    let state = RwSignal::new(None);
    let context = ConfirmContext {
        confirm: Arc::new({
            move |message: String, on_confirm: ArcFn| {
                state.set(Some((message, on_confirm)));
            }
        }),
    };
    provide_context(context);
    state
}

#[component]
pub fn ConfirmationModal(state: RwSignal<Option<(String, ArcFn)>>) -> impl IntoView {
    let node_ref = NodeRef::new();

    let _ = on_click_outside(node_ref, move |_| {
        state.set(None);
    });

    view! {
        <Show when=move || state.read().is_some()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-4">
                <div
                    node_ref=node_ref
                    class="relative isolate clip-octagon w-full max-w-md overflow-clip
                    border border-[#6c5734]/45
                    shadow-[0_18px_42px_rgba(0,0,0,0.5),inset_2px_2px_1px_rgba(255,255,255,0.06),inset_-2px_-2px_1px_rgba(0,0,0,0.15)]
                    animate-[modal-fade_0.18s_ease-out]"
                    style="
                    background-image:
                    linear-gradient(180deg, rgba(214,177,102,0.05), rgba(0,0,0,0)),
                    linear-gradient(135deg, rgba(39,38,44,0.96), rgba(15,15,18,1));
                    background-blend-mode: screen, normal;"
                >
                    <style>
                        "
                        @keyframes modal-fade {
                            from { opacity: 0; transform: scale(0.96) translateY(6px); }
                            to { opacity: 1; transform: scale(1) translateY(0); }
                        }
                        "
                    </style>
                    <div class="pointer-events-none clip-octagon [--cut:11px] absolute inset-[1px] border border-white/6"></div>
                    <div class="pointer-events-none absolute inset-x-5 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></div>
                    <div class="relative z-10 flex flex-col gap-4 p-5 xl:p-6 text-center">
                        <CardTitle>"Confirm Action"</CardTitle>

                        <p class="px-2 text-sm xl:text-base text-stone-200 leading-relaxed">
                            {move || {
                                state
                                    .read()
                                    .as_ref()
                                    .map(|(msg, _)| msg.clone())
                                    .unwrap_or_default()
                            }}
                        </p>

                        <div class="flex justify-center gap-3">
                            <MenuButtonRed on:click=move |_| {
                                state.set(None)
                            }>"Cancel"</MenuButtonRed>

                            <MenuButton on:click=move |_| {
                                if let Some((_, cb)) = state.get() {
                                    state.set(None);
                                    cb();
                                }
                            }>"Confirm"</MenuButton>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
