use anyhow::Result;

use leptos::html::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

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

    let navigate = use_navigate();
    let navigate_to_online_game = move |_| navigate("./game", Default::default());

    let navigate = use_navigate();
    let navigate_to_local_game = move |_| navigate("./local_game", Default::default());

    let toast_context = expect_context::<Toasts>();
    let toast = move |_| {
        show_toast(toast_context, "Hello!", ToastVariant::Normal);
    };

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center">
            <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">
                "Welcome to Webidler!"
            </h1>
            <div class="flex flex-col space-y-2">
                <MenuButton on:click=navigate_to_online_game>"Play Online"</MenuButton>
                <MenuButton on:click=navigate_to_local_game>"Play Locally"</MenuButton>
                <MenuButton on:click=ping_online_action>"Ping Online server"</MenuButton>
                <MenuButton on:click=ping_local_action>"Ping Local server"</MenuButton>
                <MenuButton on:click=toast>"Toast"</MenuButton>
            </div>
            <p>"From server:" {data}</p>
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
