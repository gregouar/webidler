use std::sync::Arc;

use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::data::{item::ItemSpecs, user::UserGrindArea};

use crate::{
    assets::img_asset,
    components::{
        shared::{
            item_card::ItemCard,
            tooltips::{item_tooltip::ItemTooltipContent, SkillTooltip},
        },
        town::TownContext,
        ui::{
            buttons::{MenuButton, MenuButtonRed},
            menu_panel::MenuPanel,
            progress_bars::CircularProgressBar,
            tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
        },
    },
};

#[component]
pub fn TownScene(#[prop(default = false)] view_only: bool) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let max_area_level = move || town_context.character.read().max_area_level;

    let open_grind_panel = RwSignal::new(false);
    let selected_area = RwSignal::new(None);

    Effect::new(move || {
        if selected_area.read().is_some() {
            open_grind_panel.set(true)
        }
    });

    view! {
        <StartGrindPanel open=open_grind_panel selected_area />
        <div class="w-full grid grid-cols-3 gap-2 xl:gap-4 p-2 xl:p-4 ">
            <PlayerCard class:col-span-1 class:justify-self-end />

            <div class="w-full col-span-2 justify-self-start">

                <div class="rounded-md shadow-md  bg-zinc-800 ring-1 ring-zinc-950 h-full w-full
                gap-1 xl:gap-2 p-1 xl:p-2 
                shadow-lg relative flex flex-col">

                    <div class="px-2 xl:px-4 relative z-10 flex items-center justify-between gap-1 xl:gap-2 flex-wrap
                    justify-between">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-lg xl:text-xl font-semibold">
                            {if view_only { "Unlocked Grinds" } else { "Choose your Grind" }}
                        </span>
                        {move || {
                            (max_area_level() > 0)
                                .then(|| {
                                    view! {
                                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-base xl:text-lg">
                                            "Item Power Level: "
                                            <span class="font-semibold">{max_area_level()}</span>
                                        </span>
                                    }
                                })
                        }}
                    </div>

                    <div class="grid grid-cols-3 xl:grid-cols-5 gap-1 xl:gap-2 p-2 xl:p-4
                    overflow-y-auto h-full place-content-start 
                    bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                        <For
                            each=move || {
                                let mut areas = town_context.areas.get();
                                areas.sort_by_key(|area| area.area_specs.starting_level);
                                areas
                            }
                            key=|area| area.area_id.clone()
                            children=move |area| {
                                view! {
                                    <GrindingAreaCard area=area.clone() view_only selected_area />
                                }
                            }
                        />
                    </div>
                </div>
            </div>

        </div>
    }
}

#[component]
fn PlayerCard() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    view! {
        <div class="
        w-full h-full flex flex-col 
        gap-1 xl:gap-2 p-1 xl:p-2 
        bg-zinc-800 ring-1 ring-zinc-950 rounded-md shadow-md 
        ">
            <div>
                <PlayerName />
            </div>

            <div class="flex flex-col gap-1 xl:gap-2">
                <div class="flex gap-1 xl:gap-2">
                    <CharacterPortrait />
                </div>

                <div class="flex-none items-center grid grid-cols-4 gap-1 xl:gap-2">
                    <For
                        each=move || {
                            0..town_context
                                .last_grind
                                .with(|last_grind| {
                                    last_grind
                                        .as_ref()
                                        .map(|last_grind| last_grind.skills_specs.len().min(4))
                                        .unwrap_or_default()
                                })
                        }
                        key=|i| *i
                        let(i)
                    >
                        <PlayerSkill index=i />
                    </For>
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

            <div class="border-6 xl:border-8 border-double border-stone-500 h-full w-full">
                <div
                    class="h-full w-full"
                    style=format!(
                        "background-image: url('{}');",
                        img_asset("ui/paper_background.webp"),
                    )
                >
                    <img
                        draggable="false"
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
    view! {
        <p class="text-shadow-md shadow-gray-950 text-amber-200 text-l xl:text-xl">
            <span class="font-bold">{character_name}</span>
        </p>
    }
}

#[component]
fn PlayerSkill(index: usize) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let icon_asset = Memo::new(move |_| {
        town_context.last_grind.with(|last_grind| {
            last_grind
                .as_ref()
                .and_then(|last_grind| last_grind.skills_specs.get(index))
                .map(|skill_specs| img_asset(&skill_specs.base.icon))
                .unwrap_or_default()
        })
    });

    let skill_name = Memo::new(move |_| {
        town_context.last_grind.with(|last_grind| {
            last_grind
                .as_ref()
                .and_then(|last_grind| last_grind.skills_specs.get(index))
                .map(|skill_specs| img_asset(&skill_specs.base.name))
                .unwrap_or_default()
        })
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let show_tooltip = move || {
        let skill_specs = town_context.last_grind.with(|last_grind| {
            last_grind
                .as_ref()
                .and_then(|last_grind| last_grind.skills_specs.get(index))
                .cloned()
        });

        if let Some(skill_specs) = skill_specs {
            let skill_specs = Arc::new(skill_specs.clone());
            tooltip_context.set_content(
                move || {
                    let skill_specs = skill_specs.clone();
                    view! { <SkillTooltip skill_specs=skill_specs /> }.into_any()
                },
                DynamicTooltipPosition::TopRight,
            );
        }
    };

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let hide_tooltip = move || tooltip_context.hide();

    view! {
        <div class="flex flex-col">
            <div
                on:touchstart=move |_| { show_tooltip() }
                on:contextmenu=move |ev| {
                    ev.prevent_default();
                }
                on:mouseenter=move |_| show_tooltip()
                on:mouseleave=move |_| hide_tooltip()
                on:click=move |_| hide_tooltip()
            >
                <button
                    class="btn p-1 w-full h-full
                    active:brightness-50 active:sepia"
                    disabled=true
                >
                    <CircularProgressBar
                        bar_color="oklch(55.5% 0.163 48.998)"
                        value=Signal::derive(|| 0.0)
                        bar_width=4
                    >
                        <img
                            draggable="false"
                            src=icon_asset
                            alt=skill_name
                            class="w-full h-full flex-no-shrink fill-current
                            xl:drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert"
                        />
                    </CircularProgressBar>
                </button>
            </div>
        </div>
    }
}

#[component]
fn GrindingAreaCard(
    area: UserGrindArea,
    view_only: bool,
    selected_area: RwSignal<Option<UserGrindArea>>,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let locked = move || {
        town_context.character.read().max_area_level < area.area_specs.required_level
            || area.area_specs.coming_soon
    };

    let play_area = {
        let area = area.clone();
        move |_| {
            if !locked() && !view_only {
                selected_area.set(Some(area.clone()));
            }
        }
    };

    view! {
        <div
            class=move || {
                format!(
                    "relative flex flex-col rounded-xl border overflow-hidden aspect-square shadow-md transition {}",
                    if locked() {
                        "bg-zinc-900 border-zinc-800 opacity-60"
                    } else if view_only {
                        "bg-zinc-800 border-zinc-700"
                    } else {
                        "bg-zinc-800 border-zinc-700 cursor-pointer hover:border-amber-400 hover:shadow-lg active:scale-95 active:border-amber-500"
                    },
                )
            }
            on:click=play_area
        >
            <div class="h-10 xl:h-16 w-full relative">
                <img
                    draggable="false"
                    src=img_asset(&area.area_specs.header_background)
                    class="object-cover w-full h-full"
                />
                <div class="absolute inset-0 bg-black/30"></div>
            </div>

            <div class="p-2 xl:p-4 space-y-1 xl:space-y-2 flex-1 flex flex-col justify-around">
                <div class="text-base xl:text-lg font-semibold text-amber-200">
                    {area.area_specs.name.clone()}
                </div>

                <div class="text-xs xl:text-sm text-gray-400">
                    "Starting Level: " {area.area_specs.starting_level}
                    {(area.area_specs.item_level_modifier > 0)
                        .then(|| format!(" (+{})", area.area_specs.item_level_modifier))}
                </div>

                <div class="text-xs xl:text-sm text-gray-400">
                    {if area.max_level_reached > 0 {
                        format!("Level Reached: {}", area.max_level_reached)
                    } else {
                        "New Grind!".to_string()
                    }}
                </div>
            </div>

            <div class="h-10 xl:h-16 w-full relative">
                <img
                    draggable="false"
                    src=img_asset(&area.area_specs.footer_background)
                    class="object-cover w-full h-full"
                />
                <div class="absolute inset-0 bg-black/20"></div>
            </div>

            <Show when=move || locked()>
                <div class="absolute inset-0 flex flex-col items-center justify-center bg-black/70 backdrop-blur-sm text-center p-2">
                    <div class="text-amber-400 text-lg font-bold tracking-wide">
                        {if area.area_specs.coming_soon { "Coming Soon..." } else { "Locked" }}
                    </div>
                    <div class="text-gray-300 text-xs mt-1">
                        {format!("Requires Level {}", area.area_specs.required_level)}
                    </div>
                    <div class="mt-2 text-xs text-gray-500 italic">
                        {if area.area_specs.coming_soon {
                            "Wait for a future update!"
                        } else {
                            "Keep grinding to unlock this area!"
                        }}

                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn StartGrindPanel(
    open: RwSignal<bool>,
    selected_area: RwSignal<Option<UserGrindArea>>,
) -> impl IntoView {
    let town_context: TownContext = expect_context();
    let max_item_level = Signal::derive(move || town_context.character.read().max_area_level);

    let play_area = {
        let navigate = use_navigate();
        let (_, set_area_id_storage, _) =
            storage::use_session_storage::<Option<String>, JsonSerdeCodec>("area_id");

        move |_| {
            if let Some(selected_area) = selected_area.get_untracked() {
                set_area_id_storage.set(Some(selected_area.area_id));
                navigate("/game", Default::default());
            }
        }
    };

    let selected_map_item_index = RwSignal::new(None);

    let selected_map = Signal::derive(move || {
        selected_map_item_index.get().and_then(|item_index: usize| {
            town_context
                .inventory
                .read()
                .bag
                .get(item_index)
                .cloned()
                .map(|item_specs: ItemSpecs| Arc::new(item_specs))
        })
    });

    let map_details = move || {
        match selected_map.get() {
            Some(selected_map) => {
                view! {
                    <div class="relative flex-shrink-0 w-1/4 aspect-[2/3]">
                        <ItemCard
                            item_specs=selected_map.clone()
                            class:pointer-events-none
                            max_item_level
                        />
                    </div>

                    <div class="flex-1 w-full max-h-full overflow-y-auto">
                        <ItemTooltipContent
                            item_specs=selected_map.clone()
                            class:select-text
                            max_item_level
                        />
                    </div>
                }
                .into_any()
            }
           None => {
                view! {
                    <div class="relative flex-shrink-0 w-1/4 aspect-[2/3]">
                        <div class="
                        relative group flex items-center justify-center w-full h-full
                        rounded-md border-2 border-zinc-700 bg-gradient-to-br from-zinc-800 to-zinc-900 opacity-70
                        "></div>
                    </div>

                    <div class="flex-1 text-gray-400">"Proclaim Edict"</div>
                }.into_any()
            }
        }
    };

    let choose_map = move |_| {
        town_context.open_inventory.set(true);
    };

    view! {
        <MenuPanel open=open>
            <div class="flex items-center justify-center p-2 xl:p-4 h-full">
                <div class="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl  w-full max-w-lg mx-auto max-h-full
                flex flex-col overflow-hidden">
                    <div class="h-10 xl:h-16 w-full relative">
                        <img
                            draggable="false"
                            src=move || {
                                selected_area
                                    .read()
                                    .as_ref()
                                    .map(|area| img_asset(&area.area_specs.header_background))
                                    .unwrap_or_default()
                            }
                            class="object-cover w-full h-full"
                        />
                        <div class="absolute inset-0 bg-black/30"></div>
                    </div>

                    <div class="flex flex-col  p-4 xl:p-8 space-y-4">

                        <h2 class="text-2xl font-bold text-amber-300 text-center">
                            {move || {
                                selected_area
                                    .read()
                                    .as_ref()
                                    .map(|area| area.area_specs.name.clone())
                                    .unwrap_or_default()
                            }}
                        </h2>

                        <span class="block text-sm font-medium text-gray-400 italic mb-2 border-b border-zinc-700">
                            {move || {
                                selected_area
                                    .read()
                                    .as_ref()
                                    .map(|area| area.area_specs.description.clone())
                                    .unwrap_or_default()
                            }}
                        </span>

                        <ul class="text-xs xl:text-sm text-gray-400">
                            <li>
                                "Starting Level: "
                                <span class="font-semibold text-white">
                                    {move || {
                                        selected_area
                                            .read()
                                            .as_ref()
                                            .map(|area| area.area_specs.starting_level)
                                            .unwrap_or_default()
                                    }}
                                </span>
                            </li>
                            <li>
                                "Item Level Modifier: "
                                <span class="font-semibold text-white">
                                    "+"
                                    {move || {
                                        selected_area
                                            .read()
                                            .as_ref()
                                            .map(|area| area.area_specs.item_level_modifier)
                                            .unwrap_or_default()
                                    }}
                                </span>
                            </li>
                        </ul>

                        <div
                            class="w-full h-full flex items-center justify-center cursor-pointer"
                            on:click=choose_map
                        >
                            <div class="flex flex-row gap-6 items-center
                            w-full h-auto aspect-5/2 overflow-y-auto
                            bg-neutral-800 rounded-lg  ring-1 ring-zinc-950  p-2">
                                {map_details}
                            </div>
                        </div>

                        <div class="flex justify-around gap-3 pt-4 border-t border-zinc-700">
                            <MenuButtonRed on:click=move |_| {
                                open.set(false)
                            }>"Cancel"</MenuButtonRed>
                            <MenuButton on:click=play_area.clone()>"Confirm"</MenuButton>
                        </div>

                    </div>

                    <div class="h-10 xl:h-16 w-full relative">
                        <img
                            draggable="false"
                            src=move || {
                                selected_area
                                    .read()
                                    .as_ref()
                                    .map(|area| img_asset(&area.area_specs.footer_background))
                                    .unwrap_or_default()
                            }
                            class="object-cover w-full h-full"
                        />
                        <div class="absolute inset-0 bg-black/20"></div>
                    </div>

                </div>
            </div>
        </MenuPanel>
    }
}
