use leptos::prelude::*;
use uuid::uuid;

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
        // Unique passive souls
        (
            StatType::AllocatePassive(uuid!("0088c436-0b23-4993-a966-b57d2444e996")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("0bd6d73c-0b10-4524-b280-ca71fdc054c9")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("0bf01c1a-5a2e-4d84-919a-7e4d1ca16fc8")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("17ccb40c-fe56-450b-9723-c12510d0c5c6")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("1bcd296a-d021-4892-9c9d-d1bc4fbb1482")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("214090ea-2db1-429a-b965-382e8a2e78cb")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("26eca27a-95a6-4949-bef7-25cf3b79c340")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("2c872ce5-d750-4a2b-bdc9-ecf51501a7d1")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("36aafa33-ac3d-401d-b4d5-2dbe9a82926d")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("3a4861a5-e8e3-455d-b458-fe23d932b190")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("4102c91f-9037-4b56-b695-e7c4448bd345")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("453ed12d-79ee-421f-aaca-fc21f58dc2d9")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("48348840-47ac-4112-a2a9-699d44fbc8c6")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("4993a125-fdbd-4ee2-a85e-d44f2c00e489")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("5124756b-5066-45d3-a219-3a581d7c8bb7")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("5223c739-dd9e-413d-804a-6b14ea39c1f7")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("56639962-c2b7-4135-868b-07486cbfc109")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("5aa6c4af-db53-4f03-9c7b-ad6888d6545d")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("5f96379e-5249-4b0e-8a4d-1e8c0f951f82")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("656cf6f9-32f1-488e-b188-e81cae806e70")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("665e7afa-d2aa-4075-bd54-278fe4673434")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("6afbaa18-6923-4d9f-af6e-d0edf482b786")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("6c91ed30-7f6b-4748-beab-6a64fc58be18")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("6f7434a6-2cfe-4325-8231-481f3792a47e")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("73c40387-2ca1-4169-9033-ec6e4c7d709f")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("765bb65d-4b0e-4904-9e5e-d100e174a2cd")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("7d81dac1-de3c-4a4d-8e96-7f31ff042e5e")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("7e10ea6e-22f5-43f9-8c49-9df5d664e87a")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("7fc83e8b-550b-47a9-952c-10b7ef15a0cd")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("84587682-6d87-4179-8c45-05ed5e640c38")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("84896a91-0b06-4860-92be-b52a225a0b10")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("91c8b2cf-977c-48ef-9bae-ffb5b20569a7")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("9d418005-cc92-4524-8b96-ad8a7b7ede57")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("a009d872-fa46-498a-af0c-3889e207631b")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("a2beb5ab-9c0c-485f-b5b1-f8aa631c95df")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("a796f363-7d58-4ebd-b78d-8f4cf92614bd")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("abd16719-d299-423a-a184-8cd233dfe87e")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("ad85ce7b-2f6b-4949-af76-6b6cad6ed862")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("b5fce054-f56b-47de-9e4f-d565c54f830c")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("c351443b-35e6-4860-aa43-86eec4c92e51")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("c6d1985a-af8b-4c62-a5f6-8afd826f5457")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("c85b224f-526d-4a9f-a2e4-63fd36cba963")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("c9382d85-ae3d-4efd-9352-79f820fbf726")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("cc2ee934-c6ab-462a-9dd2-74b00a3720af")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("cf14e084-a8ed-496a-b832-b3180ff93a51")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("cfff8b24-fb03-4a7e-a0b1-0c512a2ec7c4")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d1140822-66ab-4fbb-ba98-b1aced236696")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d17092aa-df88-47bd-834f-1233404609da")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d47c084d-d9f2-479a-b1a8-f538845af398")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d6af89ce-d578-44ac-9f1e-acb3df84117b")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("db23ff19-242a-4751-b628-e41b698a0788")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("de4fefa1-145a-4825-8de9-ad5c52140395")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("df82a320-e565-4f56-a224-64f4d960ec6c")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("eb5d66c8-17cc-4812-bcda-c425f0941aea")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("f275006b-3e0a-4177-a128-f501b7b91b4e")),
            Modifier::Flat,
        ),
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
