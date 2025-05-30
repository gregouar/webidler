use anyhow::Result;

use codee::string::JsonSerdeCodec;
use leptos::html::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use leptos_use::storage::{use_local_storage, use_session_storage};
use reqwest;
use serde_json;

use shared::http::server::PlayersCountResponse;
use shared::messages::SessionKey;

use crate::components::ui::buttons::MenuButton;

#[component]
pub fn MainMenu() -> impl IntoView {
    let players_count = LocalResource::new(|| async {
        get_players_count("https://webidler.gregoirenaisse.be")
            .await
            .map(|r| r.value)
            .unwrap_or_default()
    });

    let (_, _, delete_session_key) =
        use_session_storage::<Option<SessionKey>, JsonSerdeCodec>("session_key");
    let (get_user_id, set_user_id_storage, _) =
        use_local_storage::<String, JsonSerdeCodec>("user_id");
    let user_id = RwSignal::new(get_user_id.get_untracked());
    let disable_connect = Signal::derive(move || user_id.read().is_empty());

    let navigate_to_online_game = {
        let navigate = use_navigate();
        let delete_session_key = delete_session_key.clone();
        move |_| {
            delete_session_key();
            set_user_id_storage.set(user_id.get_untracked());
            navigate("game", Default::default());
        }
    };

    let navigate_to_local_game = {
        let navigate = use_navigate();
        let delete_session_key = delete_session_key.clone();
        move |_| {
            delete_session_key();
            set_user_id_storage.set(user_id.get_untracked());
            navigate("local_game", Default::default());
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
                    <div class="w-full mx-auto mb-6 text-left">
                        <label for="username" class="block mb-2 text-sm font-medium text-gray-300">
                            "Username:"
                        </label>
                        <input
                            id="username"
                            type="text"
                            bind:value=user_id
                            placeholder="Enter your username"
                            class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400
                            focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        />
                    </div>
                    <MenuButton on:click=navigate_to_online_game disabled=disable_connect>
                        "Play Online"
                    </MenuButton>
                    <MenuButton on:click=navigate_to_local_game disabled=disable_connect>
                        "Play Locally"
                    </MenuButton>
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

async fn get_players_count(host: &str) -> Result<PlayersCountResponse> {
    Ok(serde_json::from_str(
        &reqwest::get(format!("{}/players", host))
            .await?
            .error_for_status()?
            .text()
            .await?,
    )?)
}
