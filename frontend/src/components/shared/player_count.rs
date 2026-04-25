use std::collections::HashMap;

use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;
use shared::{
    data::{
        area::{AreaLevel, AreaSpecs},
        realms::Realm,
    },
    http::server::PlayersCountResponse,
};

use crate::components::{accessibility::AccessibilityContext, backend_client::BackendClient};

#[derive(Clone, PartialEq)]
struct PlayerGlimpseEntry {
    character_name: String,
    username: String,
    area_name: String,
    area_level: AreaLevel,
    realm_label: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
enum PlayerCountStatus {
    Loading,
    Online,
    Offline,
}

fn realm_label(realm: Realm) -> &'static str {
    match realm {
        Realm::Standard => "Standard",
        Realm::StandardSSF => "Standard SSF",
        Realm::Legacy => "Legacy",
    }
}

#[component]
pub fn PlayerCount() -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let accessibility: AccessibilityContext = expect_context();
    let navigate = use_navigate();

    let areas_and_players_data: RwSignal<
        Option<(HashMap<String, AreaSpecs>, PlayersCountResponse)>,
    > = RwSignal::new(None);
    let show_glimpse = RwSignal::new(false);
    let status = RwSignal::new(PlayerCountStatus::Loading);
    let player_count = Memo::new(move |_| {
        areas_and_players_data.with(|data| {
            data.as_ref()
                .map(|(_, resp)| resp.value)
                .unwrap_or_default()
        })
    });
    let player_glimpse = Memo::new(move |_| {
        areas_and_players_data.with(|data| {
            data.as_ref()
                .map(|(areas, resp)| {
                    resp.glimpse
                        .iter()
                        .map(|entry| PlayerGlimpseEntry {
                            character_name: entry.character_name.clone(),
                            username: entry.username.clone(),
                            area_name: areas
                                .get(&entry.area_id)
                                .map(|area| area.name.clone())
                                .unwrap_or_else(|| "Somewhere".into()),
                            area_level: entry.area_level,
                            realm_label: realm_label(entry.realm),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
    });

    LocalResource::new({
        move || async move {
            let areas = backend
                .get_areas()
                .await
                .map(|resp| resp.areas)
                .unwrap_or_default();

            match backend.get_players_count().await {
                Ok(resp) => {
                    status.set(PlayerCountStatus::Online);
                    areas_and_players_data.set(Some((areas, resp)));
                }
                Err(_) => {
                    status.set(PlayerCountStatus::Offline);
                    areas_and_players_data.set(None);
                }
            }
        }
    });

    (!accessibility.is_on_mobile()).then(|| {
        view! {
            <div
                class="fixed bottom-2 right-2 z-50 select-none cursor-pointer
                rounded-md border border-zinc-700/70 bg-black/45 px-2.5 py-1.5
                text-xs text-zinc-300 shadow-md backdrop-blur-sm transition-colors
                hover:border-zinc-500/80 hover:bg-black/60"
                on:click=move |_| {
                    show_glimpse.update(|s| *s = !*s);
                }
            >
                <div class="flex items-center gap-2">
                    {move || match status.get() {
                        PlayerCountStatus::Online => {
                            view! {
                                <span class="h-1.5 w-1.5 rounded-full bg-emerald-400/80"></span>
                                <span class="font-semibold text-zinc-200">
                                    {player_count.get()}
                                </span>
                                <span>"Grinding"</span>
                            }
                                .into_any()
                        }
                        PlayerCountStatus::Offline => {
                            view! {
                                <span class="h-1.5 w-1.5 rounded-full bg-red-400/80"></span>
                                <span class="font-semibold text-zinc-200">"Offline"</span>
                            }
                                .into_any()
                        }
                        PlayerCountStatus::Loading => {
                            view! {
                                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500/80"></span>
                                <span class="font-semibold text-zinc-300">"Loading"</span>
                            }
                                .into_any()
                        }
                    }}
                </div>

                {move || {
                    show_glimpse
                        .get()
                        .then(|| {
                            let glimpse = player_glimpse.get();

                            view! {
                                {if status.get() == PlayerCountStatus::Offline {
                                    view! {
                                        <div class="absolute bottom-full right-0 z-50 mb-2 min-w-48
                                        rounded-md border border-zinc-700/80 bg-zinc-900/95 p-3
                                        text-xs text-zinc-300 shadow-lg backdrop-blur-sm">
                                            <div class="text-center text-red-300/90">
                                                "Backend offline"
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                } else if glimpse.is_empty() {
                                    view! {
                                        <div class="absolute bottom-full right-0 z-50 mb-2 min-w-48
                                        rounded-md border border-zinc-700/80 bg-zinc-900/95 p-3
                                        text-xs text-zinc-300 shadow-lg backdrop-blur-sm">
                                            <div class="text-center text-zinc-500">
                                                "No players grinding"
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div class="absolute bottom-full right-0 z-50 mb-2 w-[28rem] max-h-60 overflow-auto
                                        rounded-md border border-zinc-700/80 bg-zinc-900/95 p-1.5
                                        text-xs text-zinc-300 shadow-lg backdrop-blur-sm">
                                            {glimpse
                                                .into_iter()
                                                .map(|entry| {
                                                    let href = format!(
                                                        "/view-character/{}",
                                                        &entry.character_name,
                                                    );
                                                    let navigate = navigate.clone();
                                                    view! {
                                                        <button
                                                            class="btn grid w-full grid-cols-[minmax(0,1fr)_auto]
                                                            items-center gap-3 rounded px-2 py-1 text-left hover:bg-zinc-800/70"
                                                            on:click=move |ev: web_sys::MouseEvent| {
                                                                ev.stop_propagation();
                                                                navigate(&href, Default::default());
                                                            }
                                                        >
                                                            <div class="min-w-0 leading-tight">
                                                                <span class="block truncate font-semibold text-zinc-100">
                                                                    {entry.character_name.clone()}
                                                                </span>
                                                                <span class="block truncate text-zinc-500">
                                                                    {entry.username.clone()}
                                                                </span>
                                                            </div>
                                                            <div class="min-w-0 text-right leading-tight">
                                                                <div class="truncate font-medium text-amber-200/90">
                                                                    {entry.area_name}
                                                                    <span class="font-semibold text-yellow-100">
                                                                        " "{entry.area_level}
                                                                    </span>
                                                                </div>
                                                                <div class="text-zinc-500">{entry.realm_label}</div>
                                                            </div>
                                                        </button>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    }
                                        .into_any()
                                }}
                            }
                        })
                }}
            </div>
        }
    })
}

// #[component]
// pub fn PlayerCount() -> impl IntoView {
//     let players_count = LocalResource::new({
//         let backend = expect_context::<BackendClient>();
//         move || async move {
//             backend
//                 .get_players_count()
//                 .await
//                 .map(|r| r.value)
//                 .unwrap_or_default()
//         }
//     });

//     let accessibility: AccessibilityContext = expect_context();

//     (!accessibility.is_on_mobile()).then(|| view! {
//         <div class="fixed bottom-2 right-2 bg-black/70 text-amber-300 px-3 py-1
//         rounded-lg text-sm shadow-lg font-semibold backdrop-blur-sm
//         border border-gray-700 z-50">
//             "Players online: " {move || players_count.get().map(|x| x.take()).unwrap_or_default()}
//         </div>
//     })
// }
