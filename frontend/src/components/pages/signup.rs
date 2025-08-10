use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use shared::http::client::{Email, Name, Password, SignUpRequest};

use crate::components::{
    backend_client::BackendClient,
    captcha::Captcha,
    ui::{buttons::MenuButton, input::ValidatedInput, toast::*},
};

#[component]
pub fn SignUpPage() -> impl IntoView {
    let navigate_to_menu = {
        let navigate = use_navigate();
        move |_| {
            navigate("/", Default::default());
        }
    };

    let username = RwSignal::new(None);
    let email = RwSignal::new(None);
    let password = RwSignal::new(None);
    let confirm_password = RwSignal::new(String::new());
    let accepted_terms = RwSignal::new(false);
    let captcha_token = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let passwords_match = Signal::derive(move || {
        password
            .get()
            .map(|x: Password| x.into_inner())
            .unwrap_or_default()
            == confirm_password.get()
    });
    let can_submit = Signal::derive(move || {
        !username.read().is_none()
            && !password.read().is_none()
            && passwords_match.get()
            && match email.get() {
                Some(Ok(_)) | None => true,
                Some(Err(_)) => false,
            }
            && accepted_terms.get()
            && captcha_token.read().is_some()
            && !processing.get()
    });

    let on_submit = {
        let toaster = expect_context::<Toasts>();
        let backend = use_context::<BackendClient>().unwrap();
        let navigate = use_navigate();
        move |_| {
            if !can_submit.get() {
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
                            email: email.get().map(Result::unwrap),
                            password: password.get().unwrap(),
                            accepted_terms: accepted_terms.get(),
                        })
                        .await
                    {
                        Ok(_) => {
                            // Or directly signin and go to user dashboard?
                            // set_jwt_storage.set(response.jwt);
                            navigate("/", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Registration error: {e:?}"),
                                ToastVariant::Error,
                            );
                            processing.set(false);
                        }
                    }
                }
            });
        }
    };

    view! {
        <main class="my-0 mx-auto max-w-2xl text-center flex flex-col justify-center p-6">
            <h1 class="text-amber-200 text-4xl font-extrabold mb-6">"Create Account"</h1>

            <div class="space-y-4 text-left text-white">
                <div>
                    <label for="username" class="block mb-1 text-sm font-medium text-gray-300">
                        "Username"
                    </label>
                    <ValidatedInput
                        id="username"
                        input_type="text"
                        placeholder="Enter your username"
                        bind=username
                    />

                // <input
                // id="username"
                // type="text"
                // class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                // placeholder="Enter your username"
                // on:input:target=move |ev| {
                // username.set(Name::try_new(ev.target().value()).ok())
                // }
                // />
                </div>

                <div>
                    <label for="email" class="block mb-1 text-sm font-medium text-gray-300">
                        "Email recovery"
                    </label>
                    <input
                        id="email"
                        type="email"
                        class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        placeholder="Optionally enter your email for password recovery"
                        on:input:target=move |ev| {
                            email.set(Some(Email::try_new(ev.target().value())))
                        }
                    />
                </div>

                <div>
                    <label for="password" class="block mb-1 text-sm font-medium text-gray-300">
                        "Password"
                    </label>
                    <input
                        id="password"
                        type="password"
                        class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        placeholder="Enter your password"
                        on:input:target=move |ev| {
                            password.set(Password::try_new(ev.target().value()).ok())
                        }
                    />
                </div>

                <div>
                    <input
                        id="confirm-password"
                        type="password"
                        class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        placeholder="Confirm your password"
                        bind:value=confirm_password
                    />
                    <Show when=move || !passwords_match.get() && !confirm_password.get().is_empty()>
                        <p class="text-red-400 text-sm mt-1">"Passwords do not match."</p>
                    </Show>
                </div>

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
                    <label for="terms" class="text-sm text-gray-300">
                        "I agree to the "
                        <a
                            href="/terms"
                            class="text-amber-300 underline hover:text-amber-200"
                            target="_blank"
                        >
                            "Terms and Conditions"
                        </a>
                        " and "
                        <a
                            href="/privacy"
                            class="text-amber-300 underline hover:text-amber-200"
                            target="_blank"
                        >
                            "Privacy Policy"
                        </a>
                        "."
                    </label>
                </div>

                <Captcha token=captcha_token class:justify-self-center />

                <MenuButton
                    class:w-full
                    on:click=on_submit
                    disabled=Signal::derive(move || !can_submit.get())
                >
                    "Confirm"
                </MenuButton>
            </div>

            <p class="mt-6 text-xs text-gray-400 text-left">
                "By signing up, you consent to the storage and processing of your data in accordance with GDPR. You can request data deletion at any time via the account page."
            </p>
            <div>
                <MenuButton on:click=navigate_to_menu>"Back"</MenuButton>
            </div>
        </main>
    }
}
