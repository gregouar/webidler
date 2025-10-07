use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::{components::Redirect, hooks::use_navigate};
use leptos_use::storage;

use shared::http::client::{ForgotPasswordRequest, SignInRequest};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    captcha::*,
    shared::player_count::PlayerCount,
    ui::{
        buttons::MenuButton,
        input::{Input, ValidatedInput},
        toast::*,
    },
};

#[component]
pub fn MainMenuPage() -> impl IntoView {
    let auth_context = expect_context::<AuthContext>();
    if !auth_context.token().is_empty() {
        view! { <Redirect path="user-dashboard" /> }.into_any()
    } else {
        view! { <MainMenu /> }.into_any()
    }
}

#[component]
fn MainMenu() -> impl IntoView {
    let (get_username_storage, set_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("username");

    let username = RwSignal::new(get_username_storage.get_untracked());
    let password = RwSignal::new(None);
    let captcha_token = RwSignal::new(None);

    let connecting = RwSignal::new(false);
    let disable_connect = Signal::derive(move || {
        username.read().is_none()
            || password.read().is_none()
            || captcha_token.read().is_none()
            || connecting.get()
    });

    let signin = {
        let toaster = expect_context::<Toasts>();
        let backend = expect_context::<BackendClient>();
        let auth_context = expect_context::<AuthContext>();
        let navigate = use_navigate();
        move || {
            if disable_connect.get() {
                return;
            }

            connecting.set(true);
            let navigate = navigate.clone();
            spawn_local({
                async move {
                    match backend
                        .post_signin(&SignInRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            username: username.get_untracked().unwrap(), // TODO: better error?
                            password: password.get_untracked().unwrap(),
                        })
                        .await
                    {
                        Ok(response) => {
                            auth_context.sign_in(response.jwt);
                            set_username_storage.set(username.get_untracked());
                            navigate("user-dashboard", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Authentication error: {e}"),
                                ToastVariant::Error,
                            );
                            connecting.set(false);
                        }
                    }
                }
            });
        }
    };

    let navigate_to_leaderboard = {
        let navigate = use_navigate();
        move |_| {
            navigate("leaderboard", Default::default());
        }
    };

    let navigate_to_signup = {
        let navigate = use_navigate();
        move |_| {
            navigate("signup", Default::default());
        }
    };
    let password_ref = NodeRef::<leptos::html::Input>::new();

    let show_forgot_password_modal = RwSignal::new(false);

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <PlayerCount />
            <div>
                <h1 class="text-shadow-lg mb-4 text-amber-200 text-4xl  md:text-5xl xl:text-6xl font-extrabold leading-none tracking-tight">
                    "Grind to Rust!"
                </h1>
                <div class="flex flex-col space-y-2">
                    // <form>
                    <div class="w-full mx-auto text-left">
                        <label class="block mb-2 text-sm font-medium text-gray-400">
                            "Sign In:"
                        </label>
                        <Input
                            id="username"
                            input_type="text"
                            placeholder="Enter your username"
                            bind=username
                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" && let Some(pw) = password_ref.get() {
                                    pw.focus().unwrap();
                                }
                            }
                        />
                    </div>
                    <div class="w-full mx-auto mb-6 text-left">
                        <Input
                            node_ref=password_ref
                            id="password"
                            input_type="password"
                            placeholder="Enter your password"
                            bind=password
                            on:keydown={
                                let signin = signin.clone();
                                move |ev| {
                                    if ev.key() == "Enter" {
                                        signin();
                                    }
                                }
                            }
                        />
                    </div>
                    <div class="text-right -mt-4 mb-2">
                        <button
                            class="text-amber-300 text-sm underline hover:text-amber-200"
                            on:click=move |_| show_forgot_password_modal.set(true)
                        >
                            "I forgot my password"
                        </button>
                    </div>
                    <Captcha token=captcha_token />
                    // </form>

                    <MenuButton on:click=move |_| signin() disabled=disable_connect class:mb-4>
                        {move || if connecting.get() { "Connecting..." } else { "Connect" }}
                    </MenuButton>
                    <MenuButton on:click=navigate_to_signup>"Create Account"</MenuButton>
                    <MenuButton on:click=navigate_to_leaderboard>"Leaderboard"</MenuButton>

                    <ForgotPasswordModal open=show_forgot_password_modal />
                </div>
            </div>

            <div class="bg-gray-800 text-gray-200 text-sm p-4 rounded-xl border border-gray-700 shadow-inner space-y-4">
                <div>
                    <h2 class="text-lg font-semibold mb-2">"Disclaimer"</h2>
                    <p>
                        "2D artworks featured in this app are generated using AI tools, with DALL·E 3 (free version via "
                        <a
                            href="https://chatgpt.com"
                            class="text-amber-300 underline hover:text-amber-200"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "chatgpt.com"
                        </a>")."
                    </p>
                </div>

                <div class="flex justify-center gap-6 pt-2 border-t border-zinc-700">
                    <a href="terms" class="text-amber-300 underline hover:text-amber-200">
                        "Terms & Conditions"
                    </a>
                    <a href="privacy" class="text-amber-300 underline hover:text-amber-200">
                        "Privacy Notice"
                    </a>
                </div>
            </div>

        </main>
    }
}

#[component]
pub fn ForgotPasswordModal(open: RwSignal<bool>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();

    let email = RwSignal::new(None);
    let captcha_token = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let success = RwSignal::new(false);

    let on_submit = {
        move |_| {
            if processing.get() {
                return;
            }

            processing.set(true);
            spawn_local({
                let backend = backend.clone();
                async move {
                    // TODO: captcha
                    match backend
                        .post_forgot_password(&ForgotPasswordRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            email: email.get_untracked().unwrap(),
                        })
                        .await
                    {
                        Ok(_) => {
                            success.set(true);
                            show_toast(
                                toaster.clone(),
                                "Password reset instructions sent!",
                                ToastVariant::Success,
                            );
                        }
                        Err(e) => {
                            show_toast(toaster.clone(), format!("Error: {e}"), ToastVariant::Error);
                        }
                    }
                    success.set(true);
                    processing.set(false);
                }
            });
        }
    };

    let disable_submit = Signal::derive(move || {
        email.read().is_none() || captcha_token.read().is_none() || processing.get()
    });

    let on_close = move |_| {
        success.set(false);
        open.set(false);
    };

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
                <div class="bg-zinc-900 ring-1 ring-zinc-700 rounded-lg p-6 w-full max-w-md shadow-xl text-gray-200 space-y-4">
                    <h2 class="text-xl font-bold text-amber-300">"Forgot Password"</h2>

                    <Show when=move || !success.get()>
                        <p class="text-gray-400 text-sm leading-relaxed">
                            "Enter the email associated with your account. We'll send you a link to reset your password."
                        </p>

                        <ValidatedInput
                            label="Email"
                            id="email"
                            input_type="text"
                            placeholder="Enter your email for password recovery"
                            bind=email
                        />
                        <Captcha token=captcha_token />

                        <div class="flex justify-between gap-2 pt-2">
                            <MenuButton on:click=on_close>"Cancel"</MenuButton>
                            <MenuButton on:click=on_submit disabled=disable_submit>
                                {move || {
                                    if processing.get() { "Sending..." } else { "Send Reset Link" }
                                }}
                            </MenuButton>
                        </div>
                    </Show>

                    <Show when=move || success.get()>
                        <div class="text-center space-y-3">
                            <p class="text-gray-400 text-sm">
                                "Check your email and follow the reset link to set a new password."
                            </p>
                            <MenuButton on:click=on_close>"Close"</MenuButton>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
