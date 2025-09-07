use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::messages::client::{
    LevelUpPlayerMessage, LevelUpSkillMessage, SetAutoSkillMessage, UseSkillMessage,
};

use crate::assets::img_asset;
use crate::components::{
    ui::{
        buttons::{FancyButton, Toggle},
        number::format_number,
        progress_bars::{CircularProgressBar, HorizontalProgressBar, VerticalProgressBar},
        toast::*,
        tooltip::{
            DynamicTooltipContext, DynamicTooltipPosition, StaticTooltip, StaticTooltipPosition,
        },
    },
    websocket::WebsocketContext,
};

use super::character::CharacterPortrait;
use super::tooltips::SkillTooltip;
use super::GameContext;

#[component]
pub fn PlayerCard() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let max_health = Memo::new(move |_| game_context.player_specs.read().character_specs.max_life);
    let health = Signal::derive(move || game_context.player_state.read().character_state.life);

    let health_tooltip = move || {
        view! {
            "Health: "
            {format_number(health.get())}
            "/"
            {format_number(game_context.player_specs.read().character_specs.max_life)}
        }
    };

    let health_percent = Signal::derive(move || {
        let max_health = max_health.get();
        if max_health > 0.0 {
            (health.get() / max_health * 100.0) as f32
        } else {
            0.0
        }
    });

    let max_mana = Memo::new(move |_| game_context.player_specs.read().character_specs.max_mana);
    let mana = Signal::derive(move || game_context.player_state.read().character_state.mana);

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
            (mana.get() / max_mana * 100.0) as f32
        } else {
            0.0
        }
    });

    let max_xp = Memo::new(move |_| game_context.player_specs.read().experience_needed);
    let xp = Memo::new(move |_| game_context.player_resources.read().experience);

    let xp_tooltip = move || {
        view! {
            "Experience: "
            {format_number(xp.get())}
            "/"
            {format_number(max_xp.get())}
        }
    };

    let xp_percent = Signal::derive(move || {
        let max_xp = max_xp.get();
        if max_xp > 0.0 {
            (xp.get() / max_xp * 100.0) as f32
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

    let statuses = Signal::derive(move || {
        game_context
            .player_state
            .read()
            .character_state
            .statuses
            .clone()
    });

    let conn = expect_context::<WebsocketContext>();
    let level_up = move |_| {
        conn.send(&LevelUpPlayerMessage { amount: 1 }.into());
    };
    let disable_level_up = Memo::new(move |_| {
        game_context.player_specs.read().experience_needed
            > game_context.player_resources.read().experience
    });
    let just_leveled_up = Memo::new(move |_| game_context.player_state.read().just_leveled_up);

    let buy_skill_cost = Memo::new(move |_| game_context.player_specs.read().buy_skill_cost);

    let disable_buy_skill =
        Memo::new(move |_| buy_skill_cost.get() > game_context.player_resources.read().gold);

    let buy_skill_cost_tooltip = move || {
        view! {
            <div class="flex flex-col space-y-1 text-sm max-w-xs">
                <span class="font-semibold text-white">{"Upgrade Cost"}</span>
                <span class="text-zinc-300">
                    {format!("{} Gold", format_number(buy_skill_cost.get()))}
                </span>
            </div>
        }
    };

    Effect::new({
        let toaster = expect_context::<Toasts>();
        move || {
            if is_dead.get() && just_hurt.get() {
                show_toast(
                    toaster,
                    "You are dead, going back one area level...",
                    ToastVariant::Normal,
                );
            }
        }
    });

    view! {
        <style>
            "
            @keyframes player-fade-in {
             0% { transform: translateX(-100%); opacity: 0; }
             65% { transform: translateX(0%); opacity: 1; }
             80% { transform: translateX(-5%); }
             100% { transform: translateX(0%); }
            }
            
            @keyframes player-fade-out {
             from { opacity: 1; transform: translateY(0%); }
             to { opacity: 0; transform: translateY(100%); }
            }
            "
        </style>
        // <div class="overflow-hidden">
        <div class="
        w-full h-full flex flex-col gap-1 lg:gap-2 p-1 lg:gap-2
        bg-zinc-800 
        ring-1 ring-zinc-950
        rounded-md shadow-md 
        ">

            // style=move || {
            // if is_dead.get() {
            // "animation: player-fade-out 3s ease-in; animation-fill-mode: both;"
            // } else {
            // "animation: player-fade-in 1s ease-out; animation-fill-mode: both;"
            // }
            // }
            <div>
                <PlayerName />
            </div>

            <div class="flex flex-col gap-1 lg:gap-2">
                <div class="flex flex-col gap-1 lg:gap-2 items-center">
                    <div class="flex gap-1 lg:gap-2 w-full justify-center">
                        <StaticTooltip tooltip=health_tooltip position=StaticTooltipPosition::Right>
                            <VerticalProgressBar
                                class:w-4
                                class:lg:w-6
                                bar_color="bg-gradient-to-l from-red-500 to-red-700"
                                value=health_percent
                            />
                        </StaticTooltip>
                        <CharacterPortrait
                            image_uri=game_context
                                .player_specs
                                .read_untracked()
                                .character_specs
                                .portrait
                                .clone()
                            character_name="player".to_string()
                            just_hurt=just_hurt
                            just_hurt_crit=just_hurt_crit
                            just_blocked=just_blocked
                            is_dead=is_dead
                            statuses=statuses
                        />
                        <StaticTooltip tooltip=mana_tooltip position=StaticTooltipPosition::Left>
                            <VerticalProgressBar
                                class:w-4
                                class:lg:w-6
                                bar_color="bg-gradient-to-l from-blue-500 to-blue-700"
                                value=mana_percent
                            />
                        </StaticTooltip>
                    </div>
                </div>
                <StaticTooltip tooltip=xp_tooltip position=StaticTooltipPosition::Top>
                    <HorizontalProgressBar
                        class:h-2
                        class:lg:h-4
                        bar_color="bg-gradient-to-b from-neutral-300 to-neutral-500"
                        value=xp_percent
                        reset=just_leveled_up
                    >
                        {}
                    </HorizontalProgressBar>
                </StaticTooltip>
                <FancyButton disabled=disable_level_up on:click=level_up>
                    <span class="text-base lg:text-lg">"Level Up"</span>
                </FancyButton>

            </div>

            <div class="grid grid-cols-4 gap-1 lg:gap-2">
                <For
                    each=move || {
                        0..game_context
                            .player_specs
                            .read()
                            .skills_specs
                            .len()
                            .min(game_context.player_specs.read().max_skills as usize)
                    }
                    key=|i| *i
                    let(i)
                >
                    <PlayerSkill index=i />
                </For>
                <Show when=move || {
                    game_context.player_specs.read().skills_specs.len()
                        < game_context.player_specs.read().max_skills as usize
                }>
                    <div class="flex items-center justify-center">
                        <StaticTooltip
                            tooltip=buy_skill_cost_tooltip
                            position=StaticTooltipPosition::Top
                        >
                            <FancyButton
                                class:aspect-square
                                on:click=move |_| game_context.open_skills.set(true)
                                disabled=disable_buy_skill
                            >
                                "Buy Skill"
                            </FancyButton>
                        </StaticTooltip>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn PlayerName() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let player_name = Memo::new(move |_| {
        game_context
            .player_specs
            .read()
            .character_specs
            .name
            .clone()
    });

    view! {
        <p class="text-shadow-md shadow-gray-950 text-amber-200 text-l lg:text-xl">
            <span class="font-bold">{player_name}</span>
            {move || format!(" — Level: {}", game_context.player_specs.read().level)}
        </p>
    }
}

#[component]
fn PlayerSkill(index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let icon_asset = Memo::new(move |_| {
        if let Some(skill_specs) = game_context.player_specs.read().skills_specs.get(index) {
            img_asset(&skill_specs.base.icon)
        } else {
            "".to_string()
        }
    });

    let skill_name = Memo::new(move |_| {
        game_context
            .player_specs
            .read()
            .skills_specs
            .get(index)
            .map(|x| x.base.name.clone())
            .unwrap_or_default()
    });

    let skill_cooldown = Signal::derive(move || {
        (1.0 - (game_context
            .player_state
            .read()
            .skills_states
            .get(index)
            .map(|x| x.elapsed_cooldown)
            .unwrap_or_default()) as f32)
            * game_context
                .player_specs
                .read()
                .skills_specs
                .get(index)
                .map(|x| x.cooldown)
                .unwrap_or_default()
    });

    // TODO: Make dynamic in case of reset?
    let initial_auto_use = *game_context
        .player_specs
        .read_untracked()
        .auto_skills
        .get(index)
        .unwrap_or(&false);

    let just_triggered = Memo::new(move |_| {
        game_context
            .player_state
            .read()
            .skills_states
            .get(index)
            .map(|x| x.just_triggered)
            .unwrap_or_default()
    });

    let is_ready = Memo::new(move |_| {
        game_context
            .player_state
            .read()
            .skills_states
            .get(index)
            .map(|x| x.is_ready)
            .unwrap_or_default()
    });

    let conn = expect_context::<WebsocketContext>();
    let use_skill = move |_| {
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

    let conn = expect_context::<WebsocketContext>();
    let level_up = move |_| {
        conn.send(
            &LevelUpSkillMessage {
                skill_index: index as u8,
                amount: 1,
            }
            .into(),
        );
    };

    let level_up_cost = Memo::new(move |_| {
        game_context
            .player_specs
            .read()
            .skills_specs
            .get(index)
            .map(|x| x.next_upgrade_cost)
            .unwrap_or_default()
    });

    let disable_level_up =
        Memo::new(move |_| level_up_cost.get() > game_context.player_resources.read().gold);

    let cost_tooltip = move || {
        view! {
            <div class="flex flex-col space-y-1 text-sm max-w-xs">
                <span class="font-semibold text-white">{"Upgrade Cost"}</span>
                <span class="text-zinc-300">
                    {format!("{} Gold", format_number(level_up_cost.get()))}
                </span>
            </div>
        }
    };

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let show_tooltip = move |_| {
        if let Some(skill_specs) = game_context.player_specs.read().skills_specs.get(index) {
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
    let hide_tooltip = move |_| tooltip_context.hide();

    view! {
        <div class="flex flex-col">
            <button
                class="active:brightness-50 active:sepia p-1"
                on:mouseenter=show_tooltip
                on:mouseleave=hide_tooltip
                on:click=use_skill
                disabled=move || !is_ready.get()
            >
                <CircularProgressBar
                    // bar_width=4
                    bar_color="oklch(55.5% 0.163 48.998)"
                    remaining_time=skill_cooldown
                    reset=just_triggered
                    bar_width=4
                >
                    <img
                        src=icon_asset
                        alt=skill_name
                        class="w-full h-full flex-no-shrink fill-current
                        drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert"
                    />
                </CircularProgressBar>
            </button>

            <div class="flex justify-around">
                <Toggle toggle_callback=set_auto_skill initial=initial_auto_use>
                    <span class="inline xl:hidden">"A"</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Auto"</span>
                </Toggle>
                <StaticTooltip tooltip=cost_tooltip position=StaticTooltipPosition::Top>
                    <FancyButton disabled=disable_level_up on:click=level_up>
                        <span class="text-base lg:text-2xl">"+"</span>
                    </FancyButton>
                </StaticTooltip>
            </div>
        </div>
    }
}
