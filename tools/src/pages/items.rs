use std::sync::Arc;

use indexmap::IndexMap;

use leptos::{html::*, prelude::*};

use frontend::components::{
    events::{EventsContext, Key},
    shared::{item_card::ItemCard, tooltips::item_tooltip::name_color_rarity},
    ui::{
        buttons::MenuButton,
        card::{Card, CardInset, CardTitle},
        input::Input,
    },
    utils::file_loader::{save_json, use_json_loader},
};

use leptos_use::{WatchDebouncedOptions, watch_debounced_with_options};
use shared::data::{
    item::{ArmorSpecs, ItemBase, ItemModifiers, ItemSpecs, WeaponSpecs},
    item_affix::{AffixEffect, AffixEffectScope, AffixType, ItemAffix},
    modifier::Modifier,
    skill::{DamageType, SkillType},
    stat_effect::{
        ArmorStatType, LuckyRollType, Matchable, MinMax, StatEffect, StatSkillFilter, StatType,
    },
};
use strum::IntoEnumIterator;

use crate::{header::HeaderMenu, utils::json_editor::JsonEditor};

type ItemsStore = IndexMap<String, ItemBase>;

#[component]
pub fn ItemsPage() -> impl IntoView {
    let events_context: EventsContext = expect_context();

    let items_store = RwSignal::new(Default::default());
    let selected_item = RwSignal::new(None);

    let (loaded_file, filename, on_file) = use_json_loader::<ItemsStore>();

    Effect::new(move || {
        loaded_file.with(|loaded_file| {
            if let Some(loaded_file) = loaded_file {
                selected_item.set(None);
                items_store.set(loaded_file.clone());
            }
        });
    });

    let save = move || {
        save_json(
            &items_store.get(),
            &filename.get().unwrap_or("items.json".into()),
        );
    };

    let file_input: NodeRef<Input> = NodeRef::new();

    let load = move || {
        if let Some(input) = file_input.get() {
            input.click();
        }
    };

    Effect::new({
        move || {
            if events_context.key_pressed(Key::Ctrl) {
                if events_context.key_pressed(Key::Character('s')) {
                    save();
                } else if events_context.key_pressed(Key::Character('o')) {
                    load();
                }
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <input
                node_ref=file_input
                type="file"
                accept="application/json"
                on:change=on_file
                class="hidden"
            />
            <HeaderMenu />
            <div class="relative flex-1">
                <div class="absolute inset-0 flex p-1 xl:p-4 items-center gap-4">
                    <div class="w-6xl h-full">
                        <Card>
                            <div class="flex justify-between mx-4 items-center">
                                <div class="flex flex-row items-center gap-1 xl:gap-2">
                                    <CardTitle>"Items"</CardTitle>
                                </div>

                                <div class="flex gap-2 ml-4">
                                    <MenuButton on:click=move |_| { load() }>"Load"</MenuButton>
                                    <MenuButton on:click=move |_| { save() }>"Save"</MenuButton>
                                </div>

                                <div class="flex-1" />

                                <div class="flex gap-2 ml-4">
                                    <MenuButton on:click=move |_| {}>"Duplicate"</MenuButton>
                                    <MenuButton on:click=move |_| {}>"Add"</MenuButton>
                                </div>

                            </div>
                            <CardInset class:flex-1 class:z-1>
                                <ItemsList items_store selected_item />
                            </CardInset>
                        </Card>
                    </div>

                    <Card class="h-full w-full">
                        <ItemEditor items_store selected_item />
                    </Card>

                    <Card class="h-full w-lg">
                        <ItemPreview items_store selected_item />
                    </Card>

                </div>
            </div>
        </main>
    }
}

#[component]
fn ItemsList(
    items_store: RwSignal<ItemsStore>,
    selected_item: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2">
            <For
                each=move || items_store.get().into_iter()
                key=|(item_id, _)| item_id.clone()
                let((item_id, item_specs))
            >
                <div
                    class={
                        let item_id = item_id.clone();
                        move || {
                            format!(
                                "flex justify-between hover:bg-zinc-700 {}",
                                match selected_item.get() {
                                    Some(selected_item_id) if selected_item_id == item_id => {
                                        "ring-1 ring-amber-500"
                                    }
                                    _ => "",
                                },
                            )
                        }
                    }
                    on:click=move |_| selected_item.set(Some(item_id.clone()))
                >

                    <span>{item_id.clone()}</span>
                    <span class=format!(
                        "font-semibold {}",
                        name_color_rarity(item_specs.rarity),
                    )>{item_specs.name}</span>
                </div>
            </For>
        </div>
    }
}

#[component]
fn ItemEditor(
    items_store: RwSignal<ItemsStore>,
    selected_item: RwSignal<Option<String>>,
) -> impl IntoView {
    let item_base = RwSignal::new(Default::default());

    Effect::new(move || {
        if let Some(selected_item) = selected_item.get()
            && let Some(selected_item_specs) = items_store.read().get(&selected_item)
        {
            item_base.set(selected_item_specs.clone());
        }
    });

    let _ = watch_debounced_with_options(
        move || item_base.get(),
        move |value, _, _| {
            if let Some(item_id) = selected_item.get_untracked()
                && items_store
                    .read_untracked()
                    .get(&item_id)
                    .map(|item_base| *item_base != *value)
                    .unwrap_or_default()
            {
                items_store.write().insert(item_id, value.clone());
            }
        },
        250.0,
        WatchDebouncedOptions::default().immediate(false),
    );

    view! {
        <div class="flex flex-col gap-2">
            <div class="flex justify-between">
                <span>{move || selected_item.get()}</span>

                <MenuButton class:ml-2 on:click=move |_| {}>
                    "❌"
                </MenuButton>
            </div>
            <JsonEditor label="Item" value=item_base h_size="h-160" />
        </div>
    }
}

#[component]
fn ItemPreview(
    items_store: RwSignal<ItemsStore>,
    selected_item: RwSignal<Option<String>>,
) -> impl IntoView {
    let upgrade_level = RwSignal::new(Some(0));

    let item_specs = move || {
        selected_item
            .get()
            .and_then(|selected_item| items_store.read().get(&selected_item).cloned())
            .map(|item_base| {
                let unique_effects = item_base
                    .affixes
                    .iter()
                    .map(|affix| ItemAffix {
                        name: "Unique".to_string(),
                        family: "unique".to_string(),
                        tags: Default::default(),
                        affix_type: AffixType::Unique,
                        tier: 0,
                        effects: Vec::from([AffixEffect {
                            scope: affix.scope,
                            stat_effect: StatEffect {
                                stat: affix.stat.clone(),
                                modifier: affix.modifier,
                                value: affix.value.max,
                                bypass_ignore: false,
                            },
                        }]),
                        item_level: 0,
                    })
                    .collect();
                let modifiers = ItemModifiers {
                    base_item_id: selected_item.get().unwrap_or_default(),
                    name: item_base.name.clone(),
                    rarity: item_base.rarity,
                    level: 999,
                    affixes: unique_effects,
                    quality: 0.0,
                    upgrade_level: upgrade_level.get().unwrap_or_default(),
                };
                Arc::new(create_item_specs(item_base, modifiers, true))
            })
    };
    view! {
        {move || item_specs().map(|item_specs| view! { <ItemCard item_specs /> })}
        <Input
            id="upgrade_level"
            input_type="number"
            placeholder="Empower Level"
            bind=upgrade_level
        />
    }
}

pub fn create_item_specs(
    base: ItemBase,
    mut modifiers: ItemModifiers,
    old_game: bool,
) -> ItemSpecs {
    compute_upgrade_effects(&base, &mut modifiers);

    let effects: Vec<StatEffect> =
        (&modifiers.aggregate_effects(AffixEffectScope::Local, false)).into();

    ItemSpecs {
        required_level: base.min_area_level.max(
            modifiers
                .affixes
                .iter()
                .map(|affix| affix.item_level)
                .max()
                .unwrap_or_default(),
        ),
        weapon_specs: base.weapon_specs.as_ref().map(|weapon_specs| {
            compute_weapon_specs(weapon_specs.clone(), modifiers.quality, &effects)
        }),
        armor_specs: base.armor_specs.as_ref().map(|armor_specs| {
            compute_armor_specs(armor_specs.clone(), modifiers.quality, &effects)
        }),
        base,
        modifiers,
        old_game,
    }
}

fn compute_weapon_specs(
    mut weapon_specs: WeaponSpecs,
    quality: f32,
    effects: &[StatEffect],
) -> WeaponSpecs {
    weapon_specs.damage.values_mut().for_each(|value| {
        value.min.apply_modifier(quality as f64, Modifier::More);
        value.max.apply_modifier(quality as f64, Modifier::More);
    });

    for effect in effects {
        match &effect.stat {
            StatType::Speed(skill_filter)
                if skill_filter.is_match(&StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                }) =>
            {
                weapon_specs.cooldown.apply_negative_effect(effect)
            }
            StatType::Damage {
                skill_filter,
                damage_type,
                min_max,
            } if skill_filter.is_match(&StatSkillFilter {
                skill_type: Some(SkillType::Attack),
                ..Default::default()
            }) =>
            {
                match damage_type {
                    Some(damage_type) => {
                        let value = weapon_specs.damage.entry(*damage_type).or_default();
                        if let Some(MinMax::Min) | None = min_max {
                            value.min.apply_effect(effect);
                        }
                        if let Some(MinMax::Max) | None = min_max {
                            value.max.apply_effect(effect);
                        }
                    }
                    None => {
                        for damage_type in DamageType::iter() {
                            let value = weapon_specs.damage.entry(damage_type).or_default();
                            if let Some(MinMax::Min) | None = min_max {
                                value.min.apply_effect(effect);
                            }
                            if let Some(MinMax::Max) | None = min_max {
                                value.max.apply_effect(effect);
                            }
                        }
                    }
                }
            }
            StatType::CritChance(skill_filter)
                if skill_filter.is_match(&StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                }) =>
            {
                weapon_specs.crit_chance.value.apply_effect(effect)
            }
            StatType::CritDamage(skill_filter)
                if skill_filter.is_match(&StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                }) =>
            {
                weapon_specs.crit_damage.apply_effect(effect)
            }
            StatType::Lucky {
                roll_type: LuckyRollType::CritChance,
                ..
            } => weapon_specs.crit_chance.lucky_chance.apply_effect(effect),

            StatType::Lucky {
                roll_type: LuckyRollType::Damage { damage_type },
                ..
            } => {
                match damage_type {
                    Some(damage_type) => {
                        let value = weapon_specs.damage.entry(*damage_type).or_default();
                        value.lucky_chance.apply_effect(effect);
                    }
                    None => {
                        for value in weapon_specs.damage.values_mut() {
                            value.lucky_chance.apply_effect(effect);
                        }
                    }
                };
            }
            _ => {}
        }
    }

    weapon_specs
}

fn compute_armor_specs(
    mut armor_specs: ArmorSpecs,
    quality: f32,
    effects: &[StatEffect],
) -> ArmorSpecs {
    armor_specs
        .armor
        .apply_modifier(quality as f64, Modifier::More);
    for effect in effects {
        match effect.stat {
            StatType::Armor(Some(ArmorStatType::Physical)) => {
                armor_specs.armor.apply_effect(effect)
            }
            StatType::Block(Some(SkillType::Attack) | None) => {
                armor_specs.block.apply_effect(effect);
            }
            _ => {}
        }
    }

    armor_specs
}

fn compute_upgrade_effects(base: &ItemBase, item_modifiers: &mut ItemModifiers) {
    if item_modifiers.upgrade_level > 0 {
        item_modifiers
            .affixes
            .retain(|affix| !matches!(affix.affix_type, AffixType::Upgrade));

        item_modifiers
            .affixes
            .extend(base.upgrade_effects.iter().cloned().map(|upgrade_effect| {
                ItemAffix {
                    name: "Empowered".into(),
                    family: "empowered".into(),
                    tags: Default::default(),
                    affix_type: AffixType::Upgrade,
                    tier: item_modifiers.upgrade_level,
                    effects: [AffixEffect {
                        scope: upgrade_effect.scope,
                        stat_effect: StatEffect {
                            value: upgrade_effect.stat_effect.value
                                * item_modifiers.upgrade_level as f64,
                            ..upgrade_effect.stat_effect
                        },
                    }]
                    .into(),
                    item_level: base
                        .upgrade_levels
                        .get(item_modifiers.upgrade_level.saturating_sub(1) as usize)
                        .copied()
                        .unwrap_or_default(),
                }
            }));
    }
}
