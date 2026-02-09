use leptos::{html::*, prelude::*};
use std::{collections::HashMap, time::Duration};

use shared::data::{
    character_status::{StatusId, StatusMap},
    monster::MonsterRarity,
    skill::{DamageType, SkillType},
    stat_effect::StatType,
};

use crate::assets::img_asset;

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
                HashMap::<StatusId, usize>::new(),
                |mut acc, (status_specs, _)| {
                    *acc.entry(status_specs.into()).or_default() += 1;
                    acc
                },
            )
        }
    });

    let active_statuses = Memo::new(move |_| {
        let mut active_statuses: Vec<_> = statuses_map.read().keys().cloned().collect();
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

    let (border_class, shimmer_effect) = match rarity {
        MonsterRarity::Normal => ("border-6 xl:border-8 border-double border-stone-500", ""),
        MonsterRarity::Champion => (
            "border-6 xl:border-8 border-double border-indigo-700",
            "champion-shimmer",
        ),
        MonsterRarity::Boss => (
            "border-8 xl:border-12 border-double border-red-700",
            "boss-shimmer",
        ),
    };

    view! {
        <style>
            "
            .champion-shimmer {
                background: linear-gradient(
                    130deg,
                    rgba(255,255,255,0) 40%,
                    rgba(6,182,212,0.35) 50%,
                    rgba(59,130,246,0.35) 60%,
                    rgba(255,255,255,0) 70%
                );
                background-size: 500% 100%;
                background-repeat: repeat;
                animation: shimmerMove 8s infinite linear;
                pointer-events: none;
            }
            
            .boss-shimmer {
                background: linear-gradient(
                    130deg,
                    rgba(255,255,255,0) 40%,
                    rgba(255,50,50,0.4) 50%,
                    rgba(139,0,0,0.4) 60%,
                    rgba(255,255,255,0) 70%
                );
                background-size: 500% 100%;
                background-repeat: repeat;
                animation: shimmerMove 12s infinite linear;
                pointer-events: none;
            }
            
            @keyframes shimmerMove {
                0%   { background-position: -100% 0; }
                100% { background-position: 400% 0; }
            }        
            
            @keyframes shake {
                0%, 100% { transform: translate(0, 0) rotate(0); }
                25% { transform: translate(-4px, 2px) rotate(-2deg); }
                50% { transform: translate(4px, -2px) rotate(2deg); }
                75% { transform: translate(-3px, 1px) rotate(-1deg); }
            }
            
            @keyframes shield_flash {
                0% { opacity: 0; transform: scale(0.35) translateY(60%); }
                50% { opacity: 0.8; transform: scale(.55) translateY(40%); }
                100% { opacity: 0; transform: scale(.65) translateY(60%); }
            }
            
            @keyframes evade_flash {
                0% { opacity: 0; transform: scale(0.35) translateY(100%) translateX(-100%); }
                50% { opacity: 0.8; transform: scale(.55) translateY(60%) translateX(0%); }
                100% { opacity: 0; transform: scale(.65) translateY(100%) translateX(100%); }
            }
            
            /* --- BLEED OVERLAY --- */
            .status-bleed {
                background:
                    linear-gradient(
                        to bottom,
                        rgba(150, 0, 0, 0.8) 0%,
                        rgba(150, 0, 0, 0.4) 15%,
                        rgba(150, 0, 0, 0) 40%
                    );
                animation: bleedPulse 2.5s ease-in-out infinite;
            }
            
            /* subtle pulsing of intensity */
            @keyframes bleedPulse {
                0%, 100% {
                    opacity: 0.5;
                }
                50% {
                    opacity: 1.0;
                }
            }
            
            /* --- BURN OVERLAY --- */
            .status-burn {
                background:
                linear-gradient(
                to right,
                rgba(255, 90, 0, 0.7) 0%,
                rgba(255, 90, 0, 0) 25%,
                rgba(255, 90, 0, 0) 75%,
                rgba(255, 90, 0, 0.7) 100%
                );
                animation: burnFlicker 1.4s ease-in-out infinite;
            }
            
            @keyframes burnFlicker {
                0%   { opacity: 0.7; }
                45%  { opacity: 1.0; }
                100% { opacity: 0.7; }
            }
            
            /* --- POISON OVERLAY --- */
            .status-poison {
                background:
                    linear-gradient(
                        to top,
                        rgba(64, 120, 0, 0.85) 0%,
                        rgba(28, 120, 0, 0.35) 20%,
                        rgba(0, 120, 0, 0) 45%
                    );
                animation: poisonPulse 2.8s ease-in-out infinite;
            }
            
            @keyframes poisonPulse {
                0%, 100% { opacity: 0.4; }
                50%      { opacity: 0.6;   }
            }
            "
        </style>
        <div class=move || {
            format!(
                "flex items-center justify-center h-full w-full relative overflow-hidden {}",
                is_dead_img_effect(),
            )
        }>
            <div class=format!("h-full w-full {}", border_class) style=crit_animation_style>
                <div
                    class="h-full w-full"
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

                </div>

                <div
                    class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-color-burn"
                    class:opacity-100=move || {
                        active_statuses
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
                    class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-hard-light"
                    class:opacity-100=move || {
                        active_statuses
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

                <div
                    class="absolute inset-0 transition-opacity duration-500 opacity-0 mix-blend-multiply"
                    class:opacity-100=move || {
                        active_statuses
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

                <div class="absolute inset-0 flex place-items-start p-1 xl:p-2">
                    <For each=move || active_statuses.get() key=|k| k.clone() let(k)>
                        <StatusIcon
                            status_type=k.clone()
                            stack=Signal::derive(move || status_stack(k.clone()))
                        />
                    </For>
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
                                image-rendering: pixelated; 
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
                                style="animation: evade_flash 0.5s ease-out;
                                image-rendering: pixelated; 
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

#[component]
fn StatusIcon(status_type: StatusId, stack: Signal<usize>) -> impl IntoView {
    let (icon_uri, alt) = match status_type {
        StatusId::Stun => ("statuses/stunned.svg".to_string(), "Stunned"),
        StatusId::DamageOverTime { damage_type, .. } => match damage_type {
            DamageType::Physical => ("statuses/bleed.svg".to_string(), "Bleeding"),
            DamageType::Fire => ("statuses/burning.svg".to_string(), "Burning"),
            DamageType::Poison => ("statuses/poison.svg".to_string(), "Poisoned"),
            DamageType::Storm => ("statuses/storm.svg".to_string(), "Electrocuted"),
        },
        // TODO: More buff types
        StatusId::StatModifier {
            stat,
            debuff: false,
            ..
        } => match stat {
            StatType::Damage {
                skill_type: Some(SkillType::Attack),
                ..
            } => (
                "statuses/buff_attack_damage.svg".to_string(),
                "Increased Attack Damage",
            ),
            _ => ("statuses/buff.svg".to_string(), "Buffed"),
        },
        StatusId::StatModifier {
            stat, debuff: true, ..
        } => match stat {
            StatType::Armor(_) | StatType::DamageResistance { .. } => {
                ("statuses/debuff_armor.svg".to_string(), "Broken Armor")
            }
            StatType::GoldFind => ("statuses/gold_negative.svg".to_string(), "Less Gold"),
            _ => ("statuses/debuff.svg".to_string(), "Debuffed"),
        },
        StatusId::Trigger(trigger_id) => (trigger_id, "Buffed"),
    };
    view! {
        <div class="relative h-6 xl:h-12 aspect-square bg-black/40 p-1">
            <img
                draggable="false"
                src=img_asset(&icon_uri)
                alt=alt
                class="w-full h-full xl:drop-shadow-md invert"
            />
            <Show when=move || { stack.get() > 1 }>
                <div class="absolute bottom-0 right-0 text-xs font-bold text-white bg-black/50 rounded leading-tight px-1">
                    {move || stack.get().to_string()}
                </div>
            </Show>
        </div>
    }
}
