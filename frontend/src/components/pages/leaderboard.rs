use std::collections::HashMap;

use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;

use crate::components::{
    backend_client::BackendClient,
    shared::player_count::PlayerCount,
    ui::{
        buttons::MenuButton,
        number::{format_datetime, format_duration},
    },
};

#[component]
pub fn LeaderboardPage() -> impl IntoView {
    let navigate_to_menu = {
        let navigate = use_navigate();
        move || {
            navigate("/", Default::default());
        }
    };

    view! {
        <main class="my-0 mx-auto w-full text-center flex flex-col justify-around">
            <PlayerCount />
            <div>
                <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl xl:text-6xl font-extrabold leading-none tracking-tight">
                    "Leaderboard"
                </h1>
                <div class="flex flex-col space-y-2">
                    <div class="w-full mx-auto mb-6 justify-center">
                        <MenuButton on:click={
                            let navigate_to_menu = navigate_to_menu.clone();
                            move |_| { navigate_to_menu() }
                        }>"Back"</MenuButton>
                        <LeaderboardPanel />
                        <MenuButton on:click={
                            let navigate_to_menu = navigate_to_menu.clone();
                            move |_| { navigate_to_menu() }
                        }>"Back"</MenuButton>
                    </div>
                </div>
            </div>
        </main>
    }
}

#[component]
pub fn LeaderboardPanel() -> impl IntoView {
    let leaderboard_and_areas = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move {
            (
                backend.get_leaderboard().await.unwrap_or_default(),
                backend
                    .get_areas()
                    .await
                    .map(|resp| resp.areas)
                    .unwrap_or_default(),
            )
        }
    });

    view! {
        <div class="p-4">
            <Suspense fallback=move || {
                view! { "Loading..." }
            }>
                {move || {
                    Suspend::new(async move {
                        let (leaderboard, areas) = leaderboard_and_areas.await;
                        let mut leaderboard_per_area = leaderboard
                            .entries
                            .into_iter()
                            .fold(
                                HashMap::new(),
                                |mut hash_map: std::collections::HashMap<_, Vec<_>>, entry| {
                                    hash_map.entry(entry.area_id.clone()).or_default().push(entry);
                                    hash_map
                                },
                            )
                            .into_iter()
                            .collect::<Vec<_>>();
                        leaderboard_per_area
                            .sort_by_key(|(area_id, _)| {
                                areas
                                    .get(area_id)
                                    .map(|area_specs| area_specs.required_level)
                                    .unwrap_or_default()
                            });

                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
                                {leaderboard_per_area
                                    .into_iter()
                                    .rev()
                                    .map(|(area_id, leaderboard)| {
                                        let area_name = {
                                            areas
                                                .get(&area_id)
                                                .map(|area_specs| area_specs.name.clone())
                                                .unwrap_or(area_id.clone())
                                        };
                                        view! {
                                            <div class="flex flex-col gap-3">
                                                <h2 class="text-shadow-lg shadow-gray-950 text-amber-300 text-lg  md:text-xl xl:text-2xl font-bold leading-none tracking-tight">
                                                    {area_name}
                                                </h2>
                                                {leaderboard
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(i, entry)| {
                                                        view! {
                                                            // TODO: display all infos and better
                                                            <div class="bg-zinc-800 border border-zinc-700 rounded-xl p-4 shadow-lg transition-shadow duration-200">
                                                                <div class="flex justify-between items-center mb-2">
                                                                    <div class="flex items-center space-x-3">
                                                                        <div class="text-2xl font-bold text-amber-300">
                                                                            #{i + 1}
                                                                        </div>
                                                                        <div class="text-white font-semibold text-lg">
                                                                            <a href=format!(
                                                                                "view-character/{}",
                                                                                &entry.character_name,
                                                                            )>{entry.character_name.clone()}</a>
                                                                        </div>
                                                                    </div>
                                                                    <div class="text-sm text-gray-400">{entry.username}</div>
                                                                </div>
                                                                <div class="flex justify-between items-center">
                                                                    <div class="text-sm text-zinc-300">
                                                                        "Level "
                                                                        <span class="font-semibold text-white">
                                                                            {entry.area_level}
                                                                        </span>
                                                                    </div>
                                                                    <div class="text-sm text-zinc-300">
                                                                        {entry
                                                                            .elapsed_time
                                                                            .map(|elapsed_time| format_duration(elapsed_time, true))}
                                                                    </div>
                                                                </div>

                                                                <div class="mt-2 text-xs text-left italic text-zinc-400 border-t border-zinc-700 pt-2 flex flex-col">
                                                                    <span>
                                                                        {format!(
                                                                            "Reached on {}",
                                                                            format_datetime(entry.created_at),
                                                                        )}
                                                                    </span>
                                                                    <span>{entry.comments.clone()}</span>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
