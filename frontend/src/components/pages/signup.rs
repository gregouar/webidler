use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::http::client::SignUpRequest;

use crate::components::{
    backend_client::BackendClient,
    captcha::Captcha,
    pages::{privacy::PrivacyContent, terms::TermsContent},
    ui::{
        buttons::MenuButton,
        input::{Input, ValidatedInput},
        toast::*,
    },
};

#[component]
pub fn SignUpPage() -> impl IntoView {
    let navigate_to_menu = {
        let navigate = use_navigate();
        move |_| {
            navigate("/", Default::default());
        }
    };

    let (get_username_storage, set_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("username");

    let username = RwSignal::new(get_username_storage.get_untracked());
    let email = RwSignal::new(Some(None));
    let password = RwSignal::new(None);
    let confirm_password = RwSignal::new(None);
    let accepted_terms = RwSignal::new(false);
    let captcha_token = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let passwords_mismatch = Signal::derive(move || password.get() != confirm_password.get());
    let disable_submit = Signal::derive(move || {
        username.read().is_none()
            || password.read().is_none()
            || passwords_mismatch.get()
            || email.read().is_none()
            || !accepted_terms.get()
            || captcha_token.read().is_none()
            || processing.get()
    });

    let on_submit = {
        let toaster = expect_context::<Toasts>();
        let backend = expect_context::<BackendClient>();
        let navigate = use_navigate();
        move |_| {
            if disable_submit.get() {
                return;
            }

            processing.set(true);
            let navigate = navigate.clone();
            spawn_local({
                async move {
                    match backend
                        .post_signup(&SignUpRequest {
                            captcha_token: captcha_token.get().unwrap_or_default(),
                            username: username.get().unwrap(),
                            email: email.get().unwrap(),
                            password: password.get().unwrap(),
                            accepted_terms: accepted_terms.get(),
                        })
                        .await
                    {
                        Ok(_) => {
                            // Or directly signin and go to user dashboard?
                            // set_jwt_storage.set(response.jwt);
                            set_username_storage.set(username.get());
                            navigate("/", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Registration error: {e}"),
                                ToastVariant::Error,
                            );
                            processing.set(false);
                        }
                    }
                }
            });
        }
    };

    let show_terms = RwSignal::new(false);
    let show_privacy = RwSignal::new(false);

    view! {
        <main class="my-0 mx-auto max-w-2xl text-center flex flex-col justify-center p-6">
            <h1 class="text-amber-200 text-4xl font-extrabold mb-6">"Create Account"</h1>

            <div class="space-y-4 text-left text-white">
                <ValidatedInput
                    label="Username"
                    id="username"
                    input_type="text"
                    placeholder="Enter your username"
                    bind=username
                />

                <ValidatedInput
                    label="Email recovery"
                    id="email"
                    input_type="text"
                    placeholder="Optionally enter your email for password recovery"
                    bind=email
                />

                <ValidatedInput
                    label="Password"
                    id="password"
                    input_type="password"
                    placeholder="Enter your password"
                    bind=password
                />

                <Input
                    id="confirm-password"
                    input_type="password"
                    placeholder="Confirm your password"
                    bind=confirm_password
                    invalid=passwords_mismatch
                />

                <div class="flex items-start mt-4">
                    <input
                        id="terms"
                        type="checkbox"
                        class="mt-1 mr-2"
                        prop:checked=accepted_terms
                        on:input=move |ev| {
                            accepted_terms.set(event_target_checked(&ev));
                        }
                    />
                    <label for="terms" class="text-sm text-gray-400">
                        "I agree to the "
                        <button
                            type="button"
                            class="btn text-amber-400 underline hover:text-amber-300"
                            on:click=move |_| show_terms.set(true)
                        >
                            "Terms & Conditions"
                        </button>
                        " and "
                        <button
                            type="button"
                            class="btn text-amber-400 underline hover:text-amber-300"
                            on:click=move |_| show_privacy.set(true)
                        >
                            "Privacy Policy"
                        </button>
                        "."
                    </label>
                </div>

                <Captcha token=captcha_token class:justify-self-center />

                <MenuButton class:w-full on:click=on_submit disabled=disable_submit>
                    "Confirm"
                </MenuButton>
            </div>

            <p class="mt-6 text-xs text-gray-400 text-left">
                "By signing up, you consent to the storage and processing of your data in accordance with GDPR. You can request data deletion at any time via the account page."
            </p>
            <div>
                <MenuButton on:click=navigate_to_menu>"Back"</MenuButton>
            </div>

            <Show when=move || show_terms.get()>
                <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                    <div class="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl max-w-2xl w-full max-h-[80vh] flex flex-col">

                        <div class="flex items-center justify-between p-4 border-b border-zinc-700">
                            <h2 class="text-xl font-bold text-amber-200">"Terms and Conditions"</h2>
                            <button
                                class="btn text-gray-400 hover:text-white text-xl"
                                on:click=move |_| show_terms.set(false)
                            >
                                "✕"
                            </button>
                        </div>

                        <div class="p-6 overflow-y-auto">
                            <TermsContent />
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_privacy.get()>
                <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                    <div class="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl max-w-2xl w-full max-h-[80vh] flex flex-col">

                        <div class="flex items-center justify-between p-4 border-b border-zinc-700">
                            <h2 class="text-xl font-bold text-amber-200">"Privacy Notice"</h2>
                            <button
                                class="btn text-gray-400 hover:text-white text-xl"
                                on:click=move |_| show_privacy.set(false)
                            >
                                "✕"
                            </button>
                        </div>

                        <div class="p-6 overflow-y-auto">
                            <PrivacyContent />
                        </div>
                    </div>
                </div>
            </Show>
        </main>
    }
}
