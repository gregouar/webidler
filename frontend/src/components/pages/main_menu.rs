use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::http::client::SignInRequest;

use crate::components::{
    backend_client::BackendClient, captcha::*, game::game_instance::SessionInfos,
    ui::buttons::MenuButton, ui::toast::*,
};

#[component]
pub fn MainMenuPage() -> impl IntoView {
    let players_count = LocalResource::new({
        let backend = use_context::<BackendClient>().unwrap();
        move || async move {
            backend
                .get_players_count()
                .await
                .map(|r| r.value)
                .unwrap_or_default()
        }
    });

    let (_, _, delete_session_infos) =
        storage::use_session_storage::<Option<SessionInfos>, JsonSerdeCodec>("session_infos");
    let (get_username, set_user_id_storage, _) =
        storage::use_local_storage::<String, JsonSerdeCodec>("username");

    let username = RwSignal::new(get_username.get_untracked());
    let password = RwSignal::new(String::new());
    let captcha_token = RwSignal::new(None);

    let signin = {
        let toaster = expect_context::<Toasts>();
        let backend = use_context::<BackendClient>().unwrap();
        move |_| {
            spawn_local({
                async move {
                    match backend
                        .post_signin(&SignInRequest {
                            captcha_token: captcha_token.get().unwrap_or_default(),
                            username: username.get(),
                            password: password.get(),
                        })
                        .await
                    {
                        Ok(_) => show_toast(toaster, "Connected!", ToastVariant::Success),
                        Err(e) => show_toast(
                            toaster,
                            format!("Authentication error: {e:?}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    };

    // TODO: connect pending
    let disable_connect = Signal::derive(move || {
        username.read().is_empty() || password.read().is_empty() || captcha_token.read().is_none()
    });

    // let navigate_to_game = {
    //     let navigate = use_navigate();
    //     let delete_session_infos = delete_session_infos.clone();
    //     move |_| {
    //         // TODO: give token to backend alongside
    //         delete_session_infos();
    //         set_user_id_storage.set(username.get_untracked());
    //         navigate("game", Default::default());
    //     }
    // };

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

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <div class="fixed bottom-2 right-2 bg-black/70 text-amber-300 px-3 py-1 rounded-lg text-sm shadow-lg font-semibold backdrop-blur-sm border border-gray-700 z-50">
                "Players online: "
                {move || players_count.get().map(|x| x.take()).unwrap_or_default()}
            </div>
            <div>
                <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">
                    "Grind to Rust!"
                </h1>
                <div class="flex flex-col space-y-2">
                    // <form>
                    <div class="w-full mx-auto text-left">
                        <label class="block mb-2 text-sm font-medium text-gray-300">"Login:"</label>
                        <input
                            id="username"
                            type="text"
                            bind:value=username
                            placeholder="Enter your username"
                            class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400
                            focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        />
                    </div>
                    <div class="w-full mx-auto mb-6 text-left">
                        <input
                            id="password"
                            type="password"
                            bind:value=password
                            placeholder="Enter your password"
                            class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400
                            focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        />
                    </div>
                    <Captcha token=captcha_token />
                    // </form>

                    <MenuButton on:click=signin disabled=disable_connect>
                        "Connect"
                    </MenuButton>
                    <MenuButton on:click=navigate_to_signup>"Create Account"</MenuButton>
                    <MenuButton on:click=navigate_to_leaderboard>"Leaderboard"</MenuButton>
                </div>
            </div>

            <div class="bg-gray-800 text-gray-200 text-sm p-4 rounded-xl border border-gray-700 shadow-inner">
                <h2 class="text-lg font-semibold mb-2">Disclaimer</h2>
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
                <p class="mt-2">
                    "Musics are created with the help of Suno's generative AI tools (free version via "
                    <a
                        href="https://suno.com"
                        class="text-amber-300 underline hover:text-amber-200"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "suno.com"
                    </a>")."
                </p>
            </div>
        </main>
    }
}
