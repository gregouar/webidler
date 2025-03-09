use leptos::prelude::*;
use leptos::task::spawn_local;
use reqwest;

#[component]
pub fn App() -> impl IntoView {
    let (data, set_data) = signal(String::from("waiting"));

    view! {
        <button on:click=move |_| {spawn_local(async move { set_data.set(get_data().await) })}>
            "get from server:"
        </button>
        <p>
            "From server: "
            {data}
        </p>
    }
}

async fn get_data() -> String {
    // let res = reqwest::get("http://127.0.0.1:4200/pou").await;

    let res = reqwest::Client::new()
        .get("http://127.0.0.1:4200/pou")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await;

    match res {
        Ok(res) => match res.text().await {
            Ok(text) => text,
            Err(err) => {
                format!(" Error: {}", err.to_string())
            }
        },
        Err(err) => {
            format!(" Error: {}", err.to_string())
        }
    }
}
