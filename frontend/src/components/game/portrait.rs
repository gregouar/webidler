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
        settings::{GraphicsQuality, SettingsContext},
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
    // enable_blink: bool,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let is_dead_portrait_effect = move || {
        if is_dead.get() {
            "transition-[filter,opacity] duration-1000 saturate-0 brightness-50"
        } else {
            "transition-[filter,opacity] duration-1000"
        }
    };

    let crit_hit = RwSignal::new(false);

    Effect::new(move |_| {
        if just_hurt_crit.get() && settings.read_settings().shake_on_crit {
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

    let statuses_map = Memo::new({
        move |_| {
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

    let (accent_class, shimmer_effect, fixture_class) = match rarity {
        MonsterRarity::Normal => (
            "
            border-[#7f6744]
            before:border-[#d0b173]/12
            after:border-[#5a4427]/45
            ",
            "",
            "
            border-[#b89458]
            bg-[linear-gradient(180deg,rgb(214,184,126),rgb(111,78,33))]
            ",
        ),

        MonsterRarity::Champion => (
            "
            border-[#4f5fbe]
            before:border-[#97a7ff]/14
            after:border-[#2c356d]/55
            ",
            "champion-shimmer",
            "
            border-[#7a87d8]
            bg-[linear-gradient(180deg,rgb(154,170,255),rgb(57,69,137))]
            ",
        ),

        MonsterRarity::Boss => (
            "
            border-[#ab473c]
            before:border-[#f2a18c]/20
            after:border-[#6d2119]/60
            ",
            "boss-shimmer",
            "
            border-[#d77a68]
            bg-[linear-gradient(180deg,rgb(247,167,145),rgb(116,38,30))]
            ",
        ),
    };

    let portrait_frame_class = move || {
        match settings.graphics_quality() {
        GraphicsQuality::High => format!(
            "w-full h-full relative isolate
            border-[1.5px] xl:border-2
            shadow-[0_6px_12px_rgba(0,0,0,0.34),0_1px_0_rgba(23,15,8,0.82),inset_0_1px_0_rgba(243,221,173,0.12),inset_0_-1px_0_rgba(0,0,0,0.2)]
            before:pointer-events-none before:absolute before:inset-[1px]
            before:border before:bg-[linear-gradient(180deg,rgba(228,194,119,0.06),transparent_28%)]
            after:pointer-events-none after:absolute after:inset-[4px]
            after:border-[1px]
            {}",
            accent_class,
        ),
        GraphicsQuality::Medium => format!(
            "w-full h-full relative isolate
            border-[1.5px] xl:border-2
            before:pointer-events-none before:absolute before:inset-[1px]
            before:border before:bg-[linear-gradient(180deg,rgba(228,194,119,0.045),transparent_32%)]
            after:pointer-events-none after:absolute after:inset-[4px]
            after:border-[1px]
            {}",
            accent_class,
        ),
        GraphicsQuality::Low => format!(
            "w-full h-full relative isolate
            border-[1.5px] xl:border-2
            before:pointer-events-none before:absolute before:inset-[1px]
            before:border before:bg-[linear-gradient(180deg,rgba(186,148,86,0.04),transparent_34%)]
            after:pointer-events-none after:absolute after:inset-[4px]
            after:border-[1px]
            {}",
            accent_class,
        ),
    }
    };

    // let (hit_signal, set_hit_signal) = signal(false);

    // Effect::new(move || {
    //     if just_hurt.get() && enable_blink {
    //         set_hit_signal.set(true);

    //         set_timeout(
    //             move || {
    //                 set_hit_signal.set(false);
    //             },
    //             Duration::from_millis(300),
    //         );
    //     }
    // });

    let activate_bleeding = RwSignal::new(false);
    let is_bleeding = Memo::new(move |_| {
        active_debuffs.read().contains(&StatusId::DamageOverTime {
            damage_type: DamageType::Physical,
        })
    });
    Effect::new(move || {
        if is_bleeding.get() {
            activate_bleeding.set(true)
        }
    });

    let activate_burning = RwSignal::new(false);
    let is_burning = Memo::new(move |_| {
        active_debuffs.read().contains(&StatusId::DamageOverTime {
            damage_type: DamageType::Fire,
        })
    });
    Effect::new(move || {
        if is_burning.get() {
            activate_burning.set(true)
        }
    });

    let activate_poisoned = RwSignal::new(false);
    let is_poisoned = Memo::new(move |_| {
        active_debuffs.read().contains(&StatusId::DamageOverTime {
            damage_type: DamageType::Poison,
        })
    });
    Effect::new(move || {
        if is_poisoned.get() {
            activate_poisoned.set(true)
        }
    });

    view! {
        <div class=move || {
            format!(
                "flex items-center justify-center h-full w-full relative p-1 xl:p-2 {}",
                is_dead_portrait_effect(),
            )
        }>
            <div class=portrait_frame_class style=crit_animation_style>

                // NEW ///

                <Show when=move || settings.uses_heavy_effects()>
                    <div class="pointer-events-none absolute inset-x-6 top-[1px] z-1 h-px bg-gradient-to-r from-transparent via-[#f0d79f]/28 to-transparent"></div>
                // <div class="pointer-events-none absolute inset-0 z-0 bg-[linear-gradient(90deg,rgba(0,0,0,0.12),transparent_12%,transparent_88%,rgba(0,0,0,0.15))]"></div>
                </Show>

                <div
                    class=move || {
                        format!(
                            "h-full z-0 overflow-hidden border border-black/40 bg-[#1c1714] {}",
                            if settings.uses_heavy_effects() {
                                "shadow-[inset_0_1px_0_rgba(255,241,208,0.04),inset_0_0_8px_rgba(0,0,0,0.24)]"
                            } else {
                                ""
                            },
                        )
                    }
                    style=move || {
                        if settings.uses_textures() {
                            format!(
                                "
                                background-image: url('{}');
                                background-size: cover;
                                background-position: center;
                                ",
                                img_asset("ui/paper_background.webp"),
                            )
                        } else {
                            "background-image: linear-gradient(180deg, rgba(227,207,176,0.92), rgba(189,163,121,0.88)); background-color: #e3cfb0;"
                                .to_string()
                        }
                    }
                >
                    <div class="pointer-events-none absolute inset-0 border-[2px] xl:border-[3px] border-[#2a1e19]/68"></div>
                    // <div class="pointer-events-none absolute inset-0 z-1 bg-[radial-gradient(circle_at_50%_15%,rgba(255,241,210,0.04),transparent_34%),linear-gradient(180deg,transparent_68%,rgba(0,0,0,0.14))]"></div>
                    <img
                        draggable="false"
                        src=img_asset(&image_uri)
                        alt=character_name
                        class=move || {
                            format!(
                                "object-cover h-full w-full
                                [transition:transform_1s,opacity_5s]
                                {}
                                {}",
                                if settings.uses_heavy_effects() {
                                    "xl:drop-shadow-[0_10px_15px_rgba(0,0,0,0.5)]"
                                } else {
                                    ""
                                },
                                if is_dead.get() {
                                    "opacity-50 [transform:rotateY(180deg)]"
                                } else {
                                    ""
                                },
                            )
                        }
                    />

                    // /////////
                    // class:hit-blink=hit_signal

                    <div class="absolute inset-0 flex flex-wrap content-start justify-start pointer-events-none">
                        <For
                            each=move || { active_debuffs.get().into_iter() }
                            key=|k| k.clone()
                            let(k)
                        >
                            <StatusIcon
                                status_id=k.clone()
                                stack=Signal::derive({
                                    move || {
                                        statuses_map.read().get(&k).cloned().unwrap_or_default()
                                    }
                                })
                                tooltip_position=StaticTooltipPosition::Bottom
                            />
                        </For>
                    </div>

                    <div class="absolute inset-0 flex flex-wrap-reverse content-start justify-start pointer-events-none">
                        <For
                            each=move || { active_buffs.get().into_iter() }
                            key=|k| k.clone()
                            let(k)
                        >
                            <StatusIcon
                                status_id=k.clone()
                                stack=Signal::derive({
                                    move || {
                                        statuses_map.read().get(&k).cloned().unwrap_or_default()
                                    }
                                })
                                tooltip_position=StaticTooltipPosition::Top
                            />
                        </For>
                    </div>
                </div>

                <div class=move || {
                    format!(
                        "pointer-events-none absolute -top-[5px] -left-[5px] z-2 h-[12px] w-[12px]
                         rotate-315 border {} {}",
                        if settings.uses_heavy_effects() {
                            "shadow-[0_2px_3px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,241,209,1.0)]"
                        } else {
                            ""
                        },
                        fixture_class,
                    )
                }></div>
                <div class=move || {
                    format!(
                        "pointer-events-none absolute -top-[5px] -right-[5px] z-2 h-[12px] w-[12px]
                         rotate-315 border {} {}",
                        if settings.uses_heavy_effects() {
                            "shadow-[0_2px_3px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,241,209,1.0)]"
                        } else {
                            ""
                        },
                        fixture_class,
                    )
                }></div>
                <div class=move || {
                    format!(
                        "pointer-events-none absolute -bottom-[5px] -left-[5px] z-2 h-[12px] w-[12px]
                         rotate-315 border {} {}",
                        if settings.uses_heavy_effects() {
                            "shadow-[0_2px_3px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,241,209,1.0)]"
                        } else {
                            ""
                        },
                        fixture_class,
                    )
                }></div>
                <div class=move || {
                    format!(
                        "pointer-events-none absolute -bottom-[5px] -right-[5px] z-2 h-[12px] w-[12px]
                         rotate-315 border {} {}",
                        if settings.uses_heavy_effects() {
                            "shadow-[0_2px_3px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,241,209,1.0)]"
                        } else {
                            ""
                        },
                        fixture_class,
                    )
                }></div>

                <Show when=move || activate_bleeding.get()>
                    <div
                        class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-multiply  pointer-events-none"
                        class:opacity-100=move || is_bleeding.get()
                    >
                        <div class="absolute inset-0 status-bleed"></div>
                    </div>
                </Show>

                <Show when=move || activate_burning.get()>
                    <div
                        class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-color-burn  pointer-events-none"
                        class:opacity-100=move || is_burning.get()
                    >
                        <div class="absolute inset-0 status-burn"></div>
                    </div>
                </Show>

                <Show when=move || activate_poisoned.get()>
                    <div
                        class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-hard-light  pointer-events-none"
                        class:opacity-100=move || is_poisoned.get()
                    >
                        <div class="absolute inset-0 status-poison"></div>
                    </div>
                </Show>

                {move || {
                    (!is_dead.get() && !shimmer_effect.is_empty()
                        && settings.uses_surface_effects())
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
                                style="animation: shield_flash 0.5s ease-out;"
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
                                style="animation: evade_flash 0.5s;"
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
                trigger_specs.is_debuff
                    || matches!(trigger_specs.triggered_effect.skill_type, SkillType::Curse)
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
            match &status_id {
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
                    StatType::StatusDuration {
                        status_type:
                            Some(StatStatusType::DamageOverTime {
                                damage_type: Some(DamageType::Poison),
                            }),
                        ..
                    } => "passives/ouroboros.svg".into(),
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
                // TODO: More debuff types
                StatusId::StatModifier {
                    stat, debuff: true, ..
                } => match stat {
                    StatType::Armor(_) | StatType::DamageResistance { .. } => {
                        "statuses/debuff_armor.svg".to_string()
                    }
                    StatType::Damage { .. } => "skills/curse_weakness.svg".to_string(),
                    StatType::Speed(_) => "statuses/slow.svg".to_string(),
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

    let description = move || status_description(&status_id, stack.read().1, &stack.read().2);

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
                    class="w-full h-full xl:drop-shadow-sm/80 invert"
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
    status_specs: &Option<StatusSpecs>,
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
