use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::data::user::UserGrindArea;

use crate::{assets::img_asset, components::town::TownContext};

#[component]
pub fn TownScene() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    view! {
        <div class="w-full grid grid-cols-3                                                                                                                      gap-4 p-4 ">
            <PlayerCard class:col-span-1 class:justify-self-end />

            <div class="w-full col-span-2 justify-self-start">

                <div class="rounded-md shadow-md  bg-zinc-800 ring-1 ring-zinc-950 h-full w-full gap-2 p-2 shadow-lg relative flex flex-col">

                    <div class="px-4 relative z-10 flex items-center justify-between gap-2 flex-wrap">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                            "Choose your grind"
                        </span>
                    </div>

                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 p-4
                    h-full
                    bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                        <For
                            each=move || {
                                let mut areas = town_context.areas.get();
                                areas.sort_by_key(|area| area.area_specs.starting_level);
                                areas
                            }
                            key=|area| area.area_id.clone()
                            children=move |area| view! { <GrindingAreaCard area=area.clone() /> }
                        />
                    </div>
                </div>
            </div>

        </div>
    }
}

#[component]
fn PlayerCard() -> impl IntoView {
    view! {
        <div class="
        w-full h-full flex flex-col gap-2 p-2 
        bg-zinc-800 
        ring-1 ring-zinc-950
        rounded-md shadow-md 
        ">
            <div>
                <PlayerName />
            </div>

            <div class="flex flex-col gap-2">
                <div class="flex gap-2">
                    <CharacterPortrait />
                </div>

            </div>
        </div>
    }
}

#[component]
pub fn CharacterPortrait() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let image_uri = move || img_asset(&town_context.character.read().portrait);

    view! {
        <div class="flex items-center justify-center h-full w-full relative overflow-hidden">

            <div class="border-8 border-double border-stone-500  h-full w-full">
                <div
                    class="h-full w-full"
                    style=format!(
                        "background-image: url('{}');",
                        img_asset("ui/paper_background.webp"),
                    )
                >
                    <img
                        src=image_uri
                        alt="portrait"
                        class="object-cover h-full w-full transition-all duration-[5s]"
                    />

                </div>

            </div>

        </div>
    }
}

#[component]
pub fn PlayerName() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let character_name = move || town_context.character.read().name.clone();
    let max_area_level = move || town_context.character.read().max_area_level;

    view! {
        <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">
            <span class="font-bold">{character_name}</span>
            " — "
            {move || {
                if max_area_level() > 0 {
                    format!("Max Area Level: {}", max_area_level())
                } else {
                    "Newbie".to_string()
                }
            }}
        </p>
    }
}

#[component]
fn GrindingAreaCard(area: UserGrindArea) -> impl IntoView {
    let play_area = {
        let navigate = use_navigate();
        let (_, set_area_id_storage, _) =
            storage::use_session_storage::<Option<String>, JsonSerdeCodec>("area_id");

        move |_| {
            set_area_id_storage.set(Some(area.area_id.clone()));
            navigate("/game", Default::default());
        }
    };

    view! {
        <div
            class="
            bg-zinc-800 rounded-xl border border-zinc-700 shadow-md overflow-hidden 
            aspect-square
            transition cursor-pointer 
            hover:border-amber-400 hover:shadow-lg
            active:scale-95 active:border-amber-500"
            on:click=play_area
        >
            <div class="h-20 w-full relative">
                <img
                    src=img_asset(&area.area_specs.header_background)
                    class="object-cover w-full h-full"
                />
                <div class="absolute inset-0 bg-black/30"></div>
            </div>
            <div class="p-4 space-y-2">
                <div class="text-lg font-semibold text-amber-200">{area.area_specs.name}</div>
                <div class="text-sm text-gray-400">
                    "Starting Level: "{area.area_specs.starting_level}
                </div>
                <div class="text-sm text-gray-400">
                    {if area.max_level_reached > 0 {
                        format!("Level Reached: {}", area.max_level_reached)
                    } else {
                        "New Grind!".to_string()
                    }}
                </div>

            </div>
            <div class="h-12 w-full relative">
                <img
                    src=img_asset(&area.area_specs.footer_background)
                    class="object-cover w-full h-full"
                />
                <div class="absolute inset-0 bg-black/20"></div>
            </div>
        </div>
    }
}
