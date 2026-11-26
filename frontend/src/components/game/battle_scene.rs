use leptos::{html::*, prelude::*};

use shared::{
    computations,
    constants::{MAX_AREA_LEVEL, WAVES_PER_AREA_LEVEL},
    messages::client::{
        ClientMessage, GoBackLevelMessage, SetAutoProgressMessage, SetRushModeMessage,
    },
};

use crate::{
    assets::img_asset,
    components::{
        chat::chat_panel::ChatPanel,
        events::{EventsContext, Key},
        game::{
            GameContext, loot_queue::LootQueue, monsters_grid::MonstersGrid,
            player_card::PlayerCard, websocket::WebsocketContext,
        },
        icons::{
            area::{BossAreaIcon, CrucibleAreaIcon, TrainingAreaIcon},
            battle_scene::{AreaLevelIcon, EdictIcon, RushIcon, ThreatIcon},
        },
        ui::{
            card::Card,
            number::{NumberInset, format_duration},
            progress_bars::{VerticalProgressBar, predictive_cooldown},
            tooltip::{StaticTooltip, StaticTooltipPosition},
            tutorial_popup::{TutorialPopup, TutorialPopupPosition},
        },
    },
};

#[component]
pub fn BattleScene() -> impl IntoView {
    let game_context: GameContext = expect_context();
    view! {
        <div class="absolute inset-0 p-1 xl:p-4">
            <div class="relative w-full max-h-full flex justify-between gap-1 xl:gap-4 ">
                <div class="relative w-1/3 min-w-0">
                    <div class="absolute inset-0 flex flex-col gap-1 xl:gap-4">
                        <PlayerCard />
                        <ChatPanel
                            character_id=game_context.character_id.get_untracked()
                            pinnable=true
                        />
                    </div>
                </div>
                <Card class="w-2/3 aspect-[12/8]" pad=false gap=false>
                    // <div class="w-2/3 aspect-[12/8] flex flex-col shadow-xl/30 rounded-md overflow-clip">
                    <BattleSceneHeader />
                    <div
                        class="flex relative w-full flex-1 min-h-0 isolate
                        bg-stone-800 overflow-clip shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]"
                        style="contain: layout paint;"
                    >
                        <Show when=move || !game_context.area_state.read().rush_mode>
                            <MonstersGrid />
                            <ThreatMeter />
                        </Show>

                        // <Show when=move || game_context.area_state.read().rush_mode>
                        <div
                            class="absolute inset-0 opacity-0 transition-opacity duration-500 pointer-events-none"
                            class:opacity-100=move || game_context.area_state.read().rush_mode
                            style="contain: layout paint;"
                        >
                            <RushOverlay />
                        </div>
                    // </Show>
                    </div>
                    <LootQueue />
                    <BattleSceneFooter />
                </Card>
            </div>
        </div>
    }
}

#[component]
pub fn BattleSceneHeader() -> impl IntoView {
    let game_context: GameContext = expect_context();
    let events_context: EventsContext = expect_context();

    let go_back = {
        let conn: WebsocketContext = expect_context();
        move |_| {
            let amount = if events_context.key_pressed(Key::Ctrl) {
                10
            } else {
                1
            };
            conn.send(&GoBackLevelMessage { amount }.into());
            game_context.area_state.update(|area_state| {
                area_state.going_back = area_state.going_back.saturating_add(amount);
            });
        }
    };

    let go_forth = {
        let conn: WebsocketContext = expect_context();
        move |_| {
            let requested_amount = if events_context.key_pressed(Key::Ctrl) {
                10
            } else {
                1
            };
            let amount = {
                let area_state = game_context.area_state.read_untracked();
                let max_area_level = if game_context.area_specs.read_untracked().training {
                    MAX_AREA_LEVEL
                } else {
                    area_state.max_area_level
                };
                let target_area_level = (area_state.area_level as i32)
                    .saturating_sub(area_state.going_back)
                    .clamp(1, MAX_AREA_LEVEL as i32) as u16;
                -i32::from(requested_amount.min(max_area_level.saturating_sub(target_area_level)))
            };
            if amount == 0 {
                return;
            }
            conn.send(&GoBackLevelMessage { amount }.into());
            game_context.area_state.update(|area_state| {
                area_state.going_back = area_state.going_back.saturating_add(amount);
            });
        }
    };

    let toggle_auto_progress = {
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            let auto_progress = !game_context.area_state.read_untracked().auto_progress;
            game_context.area_state.write().auto_progress = auto_progress;
            conn.send(
                &SetAutoProgressMessage {
                    value: auto_progress,
                }
                .into(),
            );
        }
    };

    let toggle_rush_mode = {
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            let rush_mode = !game_context.area_state.read_untracked().rush_mode;
            game_context.area_state.write().rush_mode = rush_mode;
            conn.send(&SetRushModeMessage { value: rush_mode }.into());
        }
    };

    let header_background = move || {
        format!(
            "background-image: url('{}'); contain: layout paint;",
            img_asset(&game_context.area_specs.read().header_background)
        )
    };

    let auto_icon = move || {
        if game_context.area_state.read().auto_progress {
            "⏸"
        } else {
            "▶"
        }
    };

    let disable_rush = Memo::new(move |_| {
        game_context.player_stamina.read().is_zero() || game_context.area_state.read().incense_lit
    });
    let disable_go_forth = Memo::new(move |_| {
        game_context.area_state.with(|area_state| {
            let max_area_level = if game_context.area_specs.read().training {
                MAX_AREA_LEVEL
            } else {
                area_state.max_area_level
            };
            let target_area_level = (area_state.area_level as i32)
                .saturating_sub(area_state.going_back)
                .clamp(1, MAX_AREA_LEVEL as i32) as u16;
            target_area_level >= max_area_level
        })
    });

    view! {
        <div
            class="h-8 xl:h-16 relative overflow-clip w-full
            bg-center bg-repeat-x flex items-center justify-between px-4"
            style=header_background
        >
            // <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>

            <div class="w-12 flex justify-start">
                <StaticTooltip
                    position=StaticTooltipPosition::Right
                    tooltip=|| "Go Back one Area Level & Pause Progression"
                >
                    <button
                        class="btn text-2xl xl:text-4xl text-amber-300 font-bold drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)]
                        hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)]
                        active:scale-90 active:brightness-125 transition"
                        title="Go Back One Level"
                        on:click=go_back
                    >
                        "←"
                    </button>
                </StaticTooltip>
            </div>

            <div class="w-12 flex justify-start">
                <StaticTooltip
                    position=StaticTooltipPosition::Right
                    tooltip=move || {
                        if disable_go_forth.get() {
                            "Maximum beaten Area Level reached"
                        } else {
                            "Go Forth one Area Level"
                        }
                    }
                >
                    <button
                        class="btn text-2xl xl:text-4xl text-amber-300 font-bold drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)]
                        hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)]
                        active:scale-90 active:brightness-125 transition
                        disabled:text-zinc-500 disabled:drop-shadow-none disabled:cursor-not-allowed"
                        title="Go Forth One Level"
                        on:click=go_forth
                        disabled=disable_go_forth
                    >
                        "→"
                    </button>
                </StaticTooltip>
            </div>

            <div class="flex-1 text-center relative">
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>
                <div class="relative z-10 inline-flex items-center justify-center space-x-2 xl:space-x-4
                text-shadow/30 text-amber-200">
                    {move || {
                        game_context
                            .area_specs
                            .read()
                            .crucible
                            .then(|| view! { <CrucibleAreaIcon /> })
                    }}
                    {move || {
                        game_context
                            .area_specs
                            .read()
                            .training
                            .then(|| view! { <TrainingAreaIcon /> })
                    }}
                    {move || {
                        game_context.area_specs.read().boss.then(|| view! { <BossAreaIcon /> })
                    }}
                    <StaticTooltip
                        position=StaticTooltipPosition::Bottom
                        tooltip=move || {
                            let area_level = game_context
                                .area_state
                                .with(|area_state| {
                                    (area_state.area_level as i32)
                                        .saturating_sub(area_state.going_back)
                                        .clamp(1, MAX_AREA_LEVEL as i32) as u16
                                });
                            let power_level = area_level
                                .saturating_add(*game_context.area_specs.read().power_level)
                                .saturating_add(
                                    *game_context.area_specs.read().item_level_modifier,
                                );
                            format!("Area Power Level: {power_level}")
                        }
                    >
                        <div class="flex gap-1 items-center text-lg xl:text-2xl font-bold leading-none  text-shadow-lg/100 shadow-gray-950">
                            <span class="[font-variant:small-caps] font-display">
                                {move || game_context.area_specs.read().name.clone()}
                            </span>
                            " — "
                            {move || {
                                game_context
                                    .area_state
                                    .with(|area_state| {
                                        (area_state.area_level as i32)
                                            .saturating_sub(area_state.going_back)
                                            .clamp(1, MAX_AREA_LEVEL as i32)
                                    })
                            }}
                        </div>
                    </StaticTooltip>
                    {move || {
                        game_context.map_item.get().map(|map_item| view! { <EdictIcon map_item /> })
                    }}
                </div>
            </div>

            <div class="w-24 flex justify-end gap-4 items-center">
                <StaticTooltip
                    position=StaticTooltipPosition::Left
                    tooltip=move || {
                        if disable_rush.get() {
                            view! {
                                <div class="flex flex-col xl:space-y-1 w-[15vw] whitespace-normal">
                                    <div class="font-semibold text-white">"Rush Mode"</div>
                                    <span class="text-zinc-300">
                                        "Unavailable, no Stamina left"
                                    </span>
                                    <span class="text-xs italic text-zinc-400">
                                        "Go Offline while in a Grind to recuperate."
                                    </span>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="flex flex-col xl:space-y-1 w-[15vw] whitespace-normal">
                                    <div class="font-semibold text-white">"Rush Mode"</div>
                                    <span class="text-zinc-300">
                                        {format_duration(game_context.player_stamina.get(), false)}
                                        " Stamina left"
                                    </span>
                                    <span class="text-xs italic text-zinc-400">
                                        "Stamina will be lost when Stopping the Grind."
                                    </span>
                                </div>
                            }
                                .into_any()
                        }
                    }
                >
                    <button
                        class="btn text-xl xl:text-3xl text-amber-300 font-bold
                        drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)]
                        hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)]
                        active:scale-90 active:brightness-125 transition
                        items-center"
                        title="Rush Mode"
                        on:click=toggle_rush_mode
                        class:grayscale=disable_rush
                        disabled=disable_rush
                    >
                        <RushIcon />
                    </button>
                </StaticTooltip>

                <StaticTooltip
                    position=StaticTooltipPosition::Left
                    tooltip=move || {
                        if game_context.area_state.read().auto_progress {
                            "Area Level will increase, click to Pause Progression"
                        } else {
                            "Area Level Progression is Paused, click to Resume Progression"
                        }
                    }
                >
                    <button
                        class="btn text-xl xl:text-3xl text-amber-300 font-bold
                        drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)]
                        hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)]
                        active:scale-90 active:brightness-125 transition
                        items-center"
                        title="Toggle Auto Progress"
                        on:click=toggle_auto_progress
                    >
                        {auto_icon}
                    </button>
                </StaticTooltip>
            </div>
        </div>
    }
}

#[component]
pub fn BattleSceneFooter() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let footer_background = move || {
        format!(
            "background-image: url('{}');",
            img_asset(&game_context.area_specs.read().footer_background)
        )
    };

    let wave_info = move || {
        if game_context.area_state.read().is_boss {
            "Boss".to_string()
        } else {
            format!(
                "Wave: {}/{}",
                game_context.area_state.read().waves_done,
                WAVES_PER_AREA_LEVEL,
            )
        }
    };

    let threat_level = move || game_context.area_threat.read().threat_level;

    view! {
        <div
            class="relative h-8 xl:h-16 w-full z-10
            bg-center bg-repeat-x
            grid grid-cols-[1fr_auto_1fr]
            place-items-center"
            style=footer_background
        >
            <div class="relative flex items-center justify-start h-full w-full z-20">
                <DemonicIncense />
            </div>

            <div class="relative flex items-center justify-center h-full px-2
            gap-2
            text-yellow-500
            text-base xl:text-2xl drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)]">
                <NewAreaLevel />
                <p class="text-shadow-lg/100 shadow-gray-950 text-amber-200 font-bold leading-none whitespace-nowrap">
                    {wave_info}
                </p>
            </div>

            <div class="flex items-center justify-end h-full w-full">
                <div class="relative px-1 py-1 xl:py-2">
                    <div class="absolute inset-0 blur-lg
                    bg-gradient-to-r from-transparent via-zinc-950 via-[percentage:10%_90%] to-transparent"></div>
                    <div class="relative text-shadow-lg/100 shadow-gray-950 text-amber-200 text-base xl:text-2xl font-bold
                    flex items-center gap-1 leading-none">
                        <span>{threat_level}</span>
                        <span class="text-yellow-500">
                            <ThreatIcon />
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn DemonicIncense() -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let conn = expect_context::<WebsocketContext>();

    let can_light = Memo::new(move |_| {
        game_context
            .area_state
            .with(computations::can_light_incense)
    });

    let tooltip = move || {
        let area_state = game_context.area_state.get();

        let description = if game_context.area_specs.read().training {
            "Demonic Incense not available.".to_string()
        } else if area_state.incense_lit {
            "Demonic Incense is lit. The next wave will consist only of Champions, rewarding Gems."
                .to_string()
        } else if area_state.demonic_incense == 0 {
            let mut chance_state = area_state.clone();
            chance_state.area_level = chance_state.max_area_level.saturating_add(1);
            let chance = computations::incense_chance(&chance_state);
            format!(
                "No Demonic Incense available. The next new Area Level has a 1/{:.0} chance to reward one.",
                1.0 / chance
            )
        } else {
            "Demonic Incense is extinguished.".to_string()
        };

        let action_str = if game_context.area_specs.read().training {
            ""
        } else if area_state.incense_lit {
            "Click to Extinguish."
        } else if area_state.is_boss {
            "Cannot be lit on a Boss Level."
        } else if area_state.area_level < area_state.max_area_level {
            "Can only be lit at your highest beaten Area Level or higher."
        } else {
            "Click to Light."
        };

        view! {
            <div class="flex w-72 max-w-[calc(100vw-2rem)] flex-col whitespace-normal break-words text-center text-sm xl:space-y-1">
                <span class="font-semibold text-white">{"Demonic Incense"}</span>
                <span class="text-zinc-300">{description}</span>
                <span class="text-xs italic text-zinc-400">{action_str}</span>
            </div>
        }
    };

    let toggle_incense = move |_| conn.send(&ClientMessage::ToggleIncense);
    let incense_tutorial_completed = RwSignal::new(false);
    Effect::new(move || {
        if game_context.area_state.read().incense_lit {
            incense_tutorial_completed.set(true);
        }
    });
    let show_incense_tutorial = Signal::derive(move || {
        !incense_tutorial_completed.get()
            && !game_context.has_ever_looted_gems.get()
            && game_context.player_resources.read().gems.get() == 0.0
            && game_context.area_state.read().demonic_incense > 0
            && !game_context.area_state.read().incense_lit
    });

    view! {
        <div class="flex items-center h-full">
            <StaticTooltip
                tooltip=tooltip
                position=StaticTooltipPosition::Top
                class="relative h-full w-14  xl:w-24  shrink-0"
            >
                <TutorialPopup
                    show=show_incense_tutorial
                    position=TutorialPopupPosition::AboveLeft
                    class="h-full"
                    message="Light Demonic Incense to make the next wave consist entirely of Champions, which reward Gems."
                >
                    <button
                        class="btn absolute inset-0 font-bold cursor-pointer
                        transition duration-150 hover:brightness-125 active:scale-90 active:brightness-150
                        focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-500
                        disabled:cursor-not-allowed disabled:hover:brightness-100
                        disabled:active:scale-100 disabled:active:brightness-100"
                        class:grayscale=move || game_context.area_state.read().demonic_incense == 0
                        class:brightness-150=move || game_context.area_state.read().incense_lit
                        disabled=move || { !can_light.get() }
                        on:click=toggle_incense
                        title="Demonic Incense"
                        aria-label=move || {
                            if game_context.area_state.read().incense_lit {
                                "Extinguish Demonic Incense"
                            } else {
                                "Light Demonic Incense"
                            }
                        }
                        aria-pressed=move || game_context.area_state.read().incense_lit.to_string()
                    >
                        <img
                            draggable="false"
                            src=move || {
                                if game_context.area_state.read().incense_lit {
                                    img_asset("ui/incense_enabled_hi.webp")
                                } else {
                                    img_asset("ui/incense_disabled_hi.webp")
                                }
                            }
                            alt="Demonic Incense"
                            class="pointer-events-none absolute bottom-1 left-1/2 -translate-x-1/2
                            w-14 xl:w-24 max-w-none object-contain"
                        />
                    </button>
                </TutorialPopup>
            </StaticTooltip>
            <NumberInset>
                <div class="w-[2ch] font-number text-base xl:text-2xl font-bold ">
                    <span class=move || {
                        if game_context.area_state.read().demonic_incense == 0 {
                            "text-zinc-400"
                        } else {
                            "text-red-500"
                        }
                    }>{move || game_context.area_state.read().demonic_incense}</span>
                </div>
            </NumberInset>
        </div>
    }
}

#[component]
pub fn NewAreaLevel() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let new_area_level = Memo::new(move |_| {
        game_context
            .area_state
            .with(|area_state| area_state.area_level > area_state.max_area_level)
    });

    let tooltip = move || {
        if new_area_level.get() {
            "Area Level not yet Cleared, Unique Items & Runes can drop."
        } else {
            "Cleared Area Level, Unique Items & Runes cannot drop."
        }
    };
    view! {
        <StaticTooltip tooltip=tooltip position=StaticTooltipPosition::Top>
            <AreaLevelIcon class:grayscale=move || !new_area_level.get() />
        </StaticTooltip>
    }
}

#[component]
pub fn ThreatMeter() -> impl IntoView {
    let game_context: GameContext = expect_context();

    let threat_increase = Memo::new(move |_| game_context.area_threat.read().just_increased);
    let threat_gain = Memo::new(move |_| game_context.player_specs.with(|x| x.threat_gain.get()));

    let time_remaining = Signal::derive(move || {
        game_context.area_threat.with(|area_threat| {
            let cooldown = area_threat.cooldown.get();
            let threat_gain = threat_gain.get();
            if cooldown > 0.0 && threat_gain > 0.0 {
                (1.0 - area_threat.elapsed_cooldown.get()) * (cooldown / (threat_gain * 0.01))
            } else {
                Default::default()
            }
        })
    });

    let no_threat = Memo::new(move |_| time_remaining.get() == 0.0);

    let reset = Signal::derive(move || threat_increase.get() || no_threat.get());
    let progress_value = predictive_cooldown(time_remaining, reset, no_threat.into(), 0.0);

    view! {
        <StaticTooltip
            position=StaticTooltipPosition::Left
            tooltip=move || {
                if no_threat.get() {
                    "No Threat".to_string()
                } else {
                    format!("Time remaining before next Threat Level: {:.0}s", time_remaining.get())
                }
            }
        >
            <div class="h-full py-1 pr-2 xl:pr-3 z-2">
                <VerticalProgressBar
                    class="z-2 w-4 xl:w-8"
                    value=progress_value
                    reset=threat_increase
                    bar_color="bg-gradient-to-l from-[#9b7429] to-[#d1a24b]"
                />
            </div>
        </StaticTooltip>
    }
}

#[component]
fn RushOverlay() -> impl IntoView {
    let game_context: GameContext = expect_context();

    view! {
        <div class="relative w-full h-full flex items-center justify-center bg-stone-900">
            <div class="absolute inset-0 bg-gradient-to-br from-yellow-500/10 via-transparent to-sky-500/10 animate-pulse" />

            <div class="z-10 flex flex-col items-center gap-4">
                <div class="text-6xl text-yellow-400 animate-pulse">
                    <RushIcon />
                </div>

                <div class=" text-shadow-md/30 shadow-gray-950 text-amber-200 text-base xl:text-2xl font-bold leading-none">
                    "Stamina Left:"
                </div>

                <div class="text-4xl font-bold text-amber-400">
                    {move || format_duration(game_context.player_stamina.get(), false)}
                </div>
            </div>
        </div>
    }
}
