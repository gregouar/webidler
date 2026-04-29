use std::sync::Arc;

use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::data::{
    area::StartAreaConfig,
    item::{ItemCategory, ItemSpecs},
    user::UserGrindArea,
};

use crate::{
    assets::img_asset,
    components::{
        data_context::DataContext,
        game::portrait::CharacterPortrait,
        icons::area::{BossAreaIcon, CrucibleAreaIcon},
        settings::SettingsContext,
        shared::{
            inventory::InventoryEquipFilter,
            skills::{SkillProgressBar, skill_specs_from_base},
            tooltips::SkillTooltip,
        },
        town::{TownContext, items_browser::ItemDetailsPanel},
        ui::{
            Separator,
            buttons::{CloseButton, MenuButton},
            card::{Card, CardInset, CardTitle, MenuCard},
            menu_panel::MenuPanel,
            number::format_duration_in_days,
            tooltip::{
                DynamicTooltipContext, DynamicTooltipPosition, StaticTooltip, StaticTooltipPosition,
            },
        },
    },
};

#[component]
pub fn TownScene(#[prop(default = false)] view_only: bool) -> impl IntoView {
    let town_context: TownContext = expect_context();
    let data_context: DataContext = expect_context();

    let open_grind_panel = RwSignal::new(false);
    let selected_area = RwSignal::new(None);

    Effect::new(move || {
        if selected_area.read().is_some() {
            open_grind_panel.set(true)
        }
    });

    view! {
        <StartGrindPanel open=open_grind_panel selected_area />

        <div class="absolute inset-0 p-1 xl:p-4">
            <div class="relative w-full max-h-full flex justify-between gap-1 xl:gap-4 ">
                <PlayerCard />

                <Card class="w-2/3 aspect-[12/8]">
                    <div class="px-2 xl:px-4 relative z-10 flex items-center justify-between gap-1 xl:gap-2 flex-wrap
                    flex justify-between">
                        <CardTitle>"Grinds"</CardTitle>
                        <span class="text-shadow-md shadow-gray-950 text-zinc-400 text-base xl:text-lg">
                            {move || {
                                (!town_context.character.read().played_time.is_zero())
                                    .then(|| {
                                        view! {
                                            "Total Grind Time: "
                                            {format_duration_in_days(
                                                town_context.character.read().played_time,
                                            )}
                                        }
                                    })
                            }}
                        </span>
                    </div>

                    <CardInset class="min-h-0 flex-1">
                        <div class="grid grid-cols-3 xl:grid-cols-4 gap-1 xl:gap-2 place-content-start">
                            <For
                                each=move || {
                                    let mut areas = town_context.areas.get();
                                    areas
                                        .retain(|area| {
                                            data_context
                                                .areas_specs
                                                .read()
                                                .get(&area.area_id)
                                                .map(|area_specs| !area_specs.hidden)
                                                .unwrap_or_default()
                                        });
                                    areas
                                        .sort_by_key(|area| {
                                            data_context
                                                .areas_specs
                                                .read()
                                                .get(&area.area_id)
                                                .map(|area_specs| (
                                                    area_specs.coming_soon,
                                                    area_specs.required_level,
                                                ))
                                                .unwrap_or_default()
                                        });
                                    areas
                                }
                                key=|area| area.area_id.clone()
                                children=move |area| {
                                    view! {
                                        <GrindingAreaCard
                                            area=area.clone()
                                            view_only
                                            selected_area
                                        />
                                    }
                                }
                            />
                        </div>
                        <div class="flex-1"></div>
                    </CardInset>
                </Card>

            </div>
        </div>
    }
}

#[component]
fn PlayerCard() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    view! {
        <Card class="w-1/3">
            <PlayerName />

            <div class="flex-1 min-h-0 flex justify-around items-stretch gap-1 xl:gap-2">
                <div class="flex flex-col gap-1 xl:gap-2">
                    <div class="flex-1 min-h-0">
                        {move || {
                            view! {
                                <CharacterPortrait
                                    image_uri=town_context.character.read().portrait.clone()
                                    character_name="Player".into()
                                    just_hurt=Signal::derive(|| false)
                                    just_hurt_crit=Signal::derive(|| false)
                                    just_blocked=Signal::derive(|| false)
                                    just_evaded=Signal::derive(|| false)
                                    is_dead=Signal::derive(|| false)
                                    statuses=Signal::derive(Default::default)
                                />
                            }
                        }}
                    </div>
                </div>
            </div>

            <div class="flex-none items-center grid grid-cols-4 gap-1 xl:gap-2">
                <For
                    each=move || {
                        0..town_context
                            .last_grind
                            .with(|last_grind| {
                                last_grind
                                    .as_ref()
                                    .map(|last_grind| last_grind.skills.len().min(4))
                                    .unwrap_or_default()
                            })
                    }
                    key=|i| *i
                    let(i)
                >
                    <PlayerSkill index=i />
                </For>
            </div>
        </Card>
    }
}

#[component]
pub fn PlayerName() -> impl IntoView {
    let town_context = expect_context::<TownContext>();
    let max_area_level = move || town_context.character.read().max_area_level;

    let character_name = move || town_context.character.read().name.clone();
    view! {
        <p class="text-shadow-lg/100 shadow-gray-950 text-amber-200 text-l xl:text-xl">
            <span class="font-bold font-display">{character_name}</span>
            {move || {
                (max_area_level() > 0)
                    .then(|| {
                        view! {
                            <span class="text-shadow-md shadow-gray-950 text-amber-200 text-base xl:text-lg">
                                " — Power Level: "
                                <span class="font-semibold">{max_area_level()}</span>
                            </span>
                        }
                    })
            }}
            <span></span>
        </p>
    }
}

#[component]
fn PlayerSkill(index: usize) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let skill_entry = Signal::derive(move || {
        town_context.last_grind.with(|last_grind| {
            last_grind
                .as_ref()
                .and_then(|last_grind| last_grind.skills.get_index(index))
                .map(|(skill_id, player_base_skill)| (skill_id.clone(), player_base_skill.clone()))
        })
    });

    let skill_specs = Signal::derive(move || {
        skill_entry.get().map(|(skill_id, player_base_skill)| {
            skill_specs_from_base(skill_id, &player_base_skill.base_skill_specs)
        })
    });
    let skill_progress = Memo::new(move |_| {
        skill_specs
            .read()
            .as_ref()
            .map(|skill_specs| (skill_specs.skill_type, skill_specs.icon.clone()))
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let tooltip_id = RwSignal::new(0);
    let show_tooltip = move || {
        if let Some((_, player_base_skill)) = skill_entry.get()
            && let Some(skill_specs) = skill_specs.get()
        {
            let skill_specs = Arc::new(skill_specs);
            let player_base_skill = Some(Arc::new(player_base_skill));
            tooltip_id.set(tooltip_context.set_content(
                move || {
                    let skill_specs = skill_specs.clone();
                    let player_base_skill = player_base_skill.clone();
                    view! { <SkillTooltip skill_specs=skill_specs player_base_skill=player_base_skill /> }
                    .into_any()
                },
                DynamicTooltipPosition::TopRight,
            ));
        }
    };

    let hide_tooltip = move || {
        tooltip_context.hide(tooltip_id.get_untracked());
    };
    on_cleanup(hide_tooltip);

    view! {
        <div class="flex flex-col">
            <div
                on:touchstart=move |_| show_tooltip()
                on:contextmenu=move |ev| {
                    ev.prevent_default();
                }
                on:mouseenter=move |_| show_tooltip()
                on:mouseleave=move |_| hide_tooltip()
                on:click=move |_| hide_tooltip()
            >
                // <button
                // class="btn p-1 w-full h-full
                // active:brightness-50 active:sepia"
                // disabled=true
                // >
                <div class="p-1 w-full h-full">
                    {move || {
                        skill_progress
                            .read()
                            .as_ref()
                            .map(|(skill_type, skill_icon)| {
                                view! {
                                    <SkillProgressBar
                                        skill_type=*skill_type
                                        skill_icon=skill_icon.clone()
                                        value=Signal::derive(|| 0.0)
                                        bar_width=4
                                    />
                                }
                                    .into_any()
                            })
                    }}
                </div>
            // </button>
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
    let town_context: TownContext = expect_context();
    let data_context: DataContext = expect_context();
    let settings: SettingsContext = expect_context();

    let area_specs = Memo::new({
        let area_id = area.area_id.clone();
        move |_| {
            data_context
                .areas_specs
                .read()
                .get(&area_id)
                .cloned()
                .unwrap_or_default()
        }
    });

    let locked = move || {
        town_context.character.read().max_area_level < area_specs.read().required_level
            || area_specs.read().coming_soon
    };

    let select_area = {
        let area = area.clone();
        move |_| {
            if !locked() && !view_only {
                selected_area.set(Some(area.clone()));
            }
        }
    };

    view! {
        <div
            class="relative flex flex-col max-h-full transition-all active:scale-95"
            on:click=select_area
            style=move || {
                format!(
                    "pointer-events: {}; {}",
                    if locked() { "none" } else { "auto" },
                    if settings.uses_heavy_effects() {
                        "filter: drop-shadow(0 10px 25px rgba(0,0,0,0.45));"
                    } else {
                        ""
                    },
                )
            }
        >
            <div class="absolute inset-0 bg-black clip-octagon" aria-hidden="true"></div>

            <div
                class=move || {
                    format!(
                        "clip-octagon absolute inset-0 border {}",
                        if settings.uses_heavy_effects() {
                            "border-[#6c5734]/45 shadow-[inset_2px_2px_1px_rgba(255,255,255,0.06),inset_-2px_-2px_1px_rgba(0,0,0,0.15)]"
                        } else if settings.uses_surface_effects() {
                            "border-[#665131]/50"
                        } else {
                            "border-[#5c4a2e]/60"
                        },
                    )
                }
                style=move || {
                    format!(
                        "
                        {};
                        background-color: rgb(39, 39, 42);
                        ",
                        if settings.uses_textures() {
                            format!(
                                "background-image: url('{}'); background-blend-mode: normal;",
                                img_asset("ui/dark_stone.webp"),
                            )
                        } else {
                            "background-image: linear-gradient(180deg, rgba(74,69,76,0.98), rgba(30,29,34,1));"
                                .to_string()
                        },
                    )
                }
            >
                <Show when=move || settings.uses_surface_effects()>
                    <div class="pointer-events-none clip-octagon [--cut:11px] absolute inset-[1px] border border-[#d4b57a]/8"></div>
                </Show>
            </div>

            <div class=move || {
                format!(
                    "relative clip-octagon z-10 flex flex-col h-full transition-all overflow-clip {}",
                    if locked() || view_only {
                        "cursor-default"
                    } else {
                        "cursor-pointer hover:brightness-110"
                    },
                )
            }>
                <div class="m-[3px] flex-1 overflow-hidden">
                    <div class=move || {
                        format!(
                            "flex flex-col h-full transition-[filter,transform,opacity] duration-200 {}",
                            if locked() {
                                "blur-[5px] scale-[1.1] saturate-75 brightness-75"
                            } else {
                                ""
                            },
                        )
                    }>
                        <div class="h-10 xl:h-16 w-full relative flex-shrink-0">
                            <img
                                draggable="false"
                                src=move || img_asset(&area_specs.read().header_background)
                                class="object-cover w-full h-full"
                            />
                        </div>

                        <div class="p-2 xl:p-4 xl:space-y-1 flex-1 flex flex-col justify-around">
                            <div class="text-base xl:text-lg font-semibold text-amber-200 text-shadow-lg/100 shadow-gray-950 font-display">
                                {move || area_specs.read().name.clone()}
                            </div>

                            <div class="text-xs xl:text-sm text-zinc-400">
                                {move || {
                                    format!(
                                        "Power level: +{}",
                                        *area_specs.read().power_level
                                            + *area_specs.read().item_level_modifier,
                                    )
                                }}
                            </div>

                            <div class="text-xs xl:text-sm text-zinc-400">
                                {if area.max_level_reached > 0 {
                                    format!("Level Reached: {}", area.max_level_reached)
                                } else {
                                    "New Grind!".to_string()
                                }}
                            </div>
                        </div>

                        <div class="h-10 xl:h-16 w-full relative flex-shrink-0">
                            <img
                                draggable="false"
                                src=move || img_asset(&area_specs.read().footer_background)
                                class="object-cover w-full h-full"
                            />
                        </div>
                    </div>
                </div>

                <Show when=move || locked()>
                    <div class="absolute inset-[3px] z-20 flex flex-col items-center justify-center bg-black/65 text-center p-2">
                        <div class="text-amber-400 text-lg font-bold tracking-wide">
                            {if area_specs.read().coming_soon {
                                "Coming Soon..."
                            } else {
                                "Locked"
                            }}
                        </div>
                        <div class="text-gray-300 text-xs mt-1">
                            {format!("Requires Level {}", area_specs.read().required_level)}
                        </div>
                        <div class="mt-2 text-xs text-gray-500 italic">
                            {if area_specs.read().coming_soon {
                                "Wait for a future update!"
                            } else {
                                "Keep grinding to unlock this area!"
                            }}

                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn StartGrindPanel(
    open: RwSignal<bool>,
    selected_area: RwSignal<Option<UserGrindArea>>,
) -> impl IntoView {
    let town_context: TownContext = expect_context();
    let data_context: DataContext = expect_context();

    let area_specs = move || {
        selected_area.read().as_ref().map(|selected_area| {
            data_context
                .areas_specs
                .read()
                .get(&selected_area.area_id)
                .cloned()
                .unwrap_or_default()
        })
    };

    let max_item_level = Signal::derive(move || town_context.character.read().max_area_level);

    let selected_map = Signal::derive(move || {
        town_context
            .selected_item_index
            .get()
            .and_then(|item_index: u8| {
                town_context
                    .inventory
                    .read()
                    .bag
                    .get(item_index as usize)
                    .cloned()
                    .and_then(|item_specs: ItemSpecs| {
                        item_specs
                            .base
                            .categories
                            .contains(&ItemCategory::Map)
                            .then(|| Arc::new(item_specs))
                    })
            })
    });

    let choose_map = move || {
        town_context.selected_item_index.set(None);
        town_context.equip_filter.set(InventoryEquipFilter::Map(
            selected_area
                .read_untracked()
                .as_ref()
                .map(|area| area.area_id.clone())
                .unwrap_or_default(),
        ));
        town_context.open_inventory.set(true);
    };

    let disable_confirm = Signal::derive(move || {
        selected_map
            .read()
            .as_ref()
            .map(|map| map.required_level > town_context.character.read().max_area_level)
            .unwrap_or_default()
    });

    let (_, set_area_config_storage, _) =
        storage::use_session_storage::<Option<StartAreaConfig>, JsonSerdeCodec>("area_config");

    view! {
        <MenuPanel open=open w_full=false h_full=false class:items-center>
            {move || {
                area_specs()
                    .map(|area_specs| {
                        view! {
                            <MenuCard class="max-w-4xl mx-auto overflow-clip" pad=false gap=false>
                                <div class="h-10 xl:h-16 w-full relative">
                                    <img
                                        draggable="false"
                                        src=img_asset(&area_specs.header_background)
                                        class="object-cover w-full h-full"
                                    />
                                    <div class="absolute inset-0">
                                        <div class="w-full h-full flex justify-end items-center px-4">
                                            <CloseButton on:click=move |_| open.set(false) />
                                        </div>
                                    </div>
                                </div>

                                <CardInset class="xl:space-y-4">
                                    <div class="w-full flex text-lg xl:text-2xl font-bold text-shadow-lg/100 shadow-gray-950 text-amber-300 justify-center items-center gap-4">
                                        {area_specs
                                            .disable_shards
                                            .then(|| view! { <CrucibleAreaIcon /> })}
                                        {area_specs.boss.then(|| view! { <BossAreaIcon /> })}
                                        <span class="font-display">{area_specs.name}</span>
                                    </div>

                                    <span class="block text-xs xl:text-sm font-medium text-zinc-400 italic
                                    xl:mb-4 max-w-xl mx-auto">{area_specs.description}</span>

                                    <Separator />

                                    <ul class="text-xs xl:text-sm text-zinc-400 list-none xl:space-y-1">
                                        <li class=" ">
                                            "Power Level Modifier: "
                                            <span class="font-semibold text-white">
                                                "+"
                                                {*area_specs.power_level + *area_specs.item_level_modifier}
                                            </span>
                                        </li>
                                    </ul>

                                    <div class="w-full h-full px-4 flex items-center justify-center">
                                        <ItemDetailsPanel
                                            item_specs=selected_map
                                            max_item_level
                                            selected=Signal::derive(move || {
                                                selected_map.get().is_some()
                                            })
                                            on_click=choose_map
                                        >
                                            "Proclaim Edict"
                                            <StaticTooltip
                                                position=StaticTooltipPosition::Top
                                                tooltip=|| {
                                                    view! {
                                                        <div class="flex flex-col xl:space-y-1 w-[20vw] whitespace-normal">
                                                            "An Edict is a special kind of item that drops at Power Level 100 or higher. It makes enemies stronger during a Grind, but increases rewards."
                                                        </div>
                                                    }
                                                }
                                            >
                                                <span class="ml-1 inline-flex h-5 w-5 items-center justify-center rounded-full border border-zinc-500 text-xs text-zinc-300 cursor-help">
                                                    "?"
                                                </span>
                                            </StaticTooltip>
                                        </ItemDetailsPanel>
                                    </div>

                                </CardInset>

                                <div class="h-10 xl:h-16 w-full relative">
                                    <img
                                        draggable="false"
                                        src=img_asset(&area_specs.footer_background)
                                        class="object-cover w-full h-full"
                                    />
                                    <div class="absolute inset-0">
                                        <div class="w-full h-full flex justify-around px-4 py-1 xl:py-2">
                                            <MenuButton
                                                on:click={
                                                    let navigate = use_navigate();
                                                    move |e| {
                                                        e.stop_propagation();
                                                        if let Some(selected_area) = selected_area.get_untracked() {
                                                            set_area_config_storage
                                                                .set(
                                                                    Some(StartAreaConfig {
                                                                        area_id: selected_area.area_id,
                                                                        map_item_index: town_context
                                                                            .selected_item_index
                                                                            .get_untracked(),
                                                                    }),
                                                                );
                                                            navigate("/game", Default::default());
                                                        }
                                                    }
                                                }
                                                disabled=disable_confirm
                                            >
                                                "Confirm & Start Grind"
                                            </MenuButton>
                                        </div>
                                    </div>
                                </div>
                            </MenuCard>
                        }
                    })
            }}
        </MenuPanel>
    }
}
