use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{window, HtmlInputElement};
use leptos::{html::*, prelude::*};

use crate::components::ui::buttons::MenuButton;

#[component]
pub fn SignUpPage() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let accepted_terms = RwSignal::new(false);

    let passwords_match = Signal::derive(move || password.get() == confirm_password.get());
    let can_submit = Signal::derive(move || {
        !username.get().is_empty()
            && !password.get().is_empty()
            && passwords_match.get()
            && accepted_terms.get()
    });

    let on_submit = move |_| {
        if !can_submit.get() {
            return;
        }

        if let Some(token_input) = window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector("input[name='cf-turnstile-response']")
            .unwrap()
            .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
        {
            let token = token_input.value();
            // TODO: query backend
        } else {
            // TODO: toast error
        }
    };

    view! {
        <main class="my-0 mx-auto max-w-2xl text-center flex flex-col justify-center p-6">
            <h1 class="text-amber-200 text-4xl font-extrabold mb-6">"Sign Up"</h1>

            <div class="space-y-4 text-left text-white">
                <div>
                    <label for="username" class="block mb-1 text-sm font-medium text-gray-300">
                        "Username"
                    </label>
                    <input
                        id="username"
                        type="text"
                        class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        placeholder="Enter your username"
                        bind:value=username
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
                        bind:value=password
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

                <div
                    class="cf-turnstile"
                    data-sitekey="0x4AAAAAABoSog3mlP1Ok1U9"
                    data-theme="dark"
                ></div>

                <MenuButton on:click=on_submit disabled=Signal::derive(move || !can_submit.get())>
                    "Sign Up"
                </MenuButton>
            </div>

            <p class="mt-6 text-xs text-gray-400 text-left">
                "By signing up, you consent to the storage and processing of your data in accordance with GDPR. You can request data deletion at any time via the account page."
            </p>
        </main>
    }
}
