use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local, web_sys};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::http::client::SignInRequest;

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    captcha::*,
    ui::{buttons::MenuButton, input::Input, toast::*},
};

#[component]
pub fn MainMenuPage() -> impl IntoView {
    let players_count = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move {
            backend
                .get_players_count()
                .await
                .map(|r| r.value)
                .unwrap_or_default()
        }
    });

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

    let go_fullscreen = move || {
        let window = web_sys::window().unwrap();
        let navigator = window.navigator();

        if navigator.user_agent().unwrap_or_default().contains("Mobi") {
            let document = window.document().unwrap();
            if let Some(elem) = document.document_element() {
                let _ = elem.request_fullscreen();
            }
        }

        // if let Some(win) = web_sys::window() {
        //     if let Some(screen) = win.screen() {
        //         // screen is a getter returning Option<Screen>
        //         if let Some(orientation) = screen.orientation() {
        //             let _ = orientation.lock("landscape"); // returns a Promise
        //         }
        //     }
        // }
    };

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
                            go_fullscreen();
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

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <div class="fixed bottom-2 right-2 bg-black/70 text-amber-300 px-3 py-1 rounded-lg text-sm shadow-lg font-semibold backdrop-blur-sm border border-gray-700 z-50">
                "Players online: "
                {move || players_count.get().map(|x| x.take()).unwrap_or_default()}
            </div>
            <div>
                <h1 class="text-shadow-lg/30 shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">
                    "Grind to Rust!"
                </h1>
                <div class="flex flex-col space-y-2">
                    // <form>
                    <div class="w-full mx-auto text-left">
                        <label class="block mb-2 text-sm font-medium text-gray-400">"Login:"</label>
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
                    <Captcha token=captcha_token />
                    // </form>

                    <MenuButton on:click=move |_| signin() disabled=disable_connect class:mb-4>
                        {move || if connecting.get() { "Connecting..." } else { "Connect" }}
                    </MenuButton>
                    <MenuButton on:click=navigate_to_signup>"Create Account"</MenuButton>
                    <MenuButton on:click=navigate_to_leaderboard>"Leaderboard"</MenuButton>
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
