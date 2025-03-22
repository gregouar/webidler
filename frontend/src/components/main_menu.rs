use anyhow::Result;
use leptos::html::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use reqwest;
use serde_json;

use shared::data::HelloSchema;

use super::buttons::MainMenuButton;

#[component]
pub fn MainMenu() -> impl IntoView {
    let (data, set_data) = signal(String::from(""));

    let click_action = move |_| {
        spawn_local(async move {
            set_data.set(
                get_data()
                    .await
                    .map(|x| x.greeting)
                    .unwrap_or_else(|err| format!("Error: {}", err.to_string())),
            )
        })
    };

    let navigate = leptos_router::hooks::use_navigate();
    let navigate_to_game = move |_| navigate("/game", Default::default());

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center text-white font-serif">
            <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">"Welcome to Webidler!"</h1>
            <div class="flex flex-col space-y-2">
                <MainMenuButton on:click=navigate_to_game>
                    "New Game"
                </MainMenuButton>
                <MainMenuButton on:click=click_action>
                    "Get from server"
                </MainMenuButton>
            </div>
            <p>
                "From server:" {data}
            </p>
        </main>
    }
}

async fn get_data() -> Result<HelloSchema> {
    Ok(serde_json::from_str(
        &reqwest::get("http://127.0.0.1:4200/hello")
            .await?
            .error_for_status()?
            .text()
            .await?,
    )?)
}
