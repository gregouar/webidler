use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::{components::Redirect, hooks::use_navigate};
use leptos_use::storage;
use rand::{Rng, distr::Alphanumeric};

use shared::{
    http::client::{ForgotPasswordRequest, SignInRequest, SignUpRequest},
    types::{Password, Username},
};

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

    let (get_guest_username_storage, _, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_username");

    let (get_guest_password_storage, _, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_password");

    let captcha_token = RwSignal::new(None);

    let connecting = RwSignal::new(false);
    let disable_connect = Signal::derive(move || {
        username.read().is_none()
            || password.read().is_none()
            || captcha_token.read().is_none()
            || connecting.get()
    });

    let do_signin = {
        let toaster = expect_context::<Toasts>();
        let backend = expect_context::<BackendClient>();
        let auth_context = expect_context::<AuthContext>();
        let navigate = use_navigate();

        move |username, password| {
            connecting.set(true);
            let navigate = navigate.clone();
            spawn_local({
                async move {
                    match backend
                        .post_signin(&SignInRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            username,
                            password,
                        })
                        .await
                    {
                        Ok(response) => {
                            auth_context.sign_in(response.jwt);
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

    let signin = {
        let do_signin = do_signin.clone();
        move || {
            do_signin(username.get().unwrap(), password.get().unwrap());
            set_username_storage.set(username.get());
        }
    };

    let disable_guest = Signal::derive(move || captcha_token.read().is_none() || connecting.get());
    let show_guest_modal = RwSignal::new(false);

    let guest_signin = move || match (
        get_guest_username_storage.get(),
        get_guest_password_storage.get(),
    ) {
        (Some(guest_username), Some(guest_password)) => {
            do_signin(guest_username, guest_password);
        }
        _ => show_guest_modal.set(true),
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
                <Logo />
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
                    <MenuButton on:click=move |_| guest_signin() disabled=disable_guest class:mb-4>
                        "Play as Guest"
                    </MenuButton>
                    <MenuButton on:click=navigate_to_signup>"Create Account"</MenuButton>
                    <MenuButton on:click=navigate_to_leaderboard>"Leaderboard"</MenuButton>

                    <ForgotPasswordModal open=show_forgot_password_modal />
                    <GuestModal open=show_guest_modal captcha_token />
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
pub fn Logo() -> impl IntoView {
    view! {
        <div class="relative flex flex-col items-center leading-none select-none">
            <span class="
            text-[3.8rem] xl:text-[6rem]
            font-extrabold tracking-wide [font-variant:small-caps]
            bg-gradient-to-b from-amber-200 via-amber-500 to-orange-800
            bg-clip-text text-transparent
            drop-shadow-[0_4px_2px_rgba(0,0,0,0.55)]
            [text-shadow:1px_2px_0px_#dac66eff,2px_4px_3px_rgba(0,0,0,0.7)]
            ">"GrinD"</span>

            <span class="
            text-[1.2rem] xl:text-[2rem]
            font-extrabold [font-variant:small-caps] -mt-1 xl:-mt-2 -mb-3 xl:-mb-6 tracking-wider
            bg-gradient-to-b from-amber-100 via-amber-400 to-amber-700
            bg-clip-text text-transparent
            drop-shadow-[0_3px_2px_rgba(0,0,0,0.6)]
            [text-shadow:1px_1px_0px_#dac66eff,2px_3px_2px_rgba(0,0,0,0.6)]
            ">"to"</span>

            <span class="
            text-[4rem] xl:text-[6.4rem]
            font-extrabold tracking-wide [font-variant:small-caps]
            bg-gradient-to-b from-amber-300 via-orange-500 to-[#7a3b00]
            bg-clip-text text-transparent
            drop-shadow-[0_5px_3px_rgba(0,0,0,0.55)]
            [text-shadow:1px_2px_0px_#dac66eff,3px_5px_4px_rgba(0,0,0,0.8)]
            ">"RusT"</span>
        </div>
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

    let node_ref = NodeRef::new();
    let _ = leptos_use::on_click_outside(node_ref, move |_| open.set(false));

    let on_submit = {
        move |_| {
            if processing.get() {
                return;
            }

            processing.set(true);
            spawn_local({
                async move {
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
                                toaster,
                                "Password reset instructions sent!",
                                ToastVariant::Success,
                            );
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Incorrect email: {e}"),
                                ToastVariant::Error,
                            );
                        }
                    }
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
                <div
                    node_ref=node_ref
                    class="bg-zinc-900 ring-1 ring-zinc-700 rounded-lg p-6 w-full max-w-md shadow-xl text-gray-200 space-y-4"
                >
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
                                "Check your email and follow the reset link to set a new password. Please check your spams, I don't have money to pay for a proper mail service."
                            </p>
                            <MenuButton on:click=on_close>"Close"</MenuButton>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn GuestModal(open: RwSignal<bool>, captcha_token: RwSignal<Option<String>>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();
    let auth_context = expect_context::<AuthContext>();

    let (_, set_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("username");

    let (_, set_guest_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_username");

    let (_, set_guest_password_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_password");

    let username = RwSignal::new({
        let mut rng = rand::rng();
        Username::try_new(format!(
            "Guest_{}",
            rng.random_range(111_111_111..999_999_999)
        ))
        .ok()
    });
    let password = RwSignal::new({
        let rng = rand::rng();
        let pwd: String = rng
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        Password::try_new(pwd).ok()
    });

    let accepted_terms = RwSignal::new(false);

    let processing = RwSignal::new(false);

    let node_ref = NodeRef::new();
    let _ = leptos_use::on_click_outside(node_ref, move |_| open.set(false));

    let on_submit = {
        let navigate = use_navigate();
        move || {
            if processing.get() {
                return;
            }
            processing.set(true);

            let navigate = navigate.clone();
            spawn_local({
                async move {
                    match backend
                        .post_signup(&SignUpRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            username: username.get_untracked().unwrap(),
                            email: None,
                            password: password.get_untracked().unwrap(),
                            accepted_terms: accepted_terms.get_untracked(),
                        })
                        .await
                    {
                        Ok(_) => {
                            set_guest_username_storage.set(username.get_untracked());
                            set_guest_password_storage.set(password.get_untracked());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Failed to register Guest User: {e}"),
                                ToastVariant::Error,
                            );
                            open.set(false);
                            return;
                        }
                    }

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

                            navigate("user-dashboard", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Authentication error: {e}"),
                                ToastVariant::Error,
                            );
                        }
                    }

                    set_username_storage.set(username.get_untracked());
                    processing.set(false);
                }
            });
        }
    };

    let disable_submit = Signal::derive(move || {
        username.read().is_none()
            || !accepted_terms.get()
            || captcha_token.read().is_none()
            || processing.get()
    });

    view! {
        <Show when=move || {
            open.get()
        }>
            {
                let on_submit = on_submit.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
                        <div
                            node_ref=node_ref
                            class="bg-zinc-900 ring-1 ring-zinc-700 rounded-lg p-6 w-full max-w-md shadow-xl text-gray-200 space-y-4"
                        >
                            <h2 class="text-xl font-bold text-amber-300">"Guest"</h2>

                            <p class="text-gray-400 text-sm leading-relaxed">
                                "Playing as guest will create a new account with random username and password stored locally in your browser.
                                You will be able to change them later from the 'Account Settings' page."
                            </p>

                            <ValidatedInput
                                id="username"
                                input_type="text"
                                label="Username:"
                                placeholder="Enter your username"
                                bind=username
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
                                    <a
                                        href="terms"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="text-amber-300 underline hover:text-amber-200"
                                    >
                                        "Terms & Conditions"
                                    </a>
                                    " and I have read the "
                                    <a
                                        href="privacy"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="text-amber-300 underline hover:text-amber-200"
                                    >
                                        "Privacy Notice"
                                    </a>
                                    "."
                                </label>
                            </div>

                            <div class="flex justify-between gap-2 pt-2">
                                <MenuButton on:click=move |_| {
                                    open.set(false);
                                }>"Cancel"</MenuButton>
                                <MenuButton on:click=move |_| on_submit() disabled=disable_submit>
                                    {move || {
                                        if processing.get() { "Connecting..." } else { "Play" }
                                    }}
                                </MenuButton>
                            </div>
                        </div>
                    </div>
                }
            }
        </Show>
    }
}
