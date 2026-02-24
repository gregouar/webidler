use leptos::{html::*, prelude::*};
use std::{collections::HashMap, time::Duration};

use shared::data::{
    character_status::{StatusId, StatusMap, StatusSpecs},
    monster::MonsterRarity,
    skill::{DamageType, SkillType},
    stat_effect::{MinMax, StatEffect, StatSkillEffectType, StatStatusType, StatType},
};

use crate::{
    assets::img_asset,
    components::{
        shared::tooltips::{conditions_tooltip, effects_tooltip, trigger_tooltip},
        ui::{
            number::format_number,
            tooltip::{StaticTooltip, StaticTooltipPosition},
        },
    },
};

#[component]
pub fn CharacterPortrait(
    image_uri: String,
    character_name: String,
    #[prop(default = MonsterRarity::Normal)] rarity: MonsterRarity,
    #[prop(into)] just_hurt: Signal<bool>,
    #[prop(into)] just_hurt_crit: Signal<bool>,
    #[prop(into)] just_blocked: Signal<bool>,
    #[prop(into)] just_evaded: Signal<bool>,
    #[prop(into)] is_dead: Signal<bool>,
    #[prop(into)] statuses: Signal<StatusMap>,
) -> impl IntoView {
    let is_dead_img_effect = move || {
        if is_dead.get() {
            "transition-all duration-1000 saturate-0 brightness-50
            [transform:rotateY(180deg)]"
        } else {
            "transition-all duration-1000"
        }
    };

    let crit_hit = RwSignal::new(false);

    Effect::new(move |_| {
        if just_hurt_crit.get() {
            crit_hit.set(true);
            set_timeout(move || crit_hit.set(false), Duration::from_millis(500));
        }
    });

    let crit_animation_style = move || {
        if crit_hit.get() {
            "animation: shake 0.5s linear infinite;"
        } else {
            ""
        }
    };

    let show_block_effect = RwSignal::new(false);

    Effect::new(move |_| {
        if just_blocked.get() {
            show_block_effect.set(true);
        }
    });

    let show_evade_effect = RwSignal::new(false);

    Effect::new(move |_| {
        if just_evaded.get() {
            show_evade_effect.set(true);
        }
    });

    let statuses_map = Signal::derive({
        move || {
            statuses.read().iter().fold(
                HashMap::<StatusId, (usize, f64, Option<StatusSpecs>)>::new(),
                |mut acc, (status_specs, status_state)| {
                    let entry = acc.entry(status_specs.into()).or_default();
                    entry.0 += 1;
                    entry.1 += status_state.value.get();
                    entry.2 = Some(status_specs.clone());
                    acc
                },
            )
        }
    });

    // let active_statuses = Memo::new(move |_| {
    //     let mut active_statuses: Vec<_> = statuses_map.read().keys().cloned().collect();
    //     active_statuses.sort();
    //     active_statuses
    // });

    let active_debuffs = Memo::new(move |_| {
        let mut active_statuses: Vec<_> = statuses_map
            .read()
            .iter()
            .filter_map(|(k, v)| is_debuff(v.2.as_ref()).then_some(k))
            .cloned()
            .collect();
        active_statuses.sort();
        active_statuses
    });

    let active_buffs = Memo::new(move |_| {
        let mut active_statuses: Vec<_> = statuses_map
            .read()
            .iter()
            .filter_map(|(k, v)| (!is_debuff(v.2.as_ref())).then_some(k))
            .cloned()
            .collect();
        active_statuses.sort();
        active_statuses
    });

    let status_stack = move |status_id| {
        statuses_map
            .read()
            .get(&status_id)
            .cloned()
            .unwrap_or_default()
    };

    // let (border_class, shimmer_effect) = match rarity {
    //     MonsterRarity::Normal => ("border-6 xl:border-8 border-double border-stone-500", ""),
    //     MonsterRarity::Champion => (
    //         "border-6 xl:border-8 border-double border-indigo-700",
    //         "champion-shimmer",
    //     ),
    //     MonsterRarity::Boss => (
    //         "border-8 xl:border-12 border-double border-red-700",
    //         "boss-shimmer",
    //     ),
    // };

    let (border_class, shimmer_effect) = match rarity {
        MonsterRarity::Normal => (
            "
            shadow-[0_0_0_2px_#78716c,0_0_0_4px_#1c1917,0_0_0_6px_#78716c]
            xl:shadow-[0_0_0_2px_#78716c,0_0_0_6px_#1c1917,0_0_0_8px_#78716c]
            ",
            "",
        ),

        MonsterRarity::Champion => (
            "
            shadow-[0_0_0_2px_#4338ca,0_0_0_4px_#1e1b4b,0_0_0_6px_#4338ca]
            xl:shadow-[0_0_0_2px_#4338ca,0_0_0_6px_#1e1b4b,0_0_0_8px_#4338ca]
            ",
            "champion-shimmer",
        ),

        MonsterRarity::Boss => (
            "
            shadow-[0_0_0_2px_#b91c1c,0_0_0_4px_#2b0a0a,0_0_0_8px_#b91c1c]
            xl:shadow-[0_0_0_4px_#b91c1c,0_0_0_9px_#2b0a0a,0_0_0_12px_#b91c1c]
            ",
            "boss-shimmer",
        ),
    };

    view! {
        <div class=move || {
            format!(
                "flex items-center justify-center h-full w-full relative p-1 xl:p-2 {}",
                is_dead_img_effect(),
            )
        }>
            <div class=format!("h-full w-full {}", border_class) style=crit_animation_style>
                <div
                    class="h-full w-full relative xl:shadow-[inset_0_0_8px_rgba(0,0,0,0.6)]"
                    style=format!(
                        "background-image: url('{}');",
                        img_asset("ui/paper_background.webp"),
                    )
                >
                    <img
                        draggable="false"
                        src=img_asset(&image_uri)
                        alt=character_name
                        class=move || {
                            format!(
                                "object-cover h-full w-full transition-all duration-[5s] {}",
                                if is_dead.get() { "opacity-50 " } else { "" },
                            )
                        }
                    />

                    <div class="absolute inset-0 flex flex-wrap items-start justify-items-start pointer-events-none">
                        <For
                            each=move || { active_debuffs.get().into_iter() }
                            key=|k| k.clone()
                            let(k)
                        >
                            <StatusIcon
                                status_id=k.clone()
                                stack=Signal::derive({
                                    let k = k.clone();
                                    move || status_stack(k.clone())
                                })
                                tooltip_position=StaticTooltipPosition::Bottom
                            />
                        </For>
                    </div>

                    <div class="absolute inset-0 flex flex-wrap items-end justify-items-start pointer-events-none">
                        <For
                            each=move || { active_buffs.get().into_iter() }
                            key=|k| k.clone()
                            let(k)
                        >
                            <StatusIcon
                                status_id=k.clone()
                                stack=Signal::derive({
                                    let k = k.clone();
                                    move || status_stack(k.clone())
                                })
                                tooltip_position=StaticTooltipPosition::Top
                            />
                        </For>
                    </div>
                </div>

                <div
                    class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-multiply  pointer-events-none"
                    class:opacity-100=move || {
                        active_debuffs
                            .read()
                            .contains(
                                &StatusId::DamageOverTime {
                                    damage_type: DamageType::Physical,
                                },
                            )
                    }
                >
                    <div class="absolute inset-0 status-bleed"></div>
                </div>

                <div
                    class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-color-burn  pointer-events-none"
                    class:opacity-100=move || {
                        active_debuffs
                            .read()
                            .contains(
                                &StatusId::DamageOverTime {
                                    damage_type: DamageType::Fire,
                                },
                            )
                    }
                >
                    <div class="absolute inset-0 status-burn"></div>
                </div>

                <div
                    class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-hard-light  pointer-events-none"
                    class:opacity-100=move || {
                        active_debuffs
                            .read()
                            .contains(
                                &StatusId::DamageOverTime {
                                    damage_type: DamageType::Poison,
                                },
                            )
                    }
                >
                    <div class="absolute inset-0 status-poison"></div>
                </div>

                {move || {
                    (!is_dead.get() && !shimmer_effect.is_empty())
                        .then(|| {
                            view! {
                                <div class=format!("absolute inset-0  {}", shimmer_effect)></div>
                            }
                        })
                }}

                <div
                    class=move || {
                        if just_hurt.get() {
                            "absolute inset-0 pointer-events-none opacity-100 transition-opacity duration-200"
                        } else {
                            "absolute inset-0 pointer-events-none opacity-0 transition-opacity duration-500"
                        }
                    }
                    style="box-shadow: inset 0 0 64px rgba(192, 0, 0, 1.0);"
                ></div>
            </div>

            {move || {
                if show_block_effect.get() {
                    Some(
                        view! {
                            <img
                                draggable="false"
                                src=img_asset("effects/block.svg")
                                class="absolute inset-0 w-object-contain pointer-events-none"
                                on:animationend=move |_| show_block_effect.set(false)
                                style="animation: shield_flash 0.5s ease-out;
                                image-rendering: pixelated; will-change: transform, opacity;
                                "
                            />
                        },
                    )
                } else {
                    None
                }
            }}

            {move || {
                if show_evade_effect.get() {
                    Some(
                        view! {
                            <img
                                draggable="false"
                                src=img_asset("effects/evade.svg")
                                class="absolute inset-0 w-object-contain pointer-events-none"
                                on:animationend=move |_| show_evade_effect.set(false)
                                style="animation: evade_flash 0.5s;
                                image-rendering: pixelated; will-change: transform, opacity;
                                "
                            />
                        },
                    )
                } else {
                    None
                }
            }}
        </div>
    }
}

fn is_debuff(status_specs: Option<&StatusSpecs>) -> bool {
    match status_specs {
        Some(status_specs) => match status_specs {
            StatusSpecs::Stun => true,
            StatusSpecs::DamageOverTime { .. } => true,
            StatusSpecs::StatModifier { debuff, .. } => *debuff,
            StatusSpecs::Trigger(trigger_specs) => {
                matches!(trigger_specs.triggered_effect.skill_type, SkillType::Curse)
            }
        },
        None => false,
    }
}

#[component]
fn StatusIcon(
    status_id: StatusId,
    stack: Signal<(usize, f64, Option<StatusSpecs>)>,
    tooltip_position: StaticTooltipPosition,
) -> impl IntoView {
    let icon_uri = {
        let status_id = status_id.clone();
        move || {
            match status_id.clone() {
                StatusId::Stun => "statuses/stunned.svg".to_string(),
                StatusId::DamageOverTime { damage_type, .. } => match damage_type {
                    DamageType::Physical => "statuses/bleed.svg".to_string(),
                    DamageType::Fire => "statuses/burning.svg".to_string(),
                    DamageType::Poison => "statuses/poison.svg".to_string(),
                    DamageType::Storm => "statuses/storm.svg".to_string(),
                },
                // TODO: More buff types
                StatusId::StatModifier {
                    stat,
                    debuff: false,
                    ..
                } => match stat {
                    StatType::LifeRegen => "passives/life_regen.svg".into(),
                    StatType::Damage {
                        skill_type,
                        damage_type,
                        min_max,
                    } => {
                        if let Some(damage_type) = damage_type {
                            match damage_type {
                                DamageType::Physical => "passives/mace_head.svg".into(),
                                DamageType::Fire => "passives/fire_damage.svg".into(),
                                DamageType::Poison => "passives/scorpion_tail.svg".into(),
                                DamageType::Storm => "passives/storm_damage.svg".into(),
                            }
                        } else if let Some(skill_type) = skill_type {
                            match skill_type {
                                SkillType::Attack => "passives/attack.svg".into(),
                                SkillType::Spell => "passives/spell.svg".into(),
                                _ => "statuses/buff.svg".into(),
                            }
                        } else if let Some(min_max) = min_max {
                            match min_max {
                                MinMax::Min => "statuses/buff.svg".into(),
                                MinMax::Max => "passives/thrust.svg".into(),
                            }
                        } else {
                            "statuses/buff.svg".into()
                        }
                    }
                    StatType::CritChance(_) => "passives/critical_chance.svg".into(),
                    StatType::CritDamage(_) => "passives/critical_damage.svg".into(),
                    StatType::Speed(_) => "passives/sprint.svg".into(),
                    StatType::StatusResistance {
                        status_type: Some(StatStatusType::Stun),
                        ..
                    } => "statuses/stun_immune.svg".into(),
                    StatType::SuccessChance {
                        skill_type,
                        effect_type,
                    } => match (skill_type, effect_type) {
                        (
                            _,
                            Some(StatSkillEffectType::ApplyStatus {
                                status_type:
                                    Some(StatStatusType::DamageOverTime {
                                        damage_type: Some(DamageType::Fire),
                                    }),
                            }),
                        ) => "passives/smoking_finger.svg".into(),
                        _ => "passives/success.svg".into(),
                    },
                    StatType::GoldFind => "passives/gold.svg".to_string(),
                    StatType::Lucky { .. } => "passives/loaded_dice.svg".to_string(),
                    _ => "statuses/buff.svg".to_string(),
                },
                StatusId::StatModifier {
                    stat, debuff: true, ..
                } => match stat {
                    StatType::Armor(_) | StatType::DamageResistance { .. } => {
                        "statuses/debuff_armor.svg".to_string()
                    }
                    StatType::Damage { .. } => "skills/curse_weakness.svg".to_string(),
                    StatType::GoldFind => "statuses/gold_negative.svg".to_string(),
                    _ => "statuses/debuff.svg".to_string(),
                },
                StatusId::Trigger(_) => stack
                    .read()
                    .2
                    .as_ref()
                    .and_then(|status_specs| {
                        if let StatusSpecs::Trigger(trigger_specs) = status_specs {
                            trigger_specs.icon.clone()
                        } else {
                            None
                        }
                    })
                    .unwrap_or("statuses/debuff.svg".into()),
            }
        }
    };

    let description =
        move || status_description(&status_id.clone(), stack.read().1, stack.read().2.clone());

    let tooltip = {
        let description = description.clone();
        move || view! { <span class="max-w-xl">{description.clone()}</span> }
    };

    view! {
        <StaticTooltip position=tooltip_position tooltip>
            <div class="relative h-6 xl:h-12 aspect-square bg-black/40 p-1 pointer-events-auto">
                <img
                    draggable="false"
                    src=move || img_asset(&icon_uri())
                    alt=description
                    class="w-full h-full xl:drop-shadow-md invert"
                />
                <Show when=move || { stack.read().0 > 1 }>
                    <div class="absolute bottom-0 right-0 text-xs font-bold text-white bg-black/50 rounded leading-tight px-1">
                        {move || stack.read().0.to_string()}
                    </div>
                </Show>
            </div>
        </StaticTooltip>
    }
}

pub fn status_description(
    status_id: &StatusId,
    value: f64,
    status_specs: Option<StatusSpecs>,
) -> String {
    match status_id {
        StatusId::Stun => conditions_tooltip::stunned_str(Some(true)).into(),
        StatusId::DamageOverTime { damage_type } => {
            format!(
                "{} for {} Damage per second",
                conditions_tooltip::damaged_over_time_str(Some(*damage_type)),
                format_number(value)
            )
        }
        StatusId::StatModifier {
            stat,
            modifier,
            debuff,
        } => effects_tooltip::format_stat(&StatEffect {
            stat: stat.clone(),
            modifier: *modifier,
            value: if *debuff { -value } else { value },
            bypass_ignore: false,
        }),
        StatusId::Trigger(_) => {
            if let Some(StatusSpecs::Trigger(trigger_specs)) = status_specs {
                trigger_specs.name.clone().unwrap_or(
                    trigger_specs
                        .description
                        .clone()
                        .unwrap_or(trigger_tooltip::trigger_text(*trigger_specs.clone())),
                )
            } else {
                "Special Effect".into()
            }
        }
    }
}
