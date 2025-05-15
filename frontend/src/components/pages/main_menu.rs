use anyhow::Result;

use codee::string::JsonSerdeCodec;
use leptos::html::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use leptos_use::storage::use_local_storage;
use reqwest;
use serde_json;

use shared::data::world::HelloSchema;

use crate::components::ui::buttons::MenuButton;
use crate::components::ui::toast::*;

#[component]
pub fn MainMenu() -> impl IntoView {
    let (data, set_data) = signal(String::from(""));

    let ping_local_action = move |_| {
        spawn_local(async move {
            set_data.set(
                get_data("http://127.0.0.1:4200")
                    .await
                    .map(|x| x.greeting)
                    .unwrap_or_else(|err| format!("Error: {}", err.to_string())),
            )
        })
    };

    let ping_online_action = move |_| {
        spawn_local(async move {
            set_data.set(
                get_data("https://webidler.gregoirenaisse.be")
                    .await
                    .map(|x| x.greeting)
                    .unwrap_or_else(|err| format!("Error: {}", err.to_string())),
            )
        })
    };

    let (get_user_id, set_user_id_storage, _) =
        use_local_storage::<String, JsonSerdeCodec>("user_id");
    let user_id = RwSignal::new(get_user_id.get_untracked());

    let navigate = use_navigate();
    let navigate_to_online_game = move |_| {
        set_user_id_storage.set(user_id.get_untracked());
        navigate("./game", Default::default());
    };

    let navigate = use_navigate();
    let navigate_to_local_game = move |_| {
        set_user_id_storage.set(user_id.get_untracked());
        navigate("./local_game", Default::default());
    };

    let toast_context = expect_context::<Toasts>();
    let toast = move |_| {
        show_toast(toast_context, "Hello!", ToastVariant::Normal);
    };

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <div>
                <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">
                    "Welcome to Webidler!"
                </h1>
                <div class="flex flex-col space-y-2">
                    <div class="w-full mx-auto mb-6 text-left">
                        <label for="username" class="block mb-2 text-sm font-medium text-gray-300">
                            Username:
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
                    <MenuButton on:click=navigate_to_online_game>"Play Online"</MenuButton>
                    <MenuButton on:click=navigate_to_local_game>"Play Locally"</MenuButton>
                    <MenuButton on:click=ping_online_action>"Ping Online server"</MenuButton>
                    <MenuButton on:click=ping_local_action>"Ping Local server"</MenuButton>
                    <MenuButton on:click=toast>"Toast"</MenuButton>
                    <p>"From server:" {data}</p>
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

async fn get_data(host: &str) -> Result<HelloSchema> {
    Ok(serde_json::from_str(
        &reqwest::get(format!("{}/hello", host))
            .await?
            .error_for_status()?
            .text()
            .await?,
    )?)
}
