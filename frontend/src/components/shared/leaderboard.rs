use std::collections::HashMap;

use leptos::{html::*, prelude::*};
use shared::data::realms::Realm;

use crate::components::{
    backend_client::BackendClient,
    ui::{
        ALink,
        card::{CardHeader, CardInset, MenuCard},
        list_row::MenuListRow,
        menu_panel::MenuPanel,
        number::{format_datetime, format_duration},
    },
};

const LEADERBOARD_REALM_TABS: [(Realm, &str); 3] = [
    (Realm::Standard, "Standard"),
    (Realm::StandardSSF, "Standard SSF"),
    (Realm::Legacy, "Legacy"),
];

#[component]
pub fn LeaderboardPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open>
            <MenuCard>
                <CardHeader title="Leaderboard" on_close=move || open.set(false) />
                <LeaderboardContent />
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn LeaderboardContent() -> impl IntoView {
    let selected_realm = RwSignal::new(Realm::Standard);

    let leaderboard_and_areas = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || {
            let selected_realm = selected_realm.get();
            async move {
                (
                    backend
                        .get_leaderboard(selected_realm)
                        .await
                        .unwrap_or_default(),
                    backend
                        .get_areas()
                        .await
                        .map(|resp| resp.areas)
                        .unwrap_or_default(),
                )
            }
        }
    });

    view! {
        <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-3">
            {LEADERBOARD_REALM_TABS
                .into_iter()
                .map(|(realm, label)| {
                    let is_selected = Signal::derive(move || selected_realm.get() == realm);
                    view! {
                        <MenuListRow
                            selected=is_selected
                            on_click=move || selected_realm.set(realm)
                        >
                            <div class="px-3 py-2 text-center">
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
                        <div class="min-h-0 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
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
                                        <CardInset pad=false>
                                            <h2 class=" text-shadow-lg/100 shadow-gray-950 text-amber-300
                                            text-sm xl:text-base mb-2 font-display
                                            font-bold leading-none tracking-tight my-2">{area_name}</h2>
                                            <div class="h-full overflow-y-auto px-2">
                                                {leaderboard
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(i, entry)| {
                                                        view! {
                                                            <MenuListRow class="mb-2">
                                                                <div class="p-3">
                                                                    <div class="flex justify-between items-center mb-2">
                                                                        <div class="flex items-center space-x-3">
                                                                            <div class="text-2xl font-bold text-amber-300">
                                                                                #{i + 1}
                                                                            </div>
                                                                            <ALink
                                                                                href=format!("/view-character/{}", &entry.character_name)
                                                                                underline=false
                                                                            >
                                                                                <span class="text-white font-semibold text-lg font-display text-shadow-lg/100 shadow-gray-950">
                                                                                    {entry.character_name.clone()}
                                                                                </span>
                                                                            </ALink>
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
                                                                            {format_duration(entry.elapsed_time, true)}
                                                                        </div>
                                                                    </div>

                                                                    <div class="mt-2 text-xs text-left italic text-zinc-400 pt-2 flex flex-col">
                                                                        <span>
                                                                            {format!(
                                                                                "Reached on {}",
                                                                                format_datetime(entry.created_at),
                                                                            )}
                                                                        </span>
                                                                        <span>{entry.comments.clone()}</span>
                                                                    </div>
                                                                </div>
                                                            </MenuListRow>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </div>
                                        </CardInset>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }
                })
            }}
        </Suspense>
    }
}
