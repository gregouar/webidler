use std::sync::Arc;

use codee::string::JsonSerdeCodec;
use indexmap::IndexMap;
use leptos::{html::*, prelude::*};

use leptos_use::storage;
use serde::{Deserialize, Serialize};
use shared::{
    data::{
        area::AreaLevel,
        item::{ItemCategory, ItemRarity},
        market::STAT_FILTERS_AMOUNT,
        modifier::Modifier,
        stat_effect::StatType,
    },
    types::ItemName,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::components::{
    game::GameContext,
    shared::inventory::loot_filter_category_to_str,
    town::panels::market::{StatDropdown, item_rarity_str},
    ui::{
        buttons::{FancyButton, MenuButton, Toggle},
        card::{Card, CardHeader, CardInset},
        confirm::ConfirmContext,
        dropdown::{DropdownMenu, SearchableDropdownMenu},
        input::{Input, ValidatedInput},
        menu_panel::MenuPanel,
    },
    utils::file_loader::{save_json, use_json_loader},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LootFilter {
    pub rules: IndexMap<Uuid, FilterRule>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub enum FilterRuleType {
    #[default]
    Pickup,
    Sell,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct FilterRule {
    pub rule_type: FilterRuleType,
    pub rule_name: String,

    pub enabled: bool,

    pub item_name: Option<ItemName>,
    pub req_item_level: Option<AreaLevel>,

    pub item_rarity: Option<ItemRarity>,
    pub item_category: Option<ItemCategory>,

    pub item_damages: Option<f64>,
    pub item_damage_physical: Option<f64>,
    pub item_damage_fire: Option<f64>,
    pub item_damage_poison: Option<f64>,
    pub item_damage_storm: Option<f64>,
    pub item_crit_chance: Option<f64>,
    pub item_crit_damage: Option<f64>,
    pub item_cooldown: Option<f64>,
    pub item_armor: Option<f64>,
    pub item_block: Option<f64>,

    // TODO Cooldown
    pub stat_filters: [Option<((StatType, Modifier), Option<f64>)>; STAT_FILTERS_AMOUNT],
}

impl FilterRule {
    fn new() -> Self {
        Self {
            // rule_id: Uuid::new_v4(),
            rule_type: FilterRuleType::Pickup,
            rule_name: "New Rule".into(),
            enabled: true,
            ..Default::default()
        }
    }
}

#[component]
pub fn LootFilterPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context: GameContext = expect_context();
    let loot_filter = game_context.loot_filter;

    let selected_rule = RwSignal::new(None);

    let new_rule = move || {
        let rule_id = Uuid::new_v4();
        game_context
            .loot_filter
            .write()
            .rules
            .insert(rule_id, FilterRule::new());
        selected_rule.set(Some(rule_id));
    };

    let (get_loot_filter, set_loot_filter, _) =
        storage::use_local_storage::<LootFilter, JsonSerdeCodec>(format!(
            "loot_filter_{}",
            game_context.character_id.get_untracked()
        ));

    Effect::new(move || {
        if !open.get() {
            set_loot_filter.set(loot_filter.get());
        } else {
            loot_filter.set(get_loot_filter.get())
        }
    });

    let (loaded_file, on_change) = use_json_loader::<LootFilter>();
    let file_input: NodeRef<Input> = NodeRef::new();

    Effect::new(move || {
        loaded_file.with(|loaded_file| {
            if let Some(loaded_filter) = loaded_file {
                loot_filter.set(loaded_filter.clone());
            }
        });
    });

    let on_export = move |_| {
        save_json(
            &loot_filter.get_untracked(),
            &format!(
                "loot_filter_{}.json",
                game_context
                    .player_specs
                    .read_untracked()
                    .character_specs
                    .name
            ),
        );
    };

    let on_import = move |_| {
        if let Some(input) = file_input.get() {
            input.click();
        }
    };

    view! {
        <input
            node_ref=file_input
            type="file"
            accept="application/json"
            on:change=on_change
            class="hidden"
        />
        <MenuPanel open=open class:items-center>
            <Card class="w-full h-full">
                <CardHeader title="Loot Filter" on_close=move || open.set(false)>
                    <div class="flex gap-2 mx-4">
                        <MenuButton on:click=move |_| new_rule()>"New Rule"</MenuButton>
                    </div>

                    <div class="flex-1" />

                    <div class="flex gap-2 mx-4">
                        <MenuButton on:click=on_import>"Import"</MenuButton>
                        <MenuButton on:click=on_export>"Export"</MenuButton>
                    </div>
                </CardHeader>
                <div class="grid grid-cols-2 gap-2 min-h-0 flex-1">
                    <RulesList loot_filter selected_rule />
                    <EditRule loot_filter selected_rule />
                </div>
            </Card>
        </MenuPanel>
    }
}

#[component]
pub fn RulesList(
    loot_filter: RwSignal<LootFilter>,
    selected_rule: RwSignal<Option<Uuid>>,
) -> impl IntoView {
    view! {
        <CardInset>
            <div class="gap-2 flex flex-col">
                <For
                    each=move || {
                        let keys = loot_filter.read().rules.keys().copied().collect::<Vec<_>>();
                        keys.into_iter()
                    }
                    key=|rule_id| *rule_id
                    let(rule_id)
                >
                    <RuleRow rule_id=rule_id loot_filter selected_rule />
                </For>
            </div>
        </CardInset>
    }
}

#[component]
fn RuleRow(
    rule_id: Uuid,
    loot_filter: RwSignal<LootFilter>,
    selected_rule: RwSignal<Option<Uuid>>,
) -> impl IntoView {
    let rule_name = move || {
        loot_filter
            .read()
            .rules
            .get(&rule_id)
            .map(|rule| rule.rule_name.clone())
            .unwrap_or_default()
    };

    let is_enabled = loot_filter
        .read_untracked()
        .rules
        .get(&rule_id)
        .map(|rule| rule.enabled)
        .unwrap_or_default();

    let enable_toggle = move |value| {
        if let Some(rule) = loot_filter.write().rules.get_mut(&rule_id) {
            rule.enabled = value;
        }
    };

    let move_up = move |_| {
        loot_filter.update(|filter| {
            if let Some(index) = filter.rules.get_index_of(&rule_id) {
                if index > 0 {
                    filter.rules.swap_indices(index, index - 1);
                }
            }
        });
    };

    let move_down = move |_| {
        loot_filter.update(|filter| {
            if let Some(index) = filter.rules.get_index_of(&rule_id) {
                if index + 1 < filter.rules.len() {
                    filter.rules.swap_indices(index, index + 1);
                }
            }
        });
    };
    // TODO: Move up, down, delete
    view! {
        <div
            class=move || {
                format!(
                    "relative flex w-full items-center justify-between p-3 gap-2 cursor-pointer shadow-sm transition-colors duration-150 rounded-lg
                bg-neutral-800 hover:bg-neutral-700 {}",
                    if selected_rule
                        .read()
                        .map(|selected_rule| { selected_rule == rule_id })
                        .unwrap_or_default()
                    {
                        "ring-2 ring-amber-400"
                    } else {
                        "ring-1 ring-zinc-950"
                    },
                )
            }
            on:click=move |_| { selected_rule.set(Some(rule_id)) }
        >
            <div class="flex flex-col flex-1 gap-1">

                <div class="flex items-center justify-between">
                    <div>
                        {move || {
                            match loot_filter
                                .read()
                                .rules
                                .get(&rule_id)
                                .map(|rule| rule.rule_type)
                                .unwrap_or_default()
                            {
                                FilterRuleType::Pickup => {
                                    view! { <span class="text-emerald-400">"Pickup"</span> }
                                }
                                FilterRuleType::Sell => {
                                    view! { <span class="text-orange-400">"Sell"</span> }
                                }
                            }
                        }}
                    </div>

                    <div class="text-sm xl:text-base font-semibold text-white">{rule_name}</div>

                    <div class="flex items-center gap-1">
                        <FancyButton on:click=move_up>"↑"</FancyButton>
                        <FancyButton on:click=move_down>"↓"</FancyButton>
                        <Toggle initial=is_enabled toggle_callback=enable_toggle>
                            "Enabled"
                        </Toggle>
                    </div>
                </div>

            </div>
        </div>
    }
}

fn with_selected_rule<F>(
    selected_rule: RwSignal<Option<Uuid>>,
    loot_filter: RwSignal<LootFilter>,
    f: F,
) where
    F: FnOnce(&FilterRule),
{
    if let Some(id) = selected_rule.get() {
        if let Some(rule) = loot_filter.read_untracked().rules.get(&id) {
            f(rule);
        }
    }
}

fn update_selected_rule<F>(
    selected_rule: RwSignal<Option<Uuid>>,
    loot_filter: RwSignal<LootFilter>,
    f: F,
) where
    F: FnOnce(&mut FilterRule),
{
    if let Some(id) = selected_rule.get() {
        if let Some(rule) = loot_filter.write().rules.get_mut(&id) {
            f(rule);
        }
    }
}

#[component]
pub fn EditRule(
    loot_filter: RwSignal<LootFilter>,
    selected_rule: RwSignal<Option<Uuid>>,
) -> impl IntoView {
    let do_delete = Arc::new({
        move || {
            if let Some(rule_id) = selected_rule.get_untracked() {
                loot_filter.write().rules.shift_remove(&rule_id);
                selected_rule.set(None);
            }
        }
    });

    let try_delete = {
        let confirm_context: ConfirmContext = expect_context();
        move |_| (confirm_context.confirm)("Confirm delete rule?".into(), do_delete.clone())
    };

    // Fields
    macro_rules! rule_field {
        ($name:ident) => {
            let $name = RwSignal::new(None);

            Effect::new({
                move |_| {
                    with_selected_rule(selected_rule, loot_filter, |rule| {
                        $name.set(Some(rule.$name.clone()));
                    });
                }
            });

            Effect::new({
                move |_| {
                    if let Some(value) = $name.get() {
                        update_selected_rule(selected_rule, loot_filter, |rule| {
                            rule.$name = value;
                        });
                    }
                }
            });
        };
    }

    rule_field!(rule_name);
    rule_field!(item_name);
    rule_field!(req_item_level);
    rule_field!(item_damages);
    rule_field!(item_damage_physical);
    rule_field!(item_damage_fire);
    rule_field!(item_damage_poison);
    rule_field!(item_damage_storm);
    rule_field!(item_crit_chance);
    rule_field!(item_crit_damage);
    rule_field!(item_cooldown);
    rule_field!(item_armor);
    rule_field!(item_block);

    // Stats
    let stat_filters: [(_, _); STAT_FILTERS_AMOUNT] =
        std::array::from_fn(|_| (RwSignal::new(None), RwSignal::new(None)));

    Effect::new(move || {
        with_selected_rule(selected_rule, loot_filter, move |rule| {
            for (i, (stat_type, stat_value)) in stat_filters.iter().enumerate() {
                if let Some((stat_type_2, stat_value_2)) = &rule.stat_filters[i] {
                    stat_type.set(Some(stat_type_2.clone()));
                    stat_value.set(*stat_value_2);
                } else {
                    stat_type.set(None);
                    stat_value.set(None);
                }
            }
        });
    });

    Effect::new(move || {
        for (i, (stat_type, stat_value)) in stat_filters.iter().enumerate() {
            update_selected_rule(selected_rule, loot_filter, move |rule| {
                rule.stat_filters[i] = stat_type
                    .get()
                    .map(|stat_type| (stat_type, stat_value.get()))
            });
        }
    });

    // Dropdowns
    let rule_type = RwSignal::new(FilterRuleType::Pickup);
    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, |rule| {
            rule_type.set(rule.rule_type)
        });
    });
    Effect::new(move |_| {
        update_selected_rule(selected_rule, loot_filter, |rule| {
            rule.rule_type = rule_type.get()
        });
    });
    let rule_type_options = IndexMap::from([
        (FilterRuleType::Pickup, "Pickup".to_string()),
        (FilterRuleType::Sell, "Sell".to_string()),
    ]);

    let item_rarity = RwSignal::new(None);
    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, |rule| {
            item_rarity.set(rule.item_rarity)
        });
    });
    Effect::new(move |_| {
        update_selected_rule(selected_rule, loot_filter, |rule| {
            rule.item_rarity = item_rarity.get()
        });
    });
    let item_rarity_options = std::iter::once(None)
        .chain(ItemRarity::iter().map(Some))
        .map(|rarity| (rarity, item_rarity_str(rarity).into()))
        .collect();

    let item_category = RwSignal::new(None);
    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, |rule| {
            item_category.set(rule.item_category)
        });
    });
    Effect::new(move |_| {
        update_selected_rule(selected_rule, loot_filter, |rule| {
            rule.item_category = item_category.get()
        });
    });
    let item_category_options = std::iter::once(None)
        .chain(ItemCategory::iter().map(Some))
        .map(|category| (category, loot_filter_category_to_str(category).into()))
        .collect();

    view! {
        <CardInset>
            <div class="flex justify-between items-center p-4 border-b border-zinc-700">
                <ValidatedInput
                    id="rule_name"
                    label="Rule Name:"
                    input_type="text"
                    placeholder="Enter custom rule name"
                    bind=rule_name
                />

                <DropdownMenu options=rule_type_options chosen_option=rule_type />

                <MenuButton on:click=try_delete>"❌"</MenuButton>
            </div>
            <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 p-4 border-b border-zinc-700">
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_name"
                        label="Item Name:"
                        input_type="text"
                        placeholder="Enter item name"
                        bind=item_name
                    />

                    <ValidatedInput
                        id="item_level"
                        label="Required Level:"
                        input_type="number"
                        placeholder="Enter required level"
                        bind=req_item_level
                    />
                </div>

                <div class="flex flex-col gap-4">
                    <div class="flex items-center justify-between text-gray-300 text-sm">
                        <span>"Item Category:"</span>
                        <SearchableDropdownMenu
                            options=item_category_options
                            chosen_option=item_category
                        />
                    </div>

                    <div class="flex items-center justify-between text-gray-300 text-sm">
                        <span>"Item Rarity:"</span>
                        <DropdownMenu options=item_rarity_options chosen_option=item_rarity />
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 p-4 border-b border-zinc-700">
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_damages"
                        label="Damage:"
                        input_type="number"
                        placeholder="Damage per second"
                        bind=item_damages
                    />
                    <ValidatedInput
                        id="item_damage_physical"
                        label="Physical Damage:"
                        input_type="number"
                        placeholder="Physical Damage"
                        bind=item_damage_physical
                    />
                    <ValidatedInput
                        id="item_damage_fire"
                        label="Fire Damage:"
                        input_type="number"
                        placeholder="Fire Damage"
                        bind=item_damage_fire
                    />
                    <ValidatedInput
                        id="item_damage_poison"
                        label="Poison Damage:"
                        input_type="number"
                        placeholder="Poison Damage"
                        bind=item_damage_poison
                    />
                    <ValidatedInput
                        id="item_damage_storm"
                        label="Storm Damage:"
                        input_type="number"
                        placeholder="Storm Damage"
                        bind=item_damage_storm
                    />
                </div>
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_cooldown"
                        label="Cooldown:"
                        input_type="number"
                        placeholder="Cooldown"
                        bind=item_cooldown
                    />
                    <ValidatedInput
                        id="item_damages"
                        label="Critical Hit Chance:"
                        input_type="number"
                        placeholder="Critical Percent Chance"
                        bind=item_crit_chance
                    />
                    <ValidatedInput
                        id="item_damages"
                        label="Critical Hit Damage:"
                        input_type="number"
                        placeholder="Critical Percent Damage"
                        bind=item_crit_damage
                    />
                    <ValidatedInput
                        id="item_armor"
                        label="Armor:"
                        input_type="number"
                        placeholder="Armor"
                        bind=item_armor
                    />
                    <ValidatedInput
                        id="item_block"
                        label="Block %:"
                        input_type="number"
                        placeholder="Block Percent Chance"
                        bind=item_block
                    />
                </div>
            </div>

            <div class="flex flex-col gap-2 xl:gap-4 p-2 xl:p-4">
                {stat_filters
                    .map(|(stat_type, stat_value)| {
                        view! {
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
                                                        placeholder="Min"
                                                        bind=stat_value
                                                    />
                                                </div>
                                            }
                                        })
                                }}
                            </div>
                        }
                    })}
            </div>
        </CardInset>
    }
}
