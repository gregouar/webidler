use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{
    computations::{player_level_up_cost, skill_cost_increase},
    messages::client::{
        LevelUpPlayerMessage, LevelUpSkillMessage, SetAutoSkillMessage, UseSkillMessage,
    },
};

use crate::components::{
    events::{EventsContext, Key},
    game::websocket::WebsocketContext,
    shared::{
        skills::{SKILL_PROGRESS_RING_COLOR, SkillProgressBar},
        tooltips::SkillTooltip,
    },
    ui::{
        buttons::{FancyButton, Toggle},
        card::Card,
        number::{Number, format_number},
        progress_bars::{
            CircularProgressBar, HorizontalProgressBar, VerticalProgressBar, predictive_cooldown,
        },
        tooltip::{
            DynamicTooltipPosition, DynamicTooltipTarget, StaticTooltip, StaticTooltipPosition,
        },
    },
};

use super::{GameContext, portrait::CharacterPortrait};

#[component]
pub fn PlayerCard() -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let quantize_ratio = |value: f64| (value.clamp(0.0, 1.0) * 200.0).round() / 200.0;
    let quantize_percent = |value: f64| (value.clamp(0.0, 100.0) * 2.0).round() / 2.0;

    let max_life = Memo::new(move |_| {
        game_context
            .player_specs
            .with(|player_specs| player_specs.character_specs.character_attrs.max_life.get())
    });
    let life = Signal::derive(move || game_context.player_state.read().character_state.life.get());

    let life_tooltip = move || {
        view! {
            "Life: "
            {format_number(life.get())}
            "/"
            {format_number(max_life.get())}
        }
    };

    let life_percent = Signal::derive(move || {
        let max_life = max_life.get();
        if max_life > 0.0 {
            quantize_ratio(life.get() / max_life)
        } else {
            0.0
        }
    });

    let max_mana = Memo::new(move |_| {
        game_context
            .player_specs
            .with(|player_specs| player_specs.character_specs.character_attrs.max_mana.get())
    });
    let reserved_mana = Memo::new(move |_| {
        game_context.player_specs.with(|player_specs| {
            let attrs = &player_specs.character_specs.character_attrs;
            if attrs.take_from_mana_before_life.get() > 0.0
                || attrs.take_from_life_before_mana.get() > 0.0
            {
                0.0
            } else {
                player_specs
                    .character_specs
                    .skills_specs
                    .iter()
                    .map(|s| s.mana_cost.get())
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or_default()
            }
        })
    });
    let mana = Signal::derive(move || game_context.player_state.read().character_state.mana.get());

    let mana_tooltip = move || {
        view! {
            "Mana: "
            {format_number(mana.get())}
            "/"
            {format_number(max_mana.get())}
        }
    };

    let mana_percent = Signal::derive(move || {
        let max_mana = max_mana.get();
        if max_mana > 0.0 {
            quantize_ratio(mana.get() / max_mana)
        } else {
            0.0
        }
    });
    let reserved_mana_percent = Memo::new(move |_| {
        let max_mana = max_mana.get();
        if max_mana > 0.0 {
            quantize_ratio(reserved_mana.get() / max_mana)
        } else {
            0.0
        }
    });
    let is_dead = Memo::new(move |_| !game_context.player_state.read().character_state.is_alive);

    let just_hurt = Memo::new(move |_| game_context.player_state.read().character_state.just_hurt);
    let just_hurt_crit = Memo::new(move |_| {
        game_context
            .player_state
            .read()
            .character_state
            .just_hurt_crit
    });
    let just_blocked = Memo::new(move |_| {
        game_context
            .player_state
            .read()
            .character_state
            .just_blocked
    });
    let just_evaded =
        Memo::new(move |_| game_context.player_state.read().character_state.just_evaded);

    let statuses = Signal::derive(move || {
        game_context
            .player_state
            .read()
            .character_state
            .statuses
            .clone()
    });

    let just_leveled_up = RwSignal::new(false);

    let conn = expect_context::<WebsocketContext>();
    let level_progress = Memo::new(move |_| {
        game_context.player_base_specs.with(|player_base_specs| {
            (
                player_base_specs.level,
                player_base_specs.max_level,
                player_base_specs.experience_needed,
            )
        })
    });
    let max_level = Memo::new(move |_| {
        let (level, max_level, _) = level_progress.get();
        level >= max_level
    });

    let max_xp = Memo::new(move |_| {
        if max_level.get() {
            0.0
        } else {
            level_progress.get().2
        }
    });
    let xp = Memo::new(move |_| game_context.player_resources.read().experience);

    let xp_tooltip = move || {
        if max_level.get() {
            view! { "Max Level" }.into_any()
        } else {
            view! {
                "Experience: "
                {format_number(xp.get())}
                "/"
                {format_number(max_xp.get())}
            }
            .into_any()
        }
    };

    let xp_percent = Signal::derive(move || {
        let max_xp = max_xp.get();
        if max_xp > 0.0 {
            quantize_percent(xp.get() / max_xp * 100.0) as f32
        } else {
            0.0
        }
    });

    let level_up = move |_| {
        game_context.player_base_specs.update(|player_base_specs| {
            game_context.player_resources.write().experience -= player_base_specs.experience_needed;
            player_base_specs.level += 1;
            player_base_specs.experience_needed = player_level_up_cost(player_base_specs);
            just_leveled_up.set(true);
        });
        game_context.player_resources.write().passive_points += 1;

        conn.send(&LevelUpPlayerMessage { amount: 1 }.into());
    };
    let disable_level_up = Memo::new(move |_| {
        max_xp.get() > game_context.player_resources.read().experience || max_level.get()
    });

    let skill_capacity = Memo::new(move |_| {
        let skill_count = game_context
            .player_specs
            .with(|player_specs| player_specs.character_specs.skills_specs.len());
        let max_skills = game_context
            .player_base_specs
            .with(|player_base_specs| player_base_specs.max_skills as usize);
        (skill_count, max_skills)
    });
    let visible_skill_count = Memo::new(move |_| {
        let (skill_count, max_skills) = skill_capacity.get();
        skill_count.min(max_skills)
    });
    let can_buy_skill = Memo::new(move |_| {
        let (skill_count, max_skills) = skill_capacity.get();
        skill_count < max_skills
    });

    // Effect::new({
    //     let toaster = expect_context::<Toasts>();
    //     move || {
    //         if is_dead.get() && just_hurt.get() {
    //             show_toast(
    //                 toaster,
    //                 "You are dead, going back one area level...",
    //                 ToastVariant::Normal,
    //             );
    //         }
    //     }
    // });

    view! {
        <Card class="w-1/3">
            // <div class="max-h-full w-1/3
            // flex flex-col gap-1 xl:gap-2 p-1 xl:p-2
            // bg-zinc-800 ring-1 ring-zinc-950
            // rounded-md shadow-xl/30">

            <PlayerName />

            <div
                class="flex-1 min-h-0 flex justify-around items-stretch gap-1 xl:gap-2"
                style="contain: layout paint;"
            >
                <StaticTooltip tooltip=life_tooltip position=StaticTooltipPosition::Right>
                    <VerticalProgressBar
                        class="w-6 xl:w-8"
                        bar_color="bg-gradient-to-l from-[#6b221d] to-[#c44a3d]"
                        value=life_percent
                    />
                </StaticTooltip>
                <div class="flex flex-col gap-1 xl:gap-2">
                    <div class="flex-1 min-h-0">
                        <CharacterPortrait
                            image_uri=game_context
                                .player_base_specs
                                .read_untracked()
                                .character_static
                                .portrait
                                .clone()
                            character_name="player".to_string()
                            just_hurt=just_hurt
                            just_hurt_crit=just_hurt_crit
                            just_blocked=just_blocked
                            just_evaded=just_evaded
                            is_dead=is_dead
                            statuses=statuses
                        />
                    // enable_blink=false
                    </div>
                    <FancyButton disabled=disable_level_up on:click=level_up>
                        <span class="text-base xl:text-lg">
                            {move || if max_level.get() { "Max Level" } else { "Level Up" }}
                        </span>
                    </FancyButton>
                </div>

                <StaticTooltip tooltip=mana_tooltip position=StaticTooltipPosition::Left>
                    <VerticalProgressBar
                        class="w-6 xl:w-8"
                        bar_color="bg-gradient-to-l from-[#224173] to-[#3f79c2]"
                        value=mana_percent
                    >
                        <div
                            class="h-full w-full origin-bottom"
                            style=move || {
                                format!("transform: scaleY({});", reserved_mana_percent.get())
                            }
                        >
                            <StaticTooltip
                                position=StaticTooltipPosition::Bottom
                                tooltip=move || {
                                    view! {
                                        <div class="flex flex-col xl:space-y-1 text-sm max-w-xs">
                                            <span class="font-semibold text-white">
                                                {"Mana Reserved"}
                                            </span>
                                            <span class="text-zinc-300 text-wrap">
                                                {format!(
                                                    "{} Mana Reserved for Manual Skill Use. This amount of Mana will never be used for Auto Skill Use.",
                                                    reserved_mana.get(),
                                                )}
                                            </span>
                                        </div>
                                    }
                                }
                                class:w-full
                                class:h-full
                            >
                                <div class="w-full h-full bg-[#1b2f52] opacity-55 "></div>
                            </StaticTooltip>
                        </div>
                    </VerticalProgressBar>
                </StaticTooltip>
            </div>

            <StaticTooltip tooltip=xp_tooltip position=StaticTooltipPosition::Top>
                <HorizontalProgressBar
                    class="h-2 xl:h-4"
                    bar_color="bg-gradient-to-b from-neutral-300 to-neutral-500"
                    value=xp_percent
                    reset=just_leveled_up
                />
            </StaticTooltip>

            <div class="flex-none items-center grid grid-cols-4 gap-1 xl:gap-2">
                // style="contain: layout paint;"
                <For each=move || { 0..visible_skill_count.get() } key=|i| *i let(i)>
                    <PlayerSkill index=i is_dead />
                </For>
                <Show when=move || can_buy_skill.get()>
                    <BuySkillButton />
                </Show>
            </div>
        </Card>
    }
}

#[component]
pub fn PlayerName() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let player_name = Memo::new(move |_| {
        game_context
            .player_base_specs
            .read()
            .character_static
            .name
            .clone()
    });

    view! {
        <p class="text-shadow-lg/100 shadow-gray-950 text-amber-200 text-l xl:text-xl font-display">
            <span class="font-bold">
                {player_name} " - " {move || game_context.player_base_specs.read().level}
            </span>
        </p>
    }
}

#[component]
fn BuySkillButton() -> impl IntoView {
    let game_context: GameContext = expect_context();

    let buy_skill_cost = Memo::new(move |_| game_context.player_base_specs.read().buy_skill_cost);

    let disable_buy_skill =
        Memo::new(move |_| buy_skill_cost.get() > game_context.player_resources.read().gold);

    let buy_skill_cost_tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs">
                <span class="font-semibold text-white">{"Buy Cost"}</span>
                <span class="text-zinc-300">
                    <Number class:font-semibold value=buy_skill_cost />
                    " Gold"
                </span>
            </div>
        }
    };

    view! {
        <div class="flex flex-col">
            <StaticTooltip tooltip=buy_skill_cost_tooltip position=StaticTooltipPosition::Top>
                <button
                    class="btn p-1 w-full h-full
                    hover:brightness-125
                    active:brightness-50 active:sepia active:translate-y-[2px]
                    disabled:brightness-75 disabled:saturate-10 disabled:opacity-40
                    "
                    on:click=move |_| game_context.open_skills.set(true)
                    disabled=disable_buy_skill
                >
                    <CircularProgressBar
                        bar_color=SKILL_PROGRESS_RING_COLOR
                        value=Signal::derive(|| 0.0)
                        bar_width=4
                    >
                        <svg
                            width="100%"
                            height="100%"
                            viewBox="0 0 24 24"
                            fill="none"
                            xmlns="http://www.w3.org/2000/svg"
                            class="xl:drop-shadow-[0px_4px_black] text-zinc-300"
                        >
                            <path
                                d="M12 5V19"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                            />
                            <path
                                d="M5 12H19"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                            />
                        </svg>
                    </CircularProgressBar>
                </button>
            </StaticTooltip>

            <div class="flex justify-around invisible">
                <Toggle toggle_callback=|_| {} disabled=Signal::derive(|| true)>
                    <span class="inline xl:hidden">"A"</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Auto"</span>
                </Toggle>
                <FancyButton disabled=Signal::derive(|| true)>
                    <span class="text-base xl:text-2xl">"+"</span>
                </FancyButton>
            </div>
        </div>
    }
}

#[component]
fn PlayerSkill(index: usize, is_dead: Memo<bool>) -> impl IntoView {
    let game_context: GameContext = expect_context();

    let rush_mode = Memo::new(move |_| game_context.area_state.read().rush_mode);

    let skill_specs = Memo::new(move |_| {
        game_context.player_specs.with(|player_specs| {
            player_specs
                .character_specs
                .skills_specs
                .get(index)
                .cloned()
                .map(Arc::new)
        })
    });

    let skill_cooldown = Signal::derive(move || {
        let elapsed_cooldown = game_context.player_state.with(|player_state| {
            player_state
                .character_state
                .skills_states
                .get(index)
                .map(|x| x.elapsed_cooldown.get())
                .unwrap_or_default()
        });
        let cooldown = skill_specs.with(|skill_specs| {
            skill_specs
                .as_ref()
                .map(|skill_specs| skill_specs.cooldown.get())
                .unwrap_or_default()
        });

        (1.0 - elapsed_cooldown) * cooldown
    });

    // TODO: Make dynamic in case of reset?
    let initial_auto_use = *game_context
        .player_auto_skills
        .read_untracked()
        .get(index)
        .unwrap_or(&true);

    let just_triggered = Memo::new(move |_| {
        !rush_mode.get()
            && game_context
                .player_state
                .read()
                .character_state
                .skills_states
                .get(index)
                .map(|x| x.just_triggered)
                .unwrap_or_default()
    });

    let is_ready = Memo::new(move |_| {
        !rush_mode.get()
            && game_context
                .player_state
                .read()
                .character_state
                .skills_states
                .get(index)
                .map(|x| x.is_ready)
                .unwrap_or_default()
    });

    let conn = expect_context::<WebsocketContext>();
    let use_skill = move || {
        conn.send(
            &UseSkillMessage {
                skill_index: index as u8,
            }
            .into(),
        );
    };

    let conn = expect_context::<WebsocketContext>();
    let set_auto_skill = move |value| {
        conn.send(
            &SetAutoSkillMessage {
                skill_index: index as u8,
                auto_use: value,
            }
            .into(),
        );
    };

    let player_base_skill = Memo::new_with_compare(
        move |_| {
            game_context.player_base_specs.with(|player_base_specs| {
                player_base_specs
                    .skills
                    .get_index(index)
                    .map(|(_, player_base_skill)| Arc::new(player_base_skill.clone()))
            })
        },
        |_, _| true,
    );

    let level_up_cost = Memo::new(move |_| {
        player_base_skill.with(|player_base_skill| {
            player_base_skill
                .as_ref()
                .map(|player_base_skill| player_base_skill.next_upgrade_cost)
                .unwrap_or_default()
        })
    });

    let level_up_batch = Memo::new(move |_| {
        let mut total_level = 0u8;
        let mut total_cost = 0.0;
        let gold = game_context.player_resources.read().gold;
        if let Some(player_base_skill) = player_base_skill.get() {
            let mut player_base_skill = (*player_base_skill).clone();
            for _ in 0..10 {
                if total_cost + player_base_skill.next_upgrade_cost > gold {
                    break;
                }
                total_level += 1;
                total_cost += player_base_skill.next_upgrade_cost;
                player_base_skill.upgrade_level += 1;
                player_base_skill.next_upgrade_cost = skill_cost_increase(&player_base_skill);
            }
        }

        (total_level, total_cost)
    });

    let conn = expect_context::<WebsocketContext>();
    let events_context: EventsContext = expect_context();
    let level_up = move |_| {
        let (amount, cost) = if events_context.key_pressed(Key::Ctrl) {
            level_up_batch.get()
        } else {
            (1, level_up_cost.get())
        };

        game_context.player_base_specs.update(|player_base_specs| {
            if let Some((_, player_base_skill)) = player_base_specs.skills.get_index_mut(index) {
                game_context.player_resources.write().gold -= cost;
                for _ in 0..amount {
                    player_base_skill.upgrade_level =
                        player_base_skill.upgrade_level.saturating_add(1);
                    player_base_skill.next_upgrade_cost = skill_cost_increase(player_base_skill);
                }
            }
        });

        conn.send(
            &LevelUpSkillMessage {
                skill_index: index as u8,
                amount,
            }
            .into(),
        );
    };

    let disable_level_up =
        Memo::new(move |_| level_up_cost.get() > game_context.player_resources.read().gold);

    let disabled_auto = Memo::new(move |_| {
        skill_specs.with(|skill_specs| {
            skill_specs
                .as_ref()
                .map(|x| x.cooldown.get())
                .unwrap_or_default()
                == 0.0
        })
    });

    let cost_tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs">
                <span class="font-semibold text-white">{"Upgrade Cost"}</span>
                <span class="text-zinc-300">
                    <Number class:font-semibold value=level_up_cost />
                    " Gold"
                </span>
                <span class="text-xs italic text-gray-400">
                    {format!("Hold CTRL: +{}", level_up_batch.get().0)}
                </span>
            </div>
        }
    };

    let skill_static = Memo::new(move |_| {
        player_base_skill.with(|player_base_skill| {
            player_base_skill.as_ref().map(|player_base_skill| {
                (
                    player_base_skill.base_skill_specs.skill_type,
                    player_base_skill.base_skill_specs.icon.clone(),
                )
            })
        })
    });

    let tooltip = {
        move || {
            view! {
                {skill_specs
                    .get()
                    .map(|skill_specs| {
                        let player_base_skill = player_base_skill.get();
                        view! {
                            <SkillTooltip
                                skill_specs=skill_specs
                                player_base_skill=player_base_skill
                            />
                        }
                    })}
            }
            .into_any()
        }
    };

    let reset_progress =
        Signal::derive(move || just_triggered.get() || is_dead.get() || rush_mode.get());
    let progress_value = predictive_cooldown(
        skill_cooldown,
        reset_progress,
        Signal::derive(move || is_dead.get() || rush_mode.get()),
        game_context
            .player_state
            .read_untracked()
            .character_state
            .skills_states
            .get(index)
            .map(|x| x.elapsed_cooldown.get())
            .unwrap_or_default(),
    );

    view! {
        <div class="flex flex-col">
            <DynamicTooltipTarget content=tooltip position=DynamicTooltipPosition::TopRight>
                {
                    let use_skill = use_skill.clone();
                    view! {
                        <button
                            class="btn p-1 w-full h-full isolate
                            active:brightness-50 active:sepia"
                            on:click=move |_| use_skill()
                            disabled=move || !is_ready.get()
                        >
                            {move || {
                                skill_static
                                    .read()
                                    .as_ref()
                                    .map(|(skill_type, skill_icon)| {
                                        view! {
                                            <SkillProgressBar
                                                skill_type=*skill_type
                                                skill_icon=skill_icon.clone()
                                                value=progress_value
                                                reset=just_triggered
                                                disabled=is_dead
                                                bar_width=4
                                            />
                                        }
                                            .into_any()
                                    })
                            }}
                        </button>
                    }
                }
            </DynamicTooltipTarget>

            <div class="flex justify-around">
                <Toggle
                    toggle_callback=set_auto_skill
                    initial=initial_auto_use
                    disabled=disabled_auto
                >
                    <span class="inline xl:hidden">"A"</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Auto"</span>
                </Toggle>
                <StaticTooltip tooltip=cost_tooltip position=StaticTooltipPosition::Top>
                    <FancyButton disabled=disable_level_up on:click=level_up>
                        <span class="text-base xl:text-2xl">"+"</span>
                    </FancyButton>
                </StaticTooltip>
            </div>
        </div>
    }
}
