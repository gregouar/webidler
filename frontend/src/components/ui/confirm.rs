use leptos::prelude::*;
use leptos_use::on_click_outside;
use std::sync::Arc;

#[derive(Clone)]
pub struct ConfirmContext {
    pub confirm: Arc<dyn Fn(String, Arc<dyn Fn() + Send + Sync>) + Send + Sync>,
}

pub fn provide_confirm_context() -> RwSignal<Option<(String, Arc<dyn Fn() + Send + Sync>)>> {
    let state = RwSignal::new(None);
    let context = ConfirmContext {
        confirm: Arc::new({
            let state = state.clone();
            move |message: String, on_confirm: Arc<dyn Fn() + Send + Sync>| {
                state.set(Some((message, on_confirm)));
            }
        }),
    };
    provide_context(context);
    state
}

#[component]
pub fn ConfirmationModal(
    state: RwSignal<Option<(String, Arc<dyn Fn() + Send + Sync>)>>,
) -> impl IntoView {
    let node_ref = NodeRef::new();

    let _ = on_click_outside(node_ref, move |_| {
        state.set(None);
    });

    view! {
        <Show when=move || state.read().is_some()>
            <style>
                "
                @keyframes modal-fade {
                    from { opacity: 0; transform: scale(0.95); }
                    to { opacity: 1; transform: scale(1); }
                }
                
                .confirm-btn {
                    @apply py-2 px-4 text-lg font-semibold rounded-md transition
                            focus:outline-none focus:ring-2 focus:ring-offset-2;
                }
                
                .confirm-btn:active {
                    transform: scale(0.95);
                }
                "
            </style>

            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                <div
                    node_ref=node_ref
                    class="bg-gradient-to-br from-gray-800/90 via-gray-900/90 to-black
                    border border-gray-700 ring-2 ring-gray-700
                    text-white text-center shadow-2xl rounded-xl p-6 w-80 animate-[modal-fade_0.2s_ease-out]"
                >
                    <p class="text-lg mb-6 leading-snug px-2">
                        {move || {
                            state.read().as_ref().map(|(msg, _)| msg.clone()).unwrap_or_default()
                        }}
                    </p>

                    <div class="flex justify-center gap-4">
                        <button
                            class="confirm-btn text-green-300 hover:text-green-100 hover:bg-green-800/40 p-2"
                            on:click=move |_| {
                                if let Some((_, cb)) = state.get() {
                                    state.set(None);
                                    cb();
                                }
                            }
                        >
                            "Confirm"
                        </button>

                        <button
                            class="confirm-btn text-amber-300 hover:text-amber-100 hover:bg-amber-800/40 p-2"
                            on:click=move |_| state.set(None)
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
