use leptos::{html::*, prelude::*};

use shared::{
    computations,
    constants::{MAX_AREA_LEVEL, WAVES_PER_AREA_LEVEL},
    messages::client::{GoBackLevelMessage, SetAutoProgressMessage, SetRushModeMessage},
};

use crate::{
    assets::img_asset,
    components::{
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
            number::format_duration,
            progress_bars::{VerticalProgressBar, predictive_cooldown},
            tooltip::{StaticTooltip, StaticTooltipPosition},
        },
    },
};

#[component]
pub fn BattleScene() -> impl IntoView {
    let game_context: GameContext = expect_context();
    view! {
        <div class="absolute inset-0 p-1 xl:p-4">
            <div class="relative w-full max-h-full flex justify-between gap-1 xl:gap-4 ">
                <PlayerCard />
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
            let amount = if events_context.key_pressed(Key::Ctrl) {
                -10
            } else {
                -1
            };
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

    let disable_rush = Memo::new(move |_| game_context.player_stamina.read().is_zero());

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

            {
                let go_forth = go_forth.clone();
                move || {
                    let go_forth = go_forth.clone();
                    game_context
                        .area_specs
                        .read()
                        .training
                        .then(move || {
                            view! {
                                <div class="w-12 flex justify-start">
                                    <StaticTooltip
                                        position=StaticTooltipPosition::Right
                                        tooltip=|| "Go Forth one Area Level"
                                    >
                                        <button
                                            class="btn text-2xl xl:text-4xl text-amber-300 font-bold drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)]
                                            hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)] 
                                            active:scale-90 active:brightness-125 transition"
                                            title="Go Forth One Level"
                                            on:click=go_forth
                                        >
                                            "→"
                                        </button>
                                    </StaticTooltip>
                                </div>
                            }
                        })
                }
            }

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
                                    <span class="text-xs italic text-gray-400">
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
                                    <span class="text-xs italic text-gray-400">
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
            "background-image: url('{}'); contain: layout paint;",
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
            <div class="flex items-center justify-start h-full w-full">
                <div class="relative px-4 py-1 xl:py-2">
                    <div class="absolute inset-0 blur-lg
                    bg-gradient-to-r from-transparent via-zinc-950 via-[percentage:10%_90%] to-transparent"></div>
                    <p class="relative text-shadow-lg/100 shadow-gray-950 text-amber-200 text-base xl:text-2xl font-bold leading-none">
                        {wave_info}
                    </p>
                </div>
            </div>

            <div class="relative flex items-center justify-center h-full px-2
            gap-2
            text-yellow-500
            text-base xl:text-2xl drop-shadow-[0_2px_6px_rgba(0,0,0,0.9)]">
                <GemsLoot />
                <NewAreaLevel />
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
pub fn GemsLoot() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let gems_chance = Memo::new(move |_| game_context.area_state.with(computations::gem_chance));

    let tooltip = move || {
        let gems_chance = gems_chance.get();
        if gems_chance > 0.0 {
            format!(
                "1/{:.0} Chance to find Champion Monster carrying Gems.",
                1.0 / gems_chance
            )
        } else {
            "No more Gems can be found at this Level.".into()
        }
    };
    view! {
        <StaticTooltip tooltip=tooltip position=StaticTooltipPosition::Top>
            <img
                draggable="false"
                src=img_asset("ui/gems.webp")
                alt="Gems Loot"
                class="h-[2em] aspect-square"
                class:grayscale=move || gems_chance.get() == 0.0
            />
        </StaticTooltip>
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
            "New Area Level, Unique Items can drop."
        } else {
            "Cleared Area Level, Unique Items cannot drop."
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
