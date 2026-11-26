use leptos::prelude::*;

use shared::data::{
    market::STAT_FILTERS_AMOUNT,
    modifier::Modifier,
    skill::{RestoreType, SkillType},
    stat_effect::{
        DamageStatType, StatSkillEffectType, StatSkillFilter, StatStatusFilter, StatType,
        StatusDamageType,
    },
};

use crate::components::{
    shared::tooltips::effects_tooltip,
    ui::{
        buttons::MenuButton,
        checkbox::Checkbox,
        dropdown::SearchableDropdownMenu,
        input::Input,
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};

pub type StatFilterSignals = [(
    RwSignal<Option<(StatType, Modifier)>>,
    RwSignal<Option<f64>>,
    RwSignal<bool>,
); STAT_FILTERS_AMOUNT];

#[component]
pub fn StatFilters(stat_filters: StatFilterSignals) -> impl IntoView {
    let visible_stat_filters = Memo::new(move |_| {
        crate::components::shared::affix_filters::visible_filter_slots(
            &stat_filters.map(|(stat_type, _, _)| stat_type.get().is_some()),
        )
    });

    view! {
        <div class="flex flex-col gap-2 xl:gap-4 p-2 xl:p-4">
            {stat_filters
                .into_iter()
                .enumerate()
                .map(|(index, (stat_type, stat_value, stat_excluded))| {
                    view! {
                        <Show when=move || index < visible_stat_filters.get()>
                            <div class="flex gap-2 xl:gap-4 items-center">
                                {move || {
                                    stat_type
                                        .read()
                                        .is_some()
                                        .then(|| {
                                            view! {
                                                <MenuButton
                                                    class:flex-none
                                                    on:click=move |_| {
                                                        stat_type.set(None);
                                                        stat_value.set(None);
                                                        stat_excluded.set(false);
                                                    }
                                                >
                                                    "❌"
                                                </MenuButton>
                                            }
                                        })
                                }}
                                <span class=move || {
                                    if stat_type.read().is_none() {
                                        "flex-1 text-center"
                                    } else {
                                        "flex-1"
                                    }
                                }>
                                    <StatDropdown chosen_option=stat_type />
                                </span>
                                {move || {
                                    stat_type
                                        .read()
                                        .is_some()
                                        .then(|| {
                                            view! {
                                                <div class="w-36">
                                                    <Input
                                                        id="stat_value_1"
                                                        input_type="number"
                                                        placeholder="Value"
                                                        bind=stat_value
                                                    />
                                                </div>
                                                <StaticTooltip
                                                    position=StaticTooltipPosition::Top
                                                    tooltip=|| {
                                                        "Require the stat to be absent or below the given value."
                                                    }
                                                >
                                                    <Checkbox
                                                        label="Exclude".to_string()
                                                        checked=stat_excluded
                                                        on_change=move |checked| { stat_excluded.set(checked) }
                                                    />
                                                </StaticTooltip>
                                            }
                                        })
                                }}
                            </div>
                        </Show>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn StatDropdown(chosen_option: RwSignal<Option<(StatType, Modifier)>>) -> impl IntoView {
    let available_stats = vec![
        (StatType::Life, Modifier::Increased),
        (StatType::Life, Modifier::Flat),
        (StatType::LifeRegen, Modifier::Flat),
        (StatType::Mana, Modifier::Increased),
        (StatType::Mana, Modifier::Flat),
        (StatType::ManaRegen, Modifier::Flat),
        (StatType::Armor(Some(DamageStatType::Fire)), Modifier::Flat),
        (
            StatType::Armor(Some(DamageStatType::Poison)),
            Modifier::Flat,
        ),
        (StatType::Armor(Some(DamageStatType::Storm)), Modifier::Flat),
        // Stat is stored split in db/loot queue
        // (
        //     StatType::Armor(Some(DamageStatType::Elemental)),
        //     Modifier::Flat,
        // ),
        (StatType::Armor(None), Modifier::Increased),
        (StatType::Evade(None), Modifier::Flat),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Spell),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Spell),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Physical),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Physical),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Fire),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Fire),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Poison),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Poison),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Storm),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageStatType::Storm),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    skill_id: None,
                },
                damage_type: Some(DamageStatType::Elemental),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::StatusPower {
                status_filter: StatStatusFilter {
                    status_id: None,
                    damage_type: Some(StatusDamageType::Any),
                    debuff: None,
                },
                skill_filter: Default::default(),
                min_max: None,
            },
            Modifier::More,
        ),
        (
            StatType::CritDamage(Default::default()),
            Modifier::Increased,
        ),
        (
            StatType::CritChance(Default::default()),
            Modifier::Increased,
        ),
        (
            StatType::CritChance(StatSkillFilter {
                skill_type: Some(SkillType::Spell),
                ..Default::default()
            }),
            Modifier::Increased,
        ),
        (
            StatType::StatusPower {
                status_filter: Default::default(),
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Curse),
                    ..Default::default()
                },
                min_max: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::StatusDuration {
                status_filter: StatStatusFilter {
                    debuff: Some(true),
                    ..Default::default()
                },
                skill_filter: Default::default(),
            },
            Modifier::Increased,
        ),
        (
            StatType::StatusDuration {
                status_filter: StatStatusFilter {
                    debuff: Some(false),
                    ..Default::default()
                },
                skill_filter: Default::default(),
            },
            Modifier::Increased,
        ),
        (
            StatType::SuccessChance {
                skill_filter: Default::default(),
                effect_type: Some(StatSkillEffectType::ApplyStatus {
                    status_id: None,
                    debuff: Some(true),
                }),
            },
            Modifier::Increased,
        ),
        (
            StatType::Restore {
                restore_type: Some(RestoreType::Life),
                skill_filter: Default::default(),
                modifier_description: Default::default(),
            },
            Modifier::Increased,
        ),
        (StatType::Speed(Default::default()), Modifier::Increased),
        (
            StatType::Speed(StatSkillFilter {
                skill_type: Some(SkillType::Attack),
                ..Default::default()
            }),
            Modifier::Increased,
        ),
        (
            StatType::Speed(StatSkillFilter {
                skill_type: Some(SkillType::Spell),
                ..Default::default()
            }),
            Modifier::Increased,
        ),
        (
            StatType::ManaCost {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Spell),
                    ..Default::default()
                },
            },
            Modifier::Increased,
        ),
        (StatType::MovementSpeed, Modifier::Increased),
        (StatType::GoldFind, Modifier::More),
        (StatType::ThreatGain, Modifier::Increased),
        (
            StatType::RestoreOnHit {
                restore_type: RestoreType::Life,
                skill_type: Some(SkillType::Attack),
            },
            Modifier::Flat,
        ),
        (
            StatType::RestoreOnHit {
                restore_type: RestoreType::Mana,
                skill_type: Some(SkillType::Attack),
            },
            Modifier::Flat,
        ),
        (StatType::SkillLevel(Default::default()), Modifier::Flat),
        (StatType::ItemRarity, Modifier::Increased),
        (StatType::ItemAreaChance, Modifier::Increased),
        (StatType::ItemLevel, Modifier::Flat),
        (StatType::GemsFind, Modifier::Increased),
    ];

    let options = available_stats
        .into_iter()
        .map(|(stat_type, modifier)| {
            (
                Some((stat_type.clone(), modifier)),
                format_stat_filter(&stat_type, modifier, false),
            )
        })
        .collect();

    view! {
        <SearchableDropdownMenu
            options
            chosen_option
            class:w-full
            missing_text="+ Add Stat Filter"
        />
    }
}

pub fn format_stat_filter(stat_type: &StatType, modifier: Modifier, negative: bool) -> String {
    match (modifier, negative) {
        (Modifier::Increased, false) => {
            format!(
                "#% Increased {}",
                effects_tooltip::format_multiplier_stat_name(stat_type)
            )
        }
        (Modifier::Increased, true) => {
            format!(
                "#% Decreased {}",
                effects_tooltip::format_multiplier_stat_name(stat_type)
            )
        }
        (Modifier::More, false) => format!(
            "#% More {}",
            effects_tooltip::format_multiplier_stat_name(stat_type)
        ),
        (Modifier::More, true) => format!(
            "#% Less {}",
            effects_tooltip::format_multiplier_stat_name(stat_type)
        ),
        (Modifier::Flat, _) => effects_tooltip::format_flat_stat(stat_type, None),
    }
}
