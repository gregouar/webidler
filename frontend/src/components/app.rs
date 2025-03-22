use anyhow::Result;
use leptos::ev;
use leptos::html::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use reqwest;
use serde_json;
use shared::data::HelloSchema;

#[component]
pub fn App() -> impl IntoView {
    let (data, set_data) = signal(String::from("waiting"));

    let click_action = move |_| {
        spawn_local(async move {
            set_data.set(
                get_data()
                    .await
                    .map(|x| x.greeting)
                    .unwrap_or_else(|err| format!("{}", err.to_string())),
            )
        })
    };

    div().child((
        button()
            .on(ev::click, click_action)
            .child("Get from server"),
        p().child(("From server:", data)),
    ))
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
