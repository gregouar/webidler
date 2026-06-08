use leptos::{html::*, prelude::*};
use std::{collections::HashMap, time::Duration};

use shared::data::{
    chance::ChanceRange,
    character_status::{StatusId, StatusMap, StatusSpecs},
    modifier::ModifiableValue,
    monster::MonsterRarity,
    skill::DamageType,
    trigger::TriggersMap,
    values::NonNegative,
};

use crate::{
    assets::img_asset,
    components::{
        data_context::DataContext,
        settings::{GraphicsQuality, SettingsContext},
        shared::tooltips::status_tooltip::format_status_effects,
        ui::tooltip::{StaticTooltip, StaticTooltipPosition},
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
    #[prop(optional)] character_triggers: Option<Memo<TriggersMap>>,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let heavy_effects = move || settings.uses_heavy_effects();
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

    let data_context: DataContext = expect_context();

    let statuses_map = Memo::new(move |_| {
        statuses.read().iter().fold(
            HashMap::<StatusId, (usize, f64)>::new(),
            |mut acc, (status_id, status_states)| {
                let entry = acc.entry(status_id.clone()).or_default();
                entry.0 += status_states.len();
                entry.1 += status_states
                    .iter()
                    .map(|status_state| status_state.value.get())
                    .sum::<f64>();
                acc
            },
        )
    });

    let active_status_ids = Memo::new(move |_| {
        let statuses_specs = data_context.statuses_specs.read();
        let mut active_statuses: Vec<_> = statuses_map
            .read()
            .keys()
            .map(|status_id| (status_id.clone(), statuses_specs.get(status_id).cloned()))
            .collect();
        active_statuses
            .sort_by(|(status_id, _), (other_status_id, _)| status_id.cmp(other_status_id));
        active_statuses
    });

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
            /*shadow-[0_6px_12px_rgba(0,0,0,0.34),0_1px_0_rgba(23,15,8,0.82),inset_0_1px_0_rgba(243,221,173,0.12),inset_0_-1px_0_rgba(0,0,0,0.2)]*/
            GraphicsQuality::High => format!(
                "w-full h-full relative isolate
            border-[1.5px] xl:border-2
            shadow-[0_2px_8px_rgba(0,0,0,0.28)]
            before:pointer-events-none before:absolute before:inset-[1px]
            before:border
            after:pointer-events-none after:absolute after:inset-[4px]
            after:border-[1px]
            {}",
                accent_class,
            ),
            GraphicsQuality::Medium => format!(
                "w-full h-full relative isolate
            border-[1.5px] xl:border-2
            before:pointer-events-none before:absolute before:inset-[1px]
            before:border
            after:pointer-events-none after:absolute after:inset-[4px]
            after:border-[1px]
            {}",
                accent_class,
            ),
            GraphicsQuality::Low => format!(
                "w-full h-full relative isolate
            border-[1.5px] xl:border-2
            before:pointer-events-none before:absolute before:inset-[1px]
            before:border 
            after:pointer-events-none after:absolute after:inset-[4px]
            after:border-[1px]
            {}",
                accent_class,
            ),
        }
    };

    let activate_bleeding = RwSignal::new(false);
    let is_bleeding =
        Memo::new(move |_| has_damage_status(&active_status_ids.read(), DamageType::Physical));
    Effect::new(move || {
        if is_bleeding.get() {
            activate_bleeding.set(true)
        }
    });

    let activate_burning = RwSignal::new(false);
    let is_burning =
        Memo::new(move |_| has_damage_status(&active_status_ids.read(), DamageType::Fire));
    Effect::new(move || {
        if is_burning.get() {
            activate_burning.set(true)
        }
    });

    let activate_poisoned = RwSignal::new(false);
    let is_poisoned =
        Memo::new(move |_| has_damage_status(&active_status_ids.read(), DamageType::Poison));
    Effect::new(move || {
        if is_poisoned.get() {
            activate_poisoned.set(true)
        }
    });

    let dot_overlay_class = move |blend_class: &'static str| {
        if matches!(settings.graphics_quality(), GraphicsQuality::Low) {
            "absolute inset-0 transition-opacity duration-500 opacity-0 pointer-events-none"
                .to_string()
        } else {
            format!(
                "absolute inset-0 transition-opacity duration-500 opacity-0 {} pointer-events-none",
                blend_class,
            )
        }
    };

    let bleed_overlay_class = move || {
        if matches!(settings.graphics_quality(), GraphicsQuality::Low) {
            "absolute inset-0 bg-[linear-gradient(to_bottom,rgba(150,0,0,0.52)_0%,rgba(150,0,0,0.24)_18%,rgba(150,0,0,0)_42%)]"
        } else {
            "absolute inset-0 status-bleed"
        }
    };

    let burn_overlay_class = move || {
        if matches!(settings.graphics_quality(), GraphicsQuality::Low) {
            "absolute inset-0 bg-[linear-gradient(to_right,rgba(255,90,0,0.34)_0%,rgba(255,90,0,0.10)_18%,rgba(255,90,0,0.10)_82%,rgba(255,90,0,0.34)_100%)]"
        } else {
            "absolute inset-0 status-burn"
        }
    };

    let poison_overlay_class = move || {
        if matches!(settings.graphics_quality(), GraphicsQuality::Low) {
            "absolute inset-0 bg-[linear-gradient(to_top,rgba(64,120,0,0.48)_0%,rgba(28,120,0,0.22)_22%,rgba(0,120,0,0)_46%)]"
        } else {
            "absolute inset-0 status-poison"
        }
    };

    view! {
        <div
            class=move || {
                format!(
                    "flex items-center justify-center h-full w-full relative p-1 xl:p-2
                overflow-clip {}",
                    is_dead_portrait_effect(),
                )
            }
            style="contain: layout paint style;"
        >
            <div class=portrait_frame_class style=crit_animation_style>
                <div
                    class=move || {
                        format!(
                            "h-full z-0 overflow-clip border border-black/40 bg-[#1c1714] {}",
                            if heavy_effects() {
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

                    <div
                        class="absolute inset-0 flex flex-wrap content-start justify-start pointer-events-none"
                        style="contain: paint;"
                    >
                        <For
                            each=move || {
                                active_status_ids
                                    .get()
                                    .into_iter()
                                    .filter(|(_, status_specs)| is_debuff(status_specs.as_ref()))
                                    .collect::<Vec<_>>()
                            }
                            key=|(status_id, _)| status_id.clone()
                            let(status)
                        >
                            <StatusIcon
                                status_id=status.0.clone()
                                status_specs=status.1
                                stack=Signal::derive({
                                    let status_id = status.0.clone();
                                    move || {
                                        statuses_map
                                            .read()
                                            .get(&status_id)
                                            .cloned()
                                            .unwrap_or_default()
                                    }
                                })
                                tooltip_position=StaticTooltipPosition::Bottom
                                character_triggers
                            />
                        </For>
                    </div>

                    <div
                        class="absolute inset-0 flex flex-wrap-reverse content-start justify-start pointer-events-none"
                        style="contain: paint;"
                    >
                        <For
                            each=move || {
                                active_status_ids
                                    .get()
                                    .into_iter()
                                    .filter(|(_, status_specs)| !is_debuff(status_specs.as_ref()))
                                    .collect::<Vec<_>>()
                            }
                            key=|(status_id, _)| status_id.clone()
                            let(status)
                        >
                            <StatusIcon
                                status_id=status.0.clone()
                                status_specs=status.1
                                stack=Signal::derive({
                                    let status_id = status.0.clone();
                                    move || {
                                        statuses_map
                                            .read()
                                            .get(&status_id)
                                            .cloned()
                                            .unwrap_or_default()
                                    }
                                })
                                tooltip_position=StaticTooltipPosition::Top
                                character_triggers
                            />
                        </For>
                    </div>
                </div>

                <div class=move || {
                    format!(
                        "pointer-events-none absolute
                         -top-[3px] xl:-top-[5px] -left-[3px] xl:-left-[5px] 
                         z-2 h-[8px] xl:h-[12px] w-[8px] xl:w-[12px]
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
                        "pointer-events-none absolute
                         -top-[3px] xl:-top-[5px] -right-[3px] xl:-right-[5px] 
                         z-2 h-[8px] xl:h-[12px] w-[8px] xl:w-[12px]
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
                        "pointer-events-none absolute
                         -bottom-[3px] xl:-bottom-[5px] -left-[3px] xl:-left-[5px] 
                         z-2 h-[8px] xl:h-[12px] w-[8px] xl:w-[12px]
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
                        "pointer-events-none absolute
                         -bottom-[3px] xl:-bottom-[5px] -right-[3px] xl:-right-[5px] 
                         z-2 h-[8px] xl:h-[12px] w-[8px] xl:w-[12px]
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
                        class=move || dot_overlay_class("mix-blend-multiply")
                        class:opacity-100=move || is_bleeding.get()
                        style="contain: paint;"
                    >
                        <div class=bleed_overlay_class></div>
                    </div>
                </Show>

                <Show when=move || activate_burning.get()>
                    <div
                        class=move || dot_overlay_class("mix-blend-color-burn")
                        class:opacity-100=move || is_burning.get()
                        style="contain: paint;"
                    >
                        <div class=burn_overlay_class></div>
                    </div>
                </Show>

                <Show when=move || activate_poisoned.get()>
                    <div
                        class=move || dot_overlay_class("mix-blend-hard-light")
                        class:opacity-100=move || is_poisoned.get()
                        style="contain: paint;"
                    >
                        <div class=poison_overlay_class></div>
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
    status_specs
        .map(|status_specs| status_specs.debuff)
        .unwrap_or_default()
}

fn has_damage_status(
    active_statuses: &[(StatusId, Option<StatusSpecs>)],
    damage_type: DamageType,
) -> bool {
    active_statuses.iter().any(|(_, status_specs)| {
        status_specs
            .as_ref()
            .and_then(|status_specs| status_specs.damage_type)
            == Some(damage_type)
    })
}

#[component]
fn StatusIcon(
    status_id: StatusId,
    status_specs: Option<StatusSpecs>,
    stack: Signal<(usize, f64)>,
    tooltip_position: StaticTooltipPosition,
    character_triggers: Option<Memo<TriggersMap>>,
) -> impl IntoView {
    let status_name = {
        let status_id = status_id.clone();
        let status_specs = status_specs.clone();
        move || {
            status_specs
                .as_ref()
                .map(|status_specs| status_specs.name.clone())
                .unwrap_or_else(|| status_id.to_string())
        }
    };

    let icon_uri = {
        let status_specs = status_specs.clone();
        move || {
            status_specs
                .as_ref()
                .map(|status_specs| status_specs.icon.clone())
                .unwrap_or("statuses/buff.svg".into())
        }
    };

    let tooltip = {
        let status_id = status_id.clone();
        let status_specs = status_specs.clone();
        move || {
            let (stacks, value) = stack.get();
            match status_specs.clone() {
                Some(status_specs) => {
                    let value = status_value_range(value);
                    view! {
                        <div class="max-w-xl text-center list-none">
                            {format_status_effects(
                                &status_id,
                                status_specs,
                                &value,
                                stacks,
                                None,
                                None,
                                None,
                                character_triggers
                                    .map(|character_triggers| character_triggers.read())
                                    .as_deref(),
                                None,
                                None,
                            )}
                        </div>
                    }
                    .into_any()
                }
                None => view! { <span class="max-w-xl">{status_id.to_string()}</span> }.into_any(),
            }
        }
    };

    view! {
        <StaticTooltip position=tooltip_position tooltip>
            <div class="relative h-6 xl:h-12 aspect-square bg-black/40 p-1 pointer-events-auto">
                <img
                    draggable="false"
                    src=move || img_asset(&icon_uri())
                    alt=status_name
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

fn status_value_range(value: f64) -> ChanceRange<ModifiableValue<NonNegative>> {
    let value: ModifiableValue<NonNegative> = NonNegative::new(value).into();
    ChanceRange {
        min: value,
        max: value,
        lucky_chance: Default::default(),
    }
}
