use std::{collections::HashMap, sync::Arc};

use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;
use rand::Rng;
use shared::{
    constants::POWER_LEVEL_LEADERBOARD_AREA_ID, data::realms::Realm, http::server::LeaderboardEntry,
};

use crate::components::{
    backend_client::BackendClient,
    data_context::DataContext,
    ui::{
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        list_row::MenuListRow,
        menu_panel::MenuPanel,
        number::{format_datetime, format_duration, format_duration_in_days},
    },
};

const LEADERBOARD_REALM_TABS: [(Realm, &str); 4] = [
    (Realm::Standard, "Standard"),
    (Realm::StandardSSF, "Standard SSF"),
    (Realm::Legacy, "Legacy"),
    (Realm::LegacySSF, "Legacy SSF"),
];

#[component]
pub fn LeaderboardPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open h_full=true>
            <MenuCard class="h-full w-full">
                <LeaderboardContent open />
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn LeaderboardContent(open: RwSignal<bool>) -> impl IntoView {
    let selected_realm = RwSignal::new(Realm::Standard);
    let selected_area_id = RwSignal::new(String::new());
    let selectable_area_ids = RwSignal::new(Vec::<String>::new());

    Effect::new(move || {
        if !open.get() {
            return;
        }

        let area_ids = selectable_area_ids.get();
        if area_ids.is_empty() {
            return;
        }

        let random_index = rand::rng().random_range(0..area_ids.len());
        selected_area_id.set(area_ids[random_index].clone());
    });

    let leaderboard_and_areas = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        let data_context = expect_context::<DataContext>();
        move || {
            let selected_realm = selected_realm.get();
            async move {
                let _ = data_context.load_data(backend).await;
                (
                    backend
                        .get_leaderboard(selected_realm)
                        .await
                        .unwrap_or_default(),
                    data_context.areas_specs.get_untracked(),
                )
            }
        }
    });

    view! {
        <CardHeader title="Leaderboard" on_close=move || open.set(false) />

        <div class="px-1 xl:px-3 grid grid-cols-2 gap-1 sm:grid-cols-4 xl:gap-2">
            {LEADERBOARD_REALM_TABS
                .into_iter()
                .map(|(realm, label)| {
                    let is_selected = Signal::derive(move || selected_realm.get() == realm);
                    view! {
                        <MenuListRow
                            selected=is_selected
                            on_click=move || selected_realm.set(realm)
                        >
                            <div class="px-2 py-1.5 text-center xl:px-3 xl:py-2">
                                <span class="text-xs sm:text-sm font-semibold uppercase tracking-[0.12em] text-amber-200/95">
                                    {label}
                                </span>
                            </div>
                        </MenuListRow>
                    }
                })
                .collect::<Vec<_>>()}
        </div>

        <Suspense fallback=move || {
            view! {
                <div class="flex flex-1 items-center justify-center text-zinc-400">
                    "Loading leaderboard..."
                </div>
            }
        }>
            {move || {
                Suspend::new(async move {
                    let (leaderboard, areas) = leaderboard_and_areas.await;
                    let leaderboard_per_area = leaderboard
                        .entries
                        .into_iter()
                        .fold(
                            HashMap::new(),
                            |mut entries: HashMap<String, Vec<LeaderboardEntry>>, entry| {
                                entries.entry(entry.area_id.clone()).or_default().push(entry);
                                entries
                            },
                        );

                    let mut crucible_area_ids = areas
                        .iter()
                        .filter(|(_, area)| area.crucible && !area.hidden)
                        .map(|(area_id, area)| {
                            (area_id.clone(), area.required_level, area.name.clone())
                        })
                        .collect::<Vec<_>>();
                    crucible_area_ids.sort_by(|left, right| {
                        right
                            .1
                            .cmp(&left.1)
                            .then_with(|| left.2.cmp(&right.2))
                            .then_with(|| left.0.cmp(&right.0))
                    });
                    let crucible_area_id = crucible_area_ids
                        .first()
                        .map(|(area_id, _, _)| area_id.clone());

                    let mut other_area_ids = leaderboard_per_area
                        .keys()
                        .filter(|area_id| {
                            area_id.as_str() != POWER_LEVEL_LEADERBOARD_AREA_ID
                                && areas
                                    .get(*area_id)
                                    .is_none_or(|area| !area.crucible)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    other_area_ids.sort_by(|left, right| {
                        let left_area = areas.get(left);
                        let right_area = areas.get(right);
                        right_area
                            .map(|area| area.required_level)
                            .unwrap_or_default()
                            .cmp(
                                &left_area
                                    .map(|area| area.required_level)
                                    .unwrap_or_default(),
                            )
                            .then_with(|| {
                                left_area
                                    .map(|area| area.name.as_str())
                                    .unwrap_or(left.as_str())
                                    .cmp(
                                        right_area
                                            .map(|area| area.name.as_str())
                                            .unwrap_or(right.as_str()),
                                    )
                            })
                    });

                    let other_area_names = other_area_ids
                        .iter()
                        .map(|area_id| {
                            (
                                area_id.clone(),
                                areas
                                    .get(area_id)
                                    .map(|area| area.name.clone())
                                    .unwrap_or_else(|| area_id.clone()),
                            )
                        })
                        .collect::<HashMap<_, _>>();

                    selectable_area_ids.set(other_area_ids.clone());

                    if !other_area_names.contains_key(&selected_area_id.get_untracked()) {
                        selected_area_id.set(
                            other_area_ids
                                .first()
                                .cloned()
                                .unwrap_or_default(),
                        );
                    }

                    let power_level_entries = leaderboard_per_area
                        .get(POWER_LEVEL_LEADERBOARD_AREA_ID)
                        .cloned()
                        .unwrap_or_default();
                    let crucible_name = crucible_area_id
                        .as_ref()
                        .and_then(|area_id| areas.get(area_id))
                        .map(|area| area.name.clone())
                        .unwrap_or_else(|| "Crucible".to_string());
                    let crucible_entries = crucible_area_id
                        .as_ref()
                        .and_then(|area_id| leaderboard_per_area.get(area_id))
                        .cloned()
                        .unwrap_or_default();

                    let leaderboard_per_area = Arc::new(leaderboard_per_area);
                    let other_area_ids = Arc::new(other_area_ids);
                    let other_area_names = Arc::new(other_area_names);

                    view! {
                        <div class="flex min-h-0 flex-1 flex-col gap-2 px-1 xl:gap-3 xl:px-3">
                            <div class="grid min-h-0 flex-1 grid-cols-1 gap-2 overflow-y-auto lg:grid-cols-3 lg:overflow-hidden xl:gap-4">
                                <LeaderboardColumn
                                    area_name="Power Level".to_string()
                                    entries=power_level_entries
                                    power_level=true
                                />
                                <LeaderboardColumn
                                    area_name=crucible_name
                                    entries=crucible_entries
                                    power_level=false
                                />
                                <SelectableLeaderboardColumn
                                    area_names=other_area_names
                                    area_ids=other_area_ids
                                    selected_area_id
                                    leaderboard_per_area
                                />
                            </div>
                        </div>
                    }
                })
            }}
        </Suspense>
    }
}

#[component]
fn LeaderboardColumn(
    area_name: String,
    entries: Vec<LeaderboardEntry>,
    power_level: bool,
) -> impl IntoView {
    view! {
        <CardInset class="min-h-[18rem] lg:min-h-0" pad=false>
            <div class="sticky top-0 z-10 px-3 pt-2">
                <CardInsetTitle>
                    <div class="flex w-full items-center justify-center gap-1 xl:gap-2">
                        <button
                            class="invisible btn flex w-8 shrink-0 justify-center text-2xl font-bold xl:w-12 xl:text-4xl"
                            tabindex="-1"
                            aria-hidden="true"
                        >
                            "←"
                        </button>
                        <div class="min-w-0 flex-1 truncate text-center">{area_name}</div>
                        <button
                            class="invisible btn flex w-8 shrink-0 justify-center text-2xl font-bold xl:w-12 xl:text-4xl"
                            tabindex="-1"
                            aria-hidden="true"
                        >
                            "→"
                        </button>
                    </div>
                </CardInsetTitle>
            </div>
            <LeaderboardEntries entries power_level />
        </CardInset>
    }
}

#[component]
fn SelectableLeaderboardColumn(
    area_names: Arc<HashMap<String, String>>,
    area_ids: Arc<Vec<String>>,
    selected_area_id: RwSignal<String>,
    leaderboard_per_area: Arc<HashMap<String, Vec<LeaderboardEntry>>>,
) -> impl IntoView {
    let previous_area_ids = area_ids.clone();
    let next_area_ids = area_ids.clone();
    let navigation_disabled = area_ids.len() <= 1;

    view! {
        <CardInset class="min-h-[18rem] lg:min-h-0" pad=false>
            <div class="sticky top-0 z-10 px-3 pt-2">
                <CardInsetTitle>
                    <div class="flex w-full items-center justify-center gap-1 xl:gap-2">
                        <button
                            class="btn flex w-8 shrink-0 justify-center text-2xl font-bold text-amber-300 drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)] transition hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)] active:scale-90 active:brightness-125 disabled:text-zinc-600 disabled:drop-shadow-none xl:w-12 xl:text-4xl"
                            title="Show previous area"
                            disabled=navigation_disabled
                            on:click=move |_| {
                                if previous_area_ids.is_empty() {
                                    return;
                                }

                                let current = selected_area_id.get_untracked();
                                let previous_index = previous_area_ids
                                    .iter()
                                    .position(|area_id| area_id == &current)
                                    .map(|index| {
                                        (index + previous_area_ids.len() - 1)
                                            % previous_area_ids.len()
                                    })
                                    .unwrap_or_default();
                                selected_area_id
                                    .set(previous_area_ids[previous_index].clone());
                            }
                        >
                            "←"
                        </button>
                        <div class="min-w-0 flex-1 truncate text-center">
                            {move || {
                                area_names
                                    .get(&selected_area_id.get())
                                    .cloned()
                                    .unwrap_or_else(|| "No areas".to_string())
                            }}
                        </div>
                        <button
                            class="btn flex w-8 shrink-0 justify-center text-2xl font-bold text-amber-300 drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)] transition hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)] active:scale-90 active:brightness-125 disabled:text-zinc-600 disabled:drop-shadow-none xl:w-12 xl:text-4xl"
                            title="Show next area"
                            disabled=navigation_disabled
                            on:click=move |_| {
                                if next_area_ids.is_empty() {
                                    return;
                                }

                                let current = selected_area_id.get_untracked();
                                let next_index = next_area_ids
                                    .iter()
                                    .position(|area_id| area_id == &current)
                                    .map(|index| (index + 1) % next_area_ids.len())
                                    .unwrap_or_default();
                                selected_area_id.set(next_area_ids[next_index].clone());
                            }
                        >
                            "→"
                        </button>
                    </div>
                </CardInsetTitle>
            </div>
            {move || {
                let entries = leaderboard_per_area
                    .get(&selected_area_id.get())
                    .cloned()
                    .unwrap_or_default();
                view! { <LeaderboardEntries entries power_level=false /> }
            }}
        </CardInset>
    }
}

#[component]
fn LeaderboardEntries(entries: Vec<LeaderboardEntry>, power_level: bool) -> impl IntoView {
    let has_entries = !entries.is_empty();

    view! {
        <div class="flex flex-col gap-2 px-2 pb-2">
            {entries
                .into_iter()
                .enumerate()
                .map(|(index, entry)| {
                    let view_character = {
                        let navigate = use_navigate();
                        let href = format!("/view-character/{}", entry.character_name);
                        move || navigate(&href, Default::default())
                    };
                    view! {
                        <MenuListRow on_click=view_character>
                            <div class="p-3">
                                <div class="flex justify-between items-center mb-2">
                                    <div class="flex items-center space-x-3">
                                        <div class="text-2xl font-bold text-amber-300">
                                            {format!("#{}", index + 1)}
                                        </div>
                                        <span class="text-white font-semibold text-lg font-display text-shadow-lg/100 shadow-gray-950">
                                            {entry.character_name.clone()}
                                        </span>
                                    </div>
                                    <div class="text-sm text-zinc-400">{entry.username}</div>
                                </div>

                                <div class="flex justify-between items-center">
                                    <div class="text-sm text-zinc-300">
                                        {if power_level { "Power Level " } else { "Level " }}
                                        <span class="font-semibold text-white">
                                            {entry.area_level}
                                        </span>
                                    </div>
                                    <div class="text-sm text-zinc-300">
                                        {if entry.elapsed_time.as_secs() > 24 * 3600 {
                                            format_duration_in_days(entry.elapsed_time)
                                        } else {
                                            format_duration(entry.elapsed_time, true)
                                        }}
                                    </div>
                                </div>

                                <div class="mt-2 text-xs text-left italic text-zinc-400 pt-2 flex flex-col">
                                    <span>
                                        {format!(
                                            "Reached on {}",
                                            format_datetime(entry.created_at),
                                        )}
                                    </span>
                                    <span>{entry.comments}</span>
                                </div>
                            </div>
                        </MenuListRow>
                    }
                })
                .collect::<Vec<_>>()}

            {(!has_entries)
                .then(|| {
                    view! {
                        <div class="flex min-h-32 items-center justify-center px-4 text-center text-sm text-zinc-500">
                            "No rankings recorded for this category."
                        </div>
                    }
                })}
        </div>
    }
}
