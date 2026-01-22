use std::sync::Arc;

use leptos::{html::*, prelude::*};

use rand::Rng;
use shared::data::monster::MonsterRarity;
use shared::data::{character::CharacterSize, monster::MonsterSpecs, skill::SkillSpecs};

use crate::assets::img_asset;
use crate::components::ui::progress_bars::predictive_cooldown;
use crate::components::{
    shared::tooltips::SkillTooltip,
    ui::{
        number::format_number,
        progress_bars::{CircularProgressBar, HorizontalProgressBar},
        tooltip::{
            DynamicTooltipContext, DynamicTooltipPosition, StaticTooltip, StaticTooltipPosition,
        },
    },
};

use super::portrait::CharacterPortrait;
use super::GameContext;

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
            format!(
                "flex-1 min-h-0
                grid grid-rows-2 grid-cols-3 p-1 xl:p-2 gap-1 xl:gap-2 
                items-center
                {} will-change-transform-opacity
                ",
                if all_monsters_dead.get() {
                    "animate-monster-fade-out pointer-events-none"
                } else if flee.get() {
                    "animate-monster-flee pointer-events-none"
                } else if !game_context.monster_states.read().is_empty() {
                    "animate-monster-fade-in"
                } else {
                    ""
                },
            )
        }>
            <For
                each=move || game_context.monster_specs.get().into_iter().enumerate()
                key=move |(index, _)| (game_context.monster_wave.get(), *index)
                children=move |(index, specs)| {
                    let (x_size, y_size) = specs.character_specs.size.get_xy_size();
                    let (x_pos, y_pos) = (
                        specs.character_specs.position_x,
                        specs.character_specs.position_y,
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
            animate-damage-float select-none font-number"
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
fn MonsterCard(specs: MonsterSpecs, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let monster_name = specs.character_specs.name.clone();
    let is_big = match specs.character_specs.size {
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

    let mut damage_tick_id = 0;
    let damage_ticks = ArcRwSignal::new(Vec::new());
    let dot_tick = ArcRwSignal::new(None);

    let mut old_life = specs.character_specs.max_life;
    let life = RwSignal::new(old_life);

    Effect::new({
        let damage_ticks = damage_ticks.clone();
        move || {
            let new_life = game_context
                .monster_states
                .read()
                .get(index)
                .map(|s| s.character_state.life)
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
            {format_number(specs.character_specs.max_life)}
        }
    };

    let life_percent = Memo::new(move |_| {
        let max_life = specs.character_specs.max_life;
        if max_life > 0.0 {
            (life.get() / specs.character_specs.max_life * 100.0) as f32
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
        MonsterRarity::Boss => "boss-title",
    };

    let x_size = specs.character_specs.size.get_xy_size().0;
    let skill_size = if x_size == 1 { "w-full" } else { "w-1/2" };

    view! {
        <style>
            "
            .gold-text {
                font-weight: bold;
                text-shadow: 0 0 8px rgba(255, 223, 0, 0.9);
            }
            
            .gems-text {
                font-weight: bold;
                text-shadow: 0 0 8px rgba(0, 200, 255, 0.9);
            }
            
            .reward-float {
                animation: rewardFloat 2.5s ease-out forwards;
                position: absolute;
            }
            
            @keyframes rewardFloat {
                0% {
                    opacity: 0;
                    transform: translateY(0) scale(0.9);
                }
                20% {
                    opacity: 1;
                    transform: translateY(-12px) scale(1.1);
                }
                40% {
                    opacity: 1;
                    transform: translateY(-24px) scale(1.1);
                }
                100% {
                    opacity: 0;
                    transform: translateY(-64px) scale(1);
                }
            }
            
            .champion-title {                
                font-weight: bold;
                color: #06b6d4;
            }
            
            .boss-title {
                font-weight: bold;
                color: #facc15;
            }
            
            
            @keyframes damage-float {
                0% {
                    opacity: 0;
                    transform: translate(var(--x-offset-start), 0) scale(calc(var(--scale-start) * 0.7)) rotate(0deg);
                }
                10% {
                    opacity: 1;
                    transform: translate(var(--x-offset-start), -5px) scale(var(--scale-start)) rotate(0deg);
                }
                60% {
                    opacity: 1;
                    transform: translate(calc(var(--x-offset-start) + var(--x-offset) * 0.6), calc(var(--y-offset) * 0.6))
                            scale(var(--scale-end))
                            rotate(calc(var(--rotate) * 0.6));
                }
                100% {
                    opacity: 0;
                    transform: translate(calc(var(--x-offset-start) + var(--x-offset)), var(--y-offset))
                            scale(calc(var(--scale-end) * 1.1))
                            rotate(var(--rotate));
                }
            }
            
            .animate-damage-float {
                animation: damage-float var(--duration) cubic-bezier(0.22, 1, 0.36, 1) forwards;
                will-change: transform, opacity;
                filter: saturate(--scale-end);
            }
            "
        </style>
        <div
            class="grid grid-cols-4 h-full
            bg-zinc-800 xl:shadow-lg/30 rounded-md ring-1 ring-zinc-950
            gap-1 xl:gap-2 p-1 xl:p-2"
            style="contain: strict;"
        >
            <div class="relative flex flex-col gap-1 xl:gap-2 col-span-3 h-full min-h-0">
                <StaticTooltip tooltip=life_tooltip position=StaticTooltipPosition::Bottom>
                    <HorizontalProgressBar
                        class=if is_big { "h-5 xl:h-8" } else { "h-4 xl:h-5" }
                        bar_color="bg-gradient-to-b from-red-500 to-red-700"
                        value=life_percent
                    >
                        <span class=title_style>{monster_name}</span>
                    </HorizontalProgressBar>
                </StaticTooltip>

                <div class="flex-1 min-h-0">
                    <CharacterPortrait
                        image_uri=specs.character_specs.portrait.clone()
                        character_name=specs.character_specs.name.clone()
                        rarity=specs.rarity
                        just_hurt=just_hurt
                        just_hurt_crit=just_hurt_crit
                        just_blocked=just_blocked
                        is_dead=is_dead
                        statuses=statuses
                    />
                </div>

                <For each=move || damage_ticks.get() key=|tick| tick.id let(tick)>
                    <DamageNumber tick />
                </For>

                <Show when=move || { gold_reward.get() > 0.0 }>
                    <div class="
                    reward-float gold-text text-amber-400 text:lg xl:text-2xl  text-shadow-md will-change-transform will-change-opacity
                    absolute left-1/2 top-[45%] transform -translate-y-1/2 -translate-x-1/2
                    pointer-events-none z-30 flex items-center gap-1
                    font-number">
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
                    reward-float gems-text text-fuchsia-400 text:lg text-2xl text-shadow-md will-change-transform will-change-opacity
                    absolute left-1/2 top-[65%] transform  -translate-y-1/2 -translate-x-1/2
                    pointer-events-none z-30 flex items-center gap-1
                    font-number">
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

            <div class="w-full flex justify-center">
                <div class=format!("flex flex-col justify-evenly {skill_size}")>
                    <For
                        each=move || { specs.skill_specs.clone().into_iter().enumerate() }
                        key=|(i, _)| *i
                        let((i, p))
                    >
                        <MonsterSkill skill_specs=p index=i monster_index=index />
                    </For>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MonsterSkill(skill_specs: SkillSpecs, index: usize, monster_index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let icon_asset = img_asset(&skill_specs.base.icon);
    let skill_name = skill_specs.base.name.clone();

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
            f32::MAX
        } else {
            (1.0 - game_context
                .monster_states
                .read()
                .get(monster_index)
                .and_then(|m| m.skill_states.get(index))
                .map(|s| s.elapsed_cooldown)
                .unwrap_or(0.0))
                * game_context
                    .monster_specs
                    .read()
                    .get(monster_index)
                    .and_then(|m| m.skill_specs.get(index))
                    .map(|s| s.cooldown)
                    .unwrap_or(0.0) as f32
        }
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let show_tooltip = move || {
        let skill_specs = Arc::new(skill_specs.clone());
        tooltip_context.set_content(
            move || {
                let skill_specs = skill_specs.clone();
                view! { <SkillTooltip skill_specs=skill_specs /> }.into_any()
            },
            DynamicTooltipPosition::Auto,
        );
    };

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let hide_tooltip = move || tooltip_context.hide();

    let just_triggered = Memo::new(move |_| {
        if !is_dead.get() {
            game_context
                .monster_states
                .read()
                .get(monster_index)
                .and_then(|m| m.skill_states.get(index))
                .map(|s| s.just_triggered)
                .unwrap_or_default()
        } else {
            false
        }
    });

    let progress_value = predictive_cooldown(skill_cooldown, just_triggered.into(), is_dead.into());

    view! {
        <CircularProgressBar
            bar_color="oklch(55.5% 0.163 48.998)"
            value=progress_value
            reset=just_triggered
            disabled=is_dead
            bar_width=2

            on:touchstart={
                let show_tooltip = show_tooltip.clone();
                move |_| { show_tooltip() }
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }

            on:mouseenter=move |_| show_tooltip()
            on:mouseleave=move |_| hide_tooltip()
        >
            <img
                draggable="false"
                src=icon_asset
                alt=skill_name
                class="w-full h-full flex-no-shrink fill-current
                xl:drop-shadow-[0px_2px_oklch(13% 0.028 261.692)] invert"
            />
        </CircularProgressBar>
    }
}
