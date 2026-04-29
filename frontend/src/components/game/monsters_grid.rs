use std::collections::HashMap;
use std::sync::Arc;

use leptos::{html::*, prelude::*};

use rand::Rng;
use shared::data::character::CharacterAttrs;
use shared::data::monster::MonsterRarity;
use shared::data::skill::{DamageType, SkillType};
use shared::data::stat_effect::{StatSkillFilter, StatStatusType};
use shared::data::{character::CharacterSize, monster::MonsterSpecs, skill::SkillSpecs};
use strum::IntoEnumIterator;

use crate::assets::img_asset;
use crate::components::icons::monster_tags::{
    ArmorIcon, EvadeIcon, LifeRegenIcon, ResilientIcon, ShieldIcon,
};
use crate::components::settings::{GraphicsQuality, SettingsContext};
use crate::components::shared::skills::SkillProgressBar;
use crate::components::shared::tooltips::effects_tooltip;
use crate::components::shared::tooltips::skill_tooltip::skill_type_str;
use crate::components::ui::progress_bars::predictive_cooldown;
use crate::components::{
    shared::tooltips::SkillTooltip,
    ui::{
        number::format_number,
        progress_bars::HorizontalProgressBar,
        tooltip::{
            DynamicTooltipContext, DynamicTooltipPosition, StaticTooltip, StaticTooltipPosition,
        },
    },
};

use super::GameContext;
use super::portrait::CharacterPortrait;

#[component]
pub fn MonstersGrid() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let all_monsters_dead = RwSignal::new(false);
    let switch_all_monsters_dead = Memo::new(move |_| {
        game_context.monster_states.with(|monster_states| {
            !monster_states.is_empty() && monster_states.iter().all(|x| !x.character_state.is_alive)
        })
    });
    Effect::new(move || {
        if switch_all_monsters_dead.get() {
            // Leave a second for the player to have time to process + play fade in when instant kill
            set_timeout(
                move || {
                    // Repeat in case the state would have change in-between
                    if switch_all_monsters_dead.get_untracked() {
                        all_monsters_dead.set(true);
                    }
                },
                std::time::Duration::from_secs(1),
            );
        } else {
            all_monsters_dead.set(false);
        }
    });

    let flee = Memo::new(move |_| {
        !game_context.player_state.read().character_state.is_alive
            || game_context.area_state.read().going_back > 0
            || game_context.quest_rewards.read().is_some()
    });

    view! {
        <div class=move || {
            let animation_class = if all_monsters_dead.get() {
                "animate-monster-fade-out pointer-events-none"
            } else if flee.get() {
                "animate-monster-flee pointer-events-none"
            } else if !game_context.monster_states.read().is_empty() {
                "animate-monster-fade-in"
            } else {
                ""
            };
            format!(
                "flex-1 min-h-0
                grid grid-rows-2 grid-cols-3 p-1 xl:p-2 gap-1 xl:gap-2
                items-center 
                will-change-transform-opacity transform-gpu
                {}
                ",
                animation_class,
            )
        }>
            <For
                each=move || 0..6
                key=|index| *index
                children=move |index| {
                    view! {
                        <Show when=move || {
                            game_context.monster_specs.read().get(index).is_some()
                        }>
                            {move || {
                                let specs = game_context
                                    .monster_specs
                                    .read()
                                    .get(index)
                                    .cloned()
                                    .expect("checked by Show");
                                let (x_size, y_size) = specs
                                    .character_specs
                                    .character_static
                                    .size
                                    .get_xy_size();
                                let (x_pos, y_pos) = (
                                    specs.character_specs.character_static.position_x,
                                    specs.character_specs.character_static.position_y,
                                );

                                view! {
                                    <div class=format!(
                                        "col-span-{} row-span-{} col-start-{} row-start-{} items-center h-full ",
                                        x_size,
                                        y_size,
                                        x_pos,
                                        y_pos,
                                    )>
                                        <MonsterCard specs=specs index=index />
                                    </div>
                                }
                            }}
                        </Show>
                    }
                }
            />
        </div>
    }
}

// TODO: make a full component to handle all that damages tick thing
#[derive(Clone)]
struct DamageTick {
    pub id: usize,
    pub amount: ArcRwSignal<f64>,
    pub is_crit: bool,
    pub cur_avg_damage: f64,
}

#[component]
fn DamageNumber(tick: DamageTick) -> impl IntoView {
    let mut rng = rand::rng();

    let angle = rng.random_range(-0.4_f32..=0.4_f32);
    let rotate = angle.to_degrees() * 0.8;
    let x_offset_start = rng.random_range(-2..=2);
    let duration = 2.0;

    let amount = tick.amount.clone();
    let style = move || {
        let importance = if tick.cur_avg_damage > 0.0 {
            (amount.get() / tick.cur_avg_damage)
                .powf(1.0 / 3.0)
                .clamp(0.0, 2.0) as f32
        } else {
            1.0
        };
        let font_scale = 0.5 + 0.5 * importance;
        let motion_scale = 1.0 + 0.5 * importance;
        let distance = 2.0 * motion_scale;
        let x_offset = -angle.sin() * distance;
        let y_offset = angle.cos() * distance;
        let scale_start = font_scale * 0.5;
        let scale_end = font_scale;
        format!(
            "--x-offset: {}em; --y-offset: {}em; --rotate: {}deg; --duration: {}s; \
         --scale-start: {}; --scale-end: {}; --x-offset-start: {}em;
         text-shadow: 0px 1px rgba(0, 0, 0, 0.9), 0px 0px 4px rgba(255, 0, 0, 0.5);",
            x_offset, y_offset, rotate, duration, scale_start, scale_end, x_offset_start
        )
    };

    view! {
        <div
            class="absolute left-1/2 top-1 -translate-x-1/2 z-30
            text-red-500 text-shadow-sm font-extrabold text-sm xl:text-lg
            animate-damage-float select-none font-number pointer-events-none"
            style=style
        >
            {move || {
                format!(
                    "{}{}",
                    format_number(tick.amount.get()),
                    if tick.is_crit { "!" } else { "" },
                )
            }}
        </div>
    }
}

#[component]
fn MonsterFeedbackOverlay(
    damage_ticks: ArcRwSignal<Vec<DamageTick>>,
    gold_reward: RwSignal<f64>,
    gems_reward: RwSignal<f64>,
) -> impl IntoView {
    view! {
        <div class="absolute inset-0 z-30 pointer-events-none" style="contain: paint;">
            <For each=move || damage_ticks.get() key=|tick| tick.id let(tick)>
                <DamageNumber tick />
            </For>

            <Show when=move || { gold_reward.get() > 0.0 }>
                <div class="
                reward-float gold-text text-amber-400 text-lg xl:text-2xl text-shadow-md
                absolute left-1/2 top-[45%] transform -translate-y-1/2 -translate-x-1/2
                flex items-center gap-1 font-number">
                    <span>+{format_number(gold_reward.get())}</span>
                    <img
                        draggable="false"
                        src=img_asset("ui/gold.webp")
                        alt="Gold"
                        class="h-[2em] aspect-square"
                    />
                </div>
            </Show>

            <Show when=move || { gems_reward.get() > 0.0 }>
                <div class="
                reward-float gems-text text-fuchsia-400 text-lg text-2xl text-shadow-md
                absolute left-1/2 top-[65%] transform  -translate-y-1/2 -translate-x-1/2
                flex items-center gap-1 font-number">
                    <span>+{format_number(gems_reward.get())}</span>
                    <img
                        draggable="false"
                        src=img_asset("ui/gems.webp")
                        alt="Gems"
                        class="h-[1.2em] aspect-square"
                    />
                </div>
            </Show>
        </div>
    }
}

#[component]
fn MonsterCard(specs: MonsterSpecs, index: usize) -> impl IntoView {
    let game_context: GameContext = expect_context();
    let settings: SettingsContext = expect_context();

    let monster_name = specs.character_specs.character_static.name.clone();
    let is_big = match specs.character_specs.character_static.size {
        CharacterSize::Small | CharacterSize::Large | CharacterSize::Tall => false,
        CharacterSize::Huge | CharacterSize::Gargantuan => true,
    };

    let is_dead = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or_default()
    });

    let just_hurt = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_hurt)
            .unwrap_or_default()
    });

    let just_hurt_crit = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_hurt_crit)
            .unwrap_or_default()
    });

    let just_blocked = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_blocked)
            .unwrap_or_default()
    });

    let just_evaded = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_evaded)
            .unwrap_or_default()
    });

    let mut damage_tick_id = 0;
    let damage_ticks = ArcRwSignal::new(Vec::new());
    let dot_tick = ArcRwSignal::new(None);

    let mut old_life = specs.character_specs.character_attrs.max_life.get();
    let life = RwSignal::new(old_life);

    Effect::new({
        let damage_ticks = damage_ticks.clone();
        move || {
            let new_life = game_context
                .monster_states
                .read()
                .get(index)
                .map(|s| s.character_state.life.get())
                .unwrap_or_default();

            let diff = old_life - new_life;
            old_life = new_life;

            if diff > 0.0 {
                if just_hurt.get_untracked() {
                    let tick_id = damage_tick_id;
                    damage_tick_id += 1;
                    game_context.game_local_stats.add_damage_tick(diff);
                    damage_ticks.write().push(DamageTick {
                        id: tick_id,
                        amount: ArcRwSignal::new(diff),
                        is_crit: just_hurt_crit.get(),
                        cur_avg_damage: game_context.game_local_stats.average_damage_tick(),
                    });

                    set_timeout(
                        {
                            let damage_ticks = damage_ticks.clone();
                            move || {
                                damage_ticks.write().retain(|tick| tick.id != tick_id);
                            }
                        },
                        std::time::Duration::from_secs(3),
                    );
                } else if let Some(dot_tick) = dot_tick.get() {
                    if let Some(tick) = damage_ticks
                        .write()
                        .iter_mut()
                        .find(|tick| tick.id == dot_tick)
                    {
                        *tick.amount.write() += diff
                    }
                } else {
                    let tick_id = damage_tick_id;
                    damage_tick_id += 1;
                    damage_ticks.write().push(DamageTick {
                        id: tick_id,
                        amount: ArcRwSignal::new(diff),
                        is_crit: false,
                        cur_avg_damage: game_context.game_local_stats.average_damage_tick(),
                    });
                    dot_tick.set(Some(tick_id));

                    set_timeout(
                        {
                            let damage_ticks = damage_ticks.clone();
                            let dot_tick = dot_tick.clone();
                            move || {
                                if let Some(amount) = damage_ticks
                                    .read()
                                    .get(tick_id)
                                    .map(|tick: &DamageTick| tick.amount.get_untracked())
                                {
                                    game_context.game_local_stats.add_damage_tick(amount)
                                }
                                dot_tick.set(None);
                            }
                        },
                        std::time::Duration::from_secs(1),
                    );

                    set_timeout(
                        {
                            let damage_ticks = damage_ticks.clone();
                            move || {
                                damage_ticks.write().retain(|tick| tick.id != tick_id);
                            }
                        },
                        std::time::Duration::from_secs(3),
                    );
                }
            }

            if diff != 0.0 {
                life.set(new_life.max(0.0));
            }
        }
    });

    let life_tooltip = move || {
        view! {
            "Life: "
            {format_number(life.get())}
            "/"
            {format_number(specs.character_specs.character_attrs.max_life.get())}
        }
    };

    let life_percent = Memo::new(move |_| {
        let max_life = specs.character_specs.character_attrs.max_life.get();
        if max_life > 0.0 {
            (life.get() / max_life * 100.0) as f32
        } else {
            0.0
        }
    });

    let statuses = Signal::derive(move || {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.statuses.clone())
            .unwrap_or_default()
    });

    let gold_reward = RwSignal::new(0.0);
    let gems_reward = RwSignal::new(0.0);

    Effect::new(move |_| {
        if is_dead.get() {
            let (gold, gems) = game_context
                .monster_states
                .read()
                .get(index)
                .map(|x| (x.gold_reward, x.gems_reward))
                .unwrap_or_default();

            gold_reward.set(gold);

            gems_reward.set(gems);
        }
    });

    let title_style = match specs.rarity {
        MonsterRarity::Normal => "",
        MonsterRarity::Champion => "champion-title",
        MonsterRarity::Boss => "boss-title xl:text-base font-display",
    };

    let x_size = specs.character_specs.character_static.size.get_xy_size().0;
    let skill_size = if x_size == 1 { "w-full" } else { "w-1/2" };

    view! {
        <div
            class=move || {
                format!(
                    "grid grid-cols-4 h-full rounded-md gap-1 xl:gap-2 p-1 xl:p-2 isolate {}",
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "border border-[#6c5734]/45 shadow-[inset_2px_2px_1px_rgba(255,255,255,0.06),inset_-2px_-2px_1px_rgba(0,0,0,0.15)]"
                        }
                        GraphicsQuality::Medium => "border border-[#6c5734]/50",
                        GraphicsQuality::Low => "border border-[#4f4532] bg-zinc-800",
                    },
                )
            }
            style=move || {
                let background_style = match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        format!(
                            "
                            background-image: url('{}');
                            ",
                            img_asset("ui/dark_stone.webp"),
                        )
                    }
                    GraphicsQuality::Medium => {
                        format!(
                            "
                            background-image: url('{}');
                            ",
                            img_asset("ui/dark_stone.webp"),
                        )
                    }
                    GraphicsQuality::Low => "".to_string(),
                };
                format!("contain: layout paint style; {background_style}")
            }
        >
            <div
                class="relative flex flex-col gap-1 xl:gap-2 col-span-3 h-full min-h-0"
                style="contain: layout paint;"
            >
                <StaticTooltip tooltip=life_tooltip position=StaticTooltipPosition::Bottom>
                    <HorizontalProgressBar
                        class=if is_big { "h-6 xl:h-10" } else { "h-4 xl:h-7" }
                        bar_color="bg-gradient-to-b from-[#bc4539] to-[#5d1e19]"
                        value=life_percent
                    >
                        <span class=title_style>{monster_name}</span>
                    </HorizontalProgressBar>
                </StaticTooltip>
                <div class="flex-1 min-h-0">
                    <CharacterPortrait
                        image_uri=specs.character_specs.character_static.portrait.clone()
                        character_name=specs.character_specs.character_static.name.clone()
                        rarity=specs.rarity
                        just_hurt=just_hurt
                        just_hurt_crit=just_hurt_crit
                        just_blocked=just_blocked
                        just_evaded=just_evaded
                        is_dead=is_dead
                        statuses=statuses
                    />
                // enable_blink=true
                </div>
                <MonsterFeedbackOverlay damage_ticks gold_reward gems_reward />
            </div>

            <div class="w-full flex flex-col justify-center gap-1">
                <MonsterTags
                    attrs=specs.character_specs.character_attrs
                    size=specs.character_specs.character_static.size
                />
                <div class=format!(
                    "flex-1 flex flex-col justify-evenly {skill_size} mx-auto",
                )>
                    {specs
                        .character_specs
                        .skills_specs
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| {
                            view! { <MonsterSkill skill_specs=p index=i monster_index=index /> }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn MonsterTags(attrs: CharacterAttrs, size: CharacterSize) -> impl IntoView {
    let armor_values = DamageType::iter()
        .filter_map(|damage_type| {
            let value = *attrs.armor.get(&damage_type).copied().unwrap_or_default();
            (value > 0.0).then_some((damage_type, value))
        })
        .collect::<Vec<_>>();
    let is_armored = !armor_values.is_empty();
    let armored_tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs text-zinc-300">
                <span class="font-semibold text-white">{"Armored"}</span>
                {armor_values
                    .iter()
                    .map(|(damage_type, value)| {
                        view! {
                            <span>
                                <span class="font-semibold">{format!("{:.0}", value)}</span>
                                {format!(
                                    " {}Armor",
                                    effects_tooltip::damage_type_str(Some(*damage_type)),
                                )}
                            </span>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        }
    };

    let shield_values = [SkillType::Attack, SkillType::Spell]
        .into_iter()
        .filter_map(|skill_type| {
            let value = attrs
                .block
                .get(&skill_type)
                .copied()
                .unwrap_or_default()
                .value
                .get();
            (value > 0.0).then_some((skill_type, value))
        })
        .collect::<Vec<_>>();
    let block_damage = attrs.block_damage.get();
    let is_shielded = !shield_values.is_empty();
    let shielded_tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs text-zinc-300">
                <span class="font-semibold text-white">{"Shielded"}</span>
                {shield_values
                    .iter()
                    .map(|(skill_type, value)| {
                        view! {
                            <span>
                                <span class="font-semibold">{format!("{:.0}%", value)}</span>
                                {format!(" {} Block Chance", skill_type_str(Some(*skill_type)))}
                            </span>
                        }
                    })
                    .collect::<Vec<_>>()}
                {(block_damage > 0.0)
                    .then(|| {
                        view! {
                            <span>
                                <span class="font-semibold">{format!("{:.0}%", block_damage)}</span>
                                " Blocked Damage Taken"
                            </span>
                        }
                    })}
            </div>
        }
    };

    let evade_values = DamageType::iter()
        .filter_map(|damage_type| {
            let value = attrs
                .evade
                .get(&damage_type)
                .copied()
                .unwrap_or_default()
                .value
                .get();
            (value > 0.0 && damage_type != DamageType::Storm).then_some((damage_type, value))
        })
        .collect::<Vec<_>>();
    let evade_damage = attrs.evade_damage.get();
    let is_evasive = !evade_values.is_empty();
    let evasive_tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs text-zinc-300">
                <span class="font-semibold text-white">{"Evasive"}</span>
                {evade_values
                    .iter()
                    .map(|(damage_type, value)| {
                        view! {
                            <span>
                                <span class="font-semibold">{format!("{:.0}%", value)}</span>
                                {format!(
                                    " {} Evade Chance",
                                    effects_tooltip::damage_over_time_type_str(Some(*damage_type)),
                                )}
                            </span>
                        }
                    })
                    .collect::<Vec<_>>()}
                {(evade_damage > 0.0)
                    .then(|| {
                        view! {
                            <span>
                                <span class="font-semibold">{format!("{:.0}%", evade_damage)}</span>
                                " Evaded Damage Taken"
                            </span>
                        }
                    })}
            </div>
        }
    };

    let mut grouped: HashMap<Option<StatStatusType>, Vec<(SkillType, f64)>> = HashMap::new();
    for ((skill_type, status_type), value) in attrs.status_resistances.iter() {
        if **value > 0.0 {
            grouped
                .entry(status_type.clone())
                .or_default()
                .push((*skill_type, **value));
        }
    }

    let skill_type_count = SkillType::iter().count();
    let mut resilient_values = Vec::new();
    for (status_type, mut entries) in grouped {
        entries.sort_by_key(|(skill_type, _)| *skill_type);
        if entries.len() == skill_type_count
            && entries
                .iter()
                .map(|(_, v)| *v)
                .collect::<Vec<_>>()
                .windows(2)
                .all(|w| w[0] == w[1])
        {
            resilient_values.push((
                entries[0].1,
                format!(
                    "{} Resilience",
                    effects_tooltip::opt_status_type_str(status_type.as_ref()),
                ),
            ));
        } else {
            resilient_values.extend(entries.into_iter().map(|(skill_type, value)| {
                (
                    value,
                    format!(
                        "{} Resilience",
                        effects_tooltip::skill_status_type_str(
                            &StatSkillFilter {
                                skill_type: Some(skill_type),
                                ..Default::default()
                            },
                            status_type.as_ref(),
                            true,
                        ),
                    ),
                )
            }));
        }
    }

    let is_resilient = !resilient_values.is_empty();
    let resilient_tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs text-zinc-300">
                <span class="font-semibold text-white">{"Resilient"}</span>

                {resilient_values
                    .iter()
                    .map(|(value, label)| {
                        let label = label.clone();
                        view! {
                            <span>
                                <span class="font-semibold">{format!("{:.0}%", value)}</span>
                                {format!(" {label}")}
                            </span>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        }
    };

    let life_regen = *attrs.life_regen * 0.1;
    let is_regenerating = life_regen > 0.0;
    let regenerating_tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs text-zinc-300">
                <span class="font-semibold text-white">{"Regenerating"}</span>
                <span>
                    <span class="font-semibold">{format!("{:.1}%", life_regen)}</span>
                    " Life Regenerated per Second"
                </span>
            </div>
        }
    };

    let grid_size = match size {
        CharacterSize::Small | CharacterSize::Tall => "grid-cols-3",
        CharacterSize::Large | CharacterSize::Huge | CharacterSize::Gargantuan => "grid-cols-6",
    };

    view! {
        <div class=format!(
            "h-5 xl:h-8 grid {grid_size} text-zinc-300",
        )>
            {(is_armored)
                .then(|| {
                    view! {
                        <StaticTooltip
                            tooltip=armored_tooltip
                            position=StaticTooltipPosition::Bottom
                        >
                            <ArmorIcon />
                        </StaticTooltip>
                    }
                })}
            {(is_shielded)
                .then(|| {
                    view! {
                        <StaticTooltip
                            tooltip=shielded_tooltip
                            position=StaticTooltipPosition::Bottom
                        >
                            <ShieldIcon />
                        </StaticTooltip>
                    }
                })}
            {(is_evasive)
                .then(|| {
                    view! {
                        <StaticTooltip
                            tooltip=evasive_tooltip
                            position=StaticTooltipPosition::Bottom
                        >
                            <EvadeIcon />
                        </StaticTooltip>
                    }
                })}
            {(is_resilient)
                .then(|| {
                    view! {
                        <StaticTooltip
                            tooltip=resilient_tooltip
                            position=StaticTooltipPosition::Bottom
                        >
                            <ResilientIcon />
                        </StaticTooltip>
                    }
                })}
            {(is_regenerating)
                .then(|| {
                    view! {
                        <StaticTooltip
                            tooltip=regenerating_tooltip
                            position=StaticTooltipPosition::Bottom
                        >
                            <LifeRegenIcon />
                        </StaticTooltip>
                    }
                })}
        </div>
    }
}

#[component]
fn MonsterSkill(skill_specs: SkillSpecs, index: usize, monster_index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let skill_type = skill_specs.skill_type;
    let skill_icon = skill_specs.icon.clone();
    let skill_specs = Arc::new(skill_specs);

    let is_dead = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(monster_index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or(false)
    });

    let skill_cooldown = Signal::derive(move || {
        if is_dead.get() {
            0.0
        } else if game_context
            .monster_states
            .read()
            .get(monster_index)
            .map(|monster_state| monster_state.character_state.is_stunned())
            .unwrap_or_default()
        {
            f64::MAX
        } else {
            (1.0 - game_context
                .monster_states
                .read()
                .get(monster_index)
                .and_then(|m| m.character_state.skills_states.get(index))
                .map(|s| s.elapsed_cooldown.get())
                .unwrap_or_default())
                * game_context
                    .monster_specs
                    .read()
                    .get(monster_index)
                    .and_then(|m| m.character_specs.skills_specs.get(index))
                    .map(|s| s.cooldown.get())
                    .unwrap_or_default()
        }
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let tooltip_id = RwSignal::new(0);
    let show_tooltip = {
        let skill_specs = skill_specs.clone();
        move || {
            let skill_specs = skill_specs.clone();
            tooltip_id.set(tooltip_context.set_content(
                move || {
                    let skill_specs = skill_specs.clone();
                    view! { <SkillTooltip skill_specs=skill_specs /> }.into_any()
                },
                DynamicTooltipPosition::Auto,
            ));
        }
    };

    let hide_tooltip = move || {
        tooltip_context.hide(tooltip_id.get_untracked());
    };
    on_cleanup(hide_tooltip);

    let just_triggered = Memo::new(move |_| {
        if !is_dead.get() {
            game_context
                .monster_states
                .read()
                .get(monster_index)
                .and_then(|m| m.character_state.skills_states.get(index))
                .map(|s| s.just_triggered)
                .unwrap_or_default()
        } else {
            false
        }
    });

    let progress_value = predictive_cooldown(
        skill_cooldown,
        just_triggered.into(),
        is_dead.into(),
        game_context
            .monster_states
            .read_untracked()
            .get(monster_index)
            .and_then(|m| m.character_state.skills_states.get(index))
            .map(|s| s.elapsed_cooldown.get())
            .unwrap_or_default(),
    );

    view! {
        <SkillProgressBar
            skill_type=skill_type
            skill_icon=skill_icon
            value=progress_value
            reset=just_triggered
            disabled=is_dead
            bar_width=2
            icon_class="w-full h-full flex-no-shrink fill-current invert"

            on:touchstart={
                let show_tooltip = show_tooltip.clone();
                move |_| show_tooltip()
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }

            on:mouseenter=move |_| show_tooltip()
            on:mouseleave=move |_| hide_tooltip()
        />
    }
}
