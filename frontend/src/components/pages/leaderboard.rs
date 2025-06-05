use anyhow::Result;

use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;

use reqwest;
use serde_json;

use shared::http::server::LeaderboardResponse;

use crate::components::ui::{buttons::MenuButton, number::format_duration};

#[component]
pub fn LeaderboardPage() -> impl IntoView {
    let navigate_to_menu = {
        let navigate = use_navigate();
        move |_| {
            navigate("/", Default::default());
        }
    };

    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <div>
                <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">
                    "Leaderboard"
                </h1>
                <div class="flex flex-col space-y-2">
                    <div class="w-full mx-auto mb-6 text-left">
                        <LeaderboardPanel />
                    </div>
                    <MenuButton on:click=navigate_to_menu>"Back"</MenuButton>
                </div>
            </div>
        </main>
    }
}

#[component]
pub fn LeaderboardPanel() -> impl IntoView {
    let leaderboard = LocalResource::new(|| async {
        get_leaderboard("https://webidler.gregoirenaisse.be")
            .await
            .unwrap_or_default()
    });

    view! {
        <div class="p-4">
            <Suspense fallback=move || {
                view! { "Loading..." }
            }>
                {move || {
                    leaderboard
                        .get()
                        .map(|leaderboard| {
                            view! {
                                <div class="grid gap-4">
                                <For
                                    each=move || leaderboard.entries.clone().into_iter().enumerate()
                                    key=|(i,_)| *i
                                    let:((i, entry))
                                >
                                    <div class="bg-zinc-800 border border-zinc-700 rounded-xl p-4 shadow hover:shadow-lg transition-shadow duration-200">
                                        <div class="flex justify-between items-center mb-2">
                                            <div class="flex items-center space-x-3">
                                                <div class="text-2xl font-bold text-amber-300">
                                                    #{i + 1}
                                                </div>
                                                <div class="text-white font-semibold text-lg">
                                                    {entry.player_name}
                                                </div>
                                            </div>
                                            <div class="text-sm text-gray-400">
                                                { format!("{}", entry.created_at.format("%Y-%m-%d %H:%M"))}
                                            </div>
                                        </div>
                                        <div class="flex justify-between items-center">
                                            <div class="text-sm text-zinc-300">
                                                "Area: " <span class="font-semibold text-white">{entry.area_level}</span>
                                            </div>
                                            <div class="text-sm text-zinc-300">
                                                "Time played: " <span class="font-semibold text-white">
                                                    {format_duration(entry.time_played)}
                                                </span>
                                            </div>
                                        </div>

                                        <div class="mt-2 text-xs italic text-zinc-400 border-t border-zinc-700 pt-2">
                                            {entry.comments.clone()}
                                        </div>
                                    </div>
                                </For>
                            </div>
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

async fn get_leaderboard(host: &str) -> Result<LeaderboardResponse> {
    Ok(serde_json::from_str(
        &reqwest::get(format!("{}/leaderboard", host))
            .await?
            .error_for_status()?
            .text()
            .await?,
    )?)
}
