use codee::string::JsonSerdeCodec;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

use leptos::{html::*, prelude::*};
use leptos_use_storage::use_local_storage;

use shared::{
    data::{
        area::AreaLevel,
        item::{ItemCategory, ItemRarity},
        item_affix::{AffixType, ItemAffixBlueprint},
        market::STAT_FILTERS_AMOUNT,
        modifier::Modifier,
        stat_effect::{StatEffect, StatType},
        user::UserCharacterId,
    },
    types::ItemName,
};

use crate::components::{
    data_context::DataContext,
    shared::{
        inventory::loot_filter_category_to_str,
        tooltips::{effects_tooltip::format_stat, item_tooltip::affix_type_str},
    },
    town::panels::market::{StatDropdown, format_stat_filter, item_rarity_str},
    ui::{
        buttons::{FancyButton, MenuButton, Toggle},
        card::{CardHeader, CardInset, MenuCard},
        checkbox::Checkbox,
        confirm::ConfirmContext,
        dropdown::{DropdownMenu, SearchableDropdownMenu},
        input::{Input, ValidatedInput},
        list_row::MenuListRow,
        menu_panel::MenuPanel,
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
    utils::file_loader::{save_json, use_json_loader},
};

pub const AFFIX_FILTERS_AMOUNT: usize = 5;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LootFilter {
    pub rules: IndexMap<Uuid, FilterRule>,

    #[serde(default)]
    pub immediate_mode: bool,
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
    pub item_level: Option<AreaLevel>,
    pub req_item_level: Option<AreaLevel>,
    #[serde(default)]
    pub req_affix_level: Option<AreaLevel>,
    #[serde(default)]
    pub max_power_shard_level: Option<AreaLevel>,

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

    #[allow(clippy::type_complexity)]
    pub stat_filters: [Option<((StatType, Modifier), Option<f64>, bool)>; STAT_FILTERS_AMOUNT],

    #[serde(default)]
    pub affix_filters: [Option<LootAffixFilter>; AFFIX_FILTERS_AMOUNT],
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LootAffixFilter {
    pub affix_id: String,
    #[serde(default)]
    pub min_level: Option<AreaLevel>,
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
pub fn LootFilterPanel(
    open: RwSignal<bool>,
    loot_filter: RwSignal<LootFilter>,
    character_id: UserCharacterId,
    character_name: String,
) -> impl IntoView {
    let selected_rule = RwSignal::new(None);

    let (get_loot_filter, set_loot_filter, _) =
        use_local_storage::<LootFilter, JsonSerdeCodec>(format!("loot_filter_{}", character_id));

    loot_filter.set(get_loot_filter.get_untracked());
    let local_loot_filter = RwSignal::new(loot_filter.get_untracked());

    Effect::new(move || {
        if open.get() {
            local_loot_filter.set(loot_filter.get());
        } else {
            loot_filter.set(local_loot_filter.get());
            set_loot_filter.set(local_loot_filter.get());
        }
    });

    let (loaded_file, filename, on_change) = use_json_loader::<LootFilter>();
    let file_input: NodeRef<Input> = NodeRef::new();

    Effect::new(move || {
        loaded_file.with(|loaded_file| {
            if let Some(loaded_filter) = loaded_file {
                local_loot_filter.set(loaded_filter.clone());
            }
        });
    });

    let on_export = {
        let character_name = character_name.clone();
        move |_| {
            save_json(
                &local_loot_filter.get_untracked(),
                &filename
                    .get()
                    .unwrap_or(format!("loot_filter_{}.json", character_name)),
            );
        }
    };

    let on_import = move |_| {
        if let Some(input) = file_input.get() {
            input.click();
        }
    };

    let new_rule = move || {
        let rule_id = Uuid::new_v4();
        local_loot_filter
            .write()
            .rules
            .insert(rule_id, FilterRule::new());
        selected_rule.set(Some(rule_id));
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
            {
                let on_export = on_export.clone();
                view! {
                    <MenuCard class="w-full h-full">
                        <CardHeader title="Loot Filter" on_close=move || open.set(false)>
                            <div class="flex gap-4 mx-4">
                                <MenuButton on:click=move |_| new_rule()>"New Rule"</MenuButton>
                                <div class="flex items-center">
                                    <Checkbox
                                        label="Immediate Mode".into()
                                        on_change=move |value| {
                                            local_loot_filter.write().immediate_mode = value
                                        }
                                        checked=Signal::derive(move || {
                                            local_loot_filter.read().immediate_mode
                                        })
                                    />

                                    <StaticTooltip
                                        position=StaticTooltipPosition::Bottom
                                        tooltip=|| {
                                            view! {
                                                <div class="flex flex-col xl:space-y-1 w-[20vw] whitespace-normal">
                                                    "By default, loot filter automatically picks up and/or sells items when they reach the end of the loot queue at the bottom right of the screen. In Immediate Mode, it instantly triggers instead."
                                                </div>
                                            }
                                        }
                                    >
                                        <span class="ml-1 inline-flex h-5 w-5 items-center justify-center rounded-full border border-zinc-500 text-xs text-zinc-300 cursor-help">
                                            "?"
                                        </span>
                                    </StaticTooltip>
                                </div>
                            </div>

                            // <div class="flex-1" />
                            // <p class="text-xs xl:text-sm font-medium text-zinc-400">
                            // "Loot filter automatically picks up and/or sells items when they reach the end of the loot queue at the bottom right of the screen."
                            // </p>
                            <div class="flex-1" />

                            <div class="flex gap-2 mx-4">
                                <MenuButton on:click=on_import>"Import"</MenuButton>
                                <MenuButton on:click=on_export>"Export"</MenuButton>
                            </div>
                        </CardHeader>
                        <div class="grid grid-cols-2 gap-2 min-h-0 flex-1">
                            <RulesList loot_filter=local_loot_filter selected_rule />
                            <EditRule loot_filter=local_loot_filter selected_rule />
                        </div>
                    </MenuCard>
                }
            }
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
                <p class="text-xs xl:text-sm font-medium text-zinc-400">
                    "Rules are evaluated in order, from top to bottom."
                </p>
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
    let is_selected = Signal::derive(move || {
        selected_rule
            .read()
            .map(|selected_rule| selected_rule == rule_id)
            .unwrap_or_default()
    });

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

    let move_up = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        loot_filter.update(|filter| {
            if let Some(index) = filter.rules.get_index_of(&rule_id)
                && index > 0
            {
                filter.rules.swap_indices(index, index - 1);
            }
        });
    };

    let move_down = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        loot_filter.update(|filter| {
            if let Some(index) = filter.rules.get_index_of(&rule_id)
                && index + 1 < filter.rules.len()
            {
                filter.rules.swap_indices(index, index + 1);
            }
        });
    };

    let is_first = Signal::derive(move || {
        loot_filter
            .read()
            .rules
            .first()
            .map(|(k, _)| *k == rule_id)
            .unwrap_or(false)
    });

    let is_last = Signal::derive(move || {
        loot_filter
            .read()
            .rules
            .last()
            .map(|(k, _)| *k == rule_id)
            .unwrap_or(false)
    });

    view! {
        <MenuListRow selected=is_selected on_click=move || { selected_rule.set(Some(rule_id)) }>
            <div class="flex flex-col flex-1 gap-1 p-2">

                <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-1 shrink-0">
                        <FancyButton on:click=move_up disabled=is_first>
                            "↑"
                        </FancyButton>
                        <FancyButton on:click=move_down disabled=is_last>
                            "↓"
                        </FancyButton>
                        {move || {
                            match loot_filter
                                .read()
                                .rules
                                .get(&rule_id)
                                .map(|rule| rule.rule_type)
                                .unwrap_or_default()
                            {
                                FilterRuleType::Pickup => {
                                    view! {
                                        <span class="ml-2 w-12 text-emerald-400">"Pickup"</span>
                                    }
                                }
                                FilterRuleType::Sell => {
                                    view! { <span class="ml-2 w-12 text-orange-400">"Sell"</span> }
                                }
                            }
                        }}
                    </div>

                    <div class="min-w-0 flex-1 px-2 text-center text-sm xl:text-base font-semibold text-white truncate">
                        {rule_name}
                    </div>

                    <div class="flex items-center gap-1 shrink-0">
                        <Toggle initial=is_enabled toggle_callback=enable_toggle>
                            {move || {
                                if loot_filter
                                    .read()
                                    .rules
                                    .get(&rule_id)
                                    .map(|rule| rule.enabled)
                                    .unwrap_or_default()
                                {
                                    "On"
                                } else {
                                    "Off"
                                }
                            }}
                        </Toggle>
                    </div>
                </div>

            </div>
        </MenuListRow>
    }
}

fn with_selected_rule<F>(
    selected_rule: RwSignal<Option<Uuid>>,
    loot_filter: RwSignal<LootFilter>,
    f: F,
) where
    F: FnOnce(&FilterRule),
{
    if let Some(id) = selected_rule.get()
        && let Some(rule) = loot_filter.read_untracked().rules.get(&id)
    {
        f(rule);
    }
}

fn update_selected_rule<F>(
    selected_rule: RwSignal<Option<Uuid>>,
    loot_filter: RwSignal<LootFilter>,
    f: F,
) where
    F: FnOnce(&mut FilterRule),
{
    if let Some(id) = selected_rule.get()
        && let Some(rule) = loot_filter.write().rules.get_mut(&id)
    {
        f(rule);
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
    rule_field!(req_affix_level);
    rule_field!(item_level);
    rule_field!(max_power_shard_level);
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
    let stat_filters: [(_, _, _); STAT_FILTERS_AMOUNT] = std::array::from_fn(|_| {
        (
            RwSignal::new(None),
            RwSignal::new(None),
            RwSignal::new(false),
        )
    });
    let visible_stat_filters = Memo::new(move |_| {
        visible_filter_slots(&stat_filters.map(|(stat_type, _, _)| stat_type.get().is_some()))
    });

    Effect::new(move || {
        with_selected_rule(selected_rule, loot_filter, move |rule| {
            for (i, (stat_type, stat_value, stat_excluded)) in stat_filters.iter().enumerate() {
                if let Some((stat_type_2, stat_value_2, stat_excluded_2)) = &rule.stat_filters[i] {
                    stat_type.set(Some(stat_type_2.clone()));
                    stat_value.set(*stat_value_2);
                    stat_excluded.set(*stat_excluded_2);
                } else {
                    stat_type.set(None);
                    stat_value.set(None);
                    stat_excluded.set(false);
                }
            }
        });
    });

    Effect::new(move || {
        for (i, (stat_type, stat_value, stat_excluded)) in stat_filters.iter().enumerate() {
            update_selected_rule(selected_rule, loot_filter, move |rule| {
                rule.stat_filters[i] = stat_type
                    .get()
                    .map(|stat_type| (stat_type, stat_value.get(), stat_excluded.get()));
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
        .chain(
            ItemCategory::iter()
                .filter(|item_category| *item_category != ItemCategory::Cloak)
                .map(Some),
        )
        .map(|category| (category, loot_filter_category_to_str(category).into()))
        .collect();

    let affix_filters: [(_, _); AFFIX_FILTERS_AMOUNT] = std::array::from_fn(|_| {
        (
            RwSignal::new(None::<String>),
            RwSignal::new(None::<AreaLevel>),
        )
    });
    let visible_affix_filters = Memo::new(move |_| {
        visible_filter_slots(&affix_filters.map(|(affix_id, _)| affix_id.get().is_some()))
    });

    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, move |rule| {
            for (index, (affix_id, min_level)) in affix_filters.iter().enumerate() {
                if let Some(filter) = &rule.affix_filters[index] {
                    affix_id.set(Some(filter.affix_id.clone()));
                    min_level.set(filter.min_level);
                } else {
                    affix_id.set(None);
                    min_level.set(None);
                }
            }
        });
    });

    Effect::new(move |_| {
        for (index, (affix_id, min_level)) in affix_filters.iter().enumerate() {
            update_selected_rule(selected_rule, loot_filter, move |rule| {
                rule.affix_filters[index] = affix_id.get().map(|affix_id| LootAffixFilter {
                    affix_id,
                    min_level: min_level.get(),
                });
            });
        }
    });

    let data_context: DataContext = expect_context();
    let available_affix_groups =
        Memo::new(move |_| affix_groups(data_context, item_category.get()));

    Effect::new(move |_| {
        let groups = available_affix_groups.read();
        for (affix_id, min_level) in affix_filters {
            let Some(selected_affix) = affix_id.get() else {
                min_level.set(None);
                continue;
            };
            let Some(group) = groups.get(&selected_affix) else {
                affix_id.set(None);
                min_level.set(None);
                continue;
            };
            if min_level
                .get()
                .is_some_and(|level| !group.levels.contains(&level))
            {
                min_level.set(None);
            }
        }
    });

    view! {
        <CardInset>
            <div class="flex justify-between items-center px-4 pb-4 border-b border-zinc-700">
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
                        label="Item Level:"
                        input_type="number"
                        placeholder="Enter item level"
                        bind=item_level
                    />

                    <div class="flex items-center justify-between text-zinc-400 text-sm">
                        <span>"Item Category:"</span>
                        <SearchableDropdownMenu
                            options=item_category_options
                            chosen_option=item_category
                        />
                    </div>
                </div>

                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="req_item_level"
                        label="Required Base Level:"
                        input_type="number"
                        placeholder="Enter required base level"
                        bind=req_item_level
                    />

                    <ValidatedInput
                        id="req_affix_level"
                        label="Required Affix Level:"
                        input_type="number"
                        placeholder="Enter required affix level"
                        bind=req_affix_level
                    />

                    <div class="flex items-center justify-between text-zinc-400 text-sm">
                        <span>"Item Rarity:"</span>
                        <DropdownMenu options=item_rarity_options chosen_option=item_rarity />
                    </div>
                </div>
            </div>

            <div class="flex flex-col gap-2 xl:gap-4 p-2 xl:p-4 border-b border-zinc-700">
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

            <div class="flex flex-col gap-2 xl:gap-4 p-2 xl:p-4 border-b border-zinc-700">
                {affix_filters
                    .into_iter()
                    .enumerate()
                    .map(|(index, (affix_id, min_level))| {
                        view! {
                            <Show when=move || index < visible_affix_filters.get()>
                                <div class="flex w-full min-w-0 gap-2 xl:gap-4 items-center">
                                    {move || {
                                        affix_id
                                            .read()
                                            .is_some()
                                            .then(|| {
                                                view! {
                                                    <MenuButton
                                                        class:flex-none
                                                        on:click=move |_| {
                                                            affix_id.set(None);
                                                            min_level.set(None);
                                                        }
                                                    >
                                                        "❌"
                                                    </MenuButton>
                                                }
                                            })
                                    }}
                                    <span class=move || {
                                        if affix_id.read().is_none() {
                                            "flex-1 min-w-0 text-center"
                                        } else {
                                            "flex-1 min-w-0"
                                        }
                                    }>
                                        {move || {
                                            let options = available_affix_groups
                                                .read()
                                                .iter()
                                                .map(|(group_id, group)| {
                                                    (Some(group_id.clone()), group.label.clone())
                                                })
                                                .collect();
                                            view! {
                                                <SearchableDropdownMenu
                                                    options
                                                    chosen_option=affix_id
                                                    missing_text="+ Add Affix Filter"
                                                    width_class="w-full min-w-0"
                                                />
                                            }
                                        }}
                                    </span>
                                    {move || {
                                        affix_id
                                            .get()
                                            .and_then(|selected_affix| {
                                                available_affix_groups.read().get(&selected_affix).cloned()
                                            })
                                            .map(|group| {
                                                let options = std::iter::once((None, "Level".to_string()))
                                                    .chain(
                                                        group
                                                            .levels
                                                            .into_iter()
                                                            .map(|level| { (Some(level), level.to_string()) }),
                                                    )
                                                    .collect();
                                                view! {
                                                    <DropdownMenu
                                                        options
                                                        chosen_option=min_level
                                                        missing_text="Level"
                                                        width_class="w-24"
                                                    />
                                                }
                                            })
                                    }}
                                </div>
                            </Show>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 p-4">
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
                    <ValidatedInput
                        id="max_power_shard_level"
                        label="Power Shards Unlock Level:"
                        input_type="number"
                        placeholder="Power Shards Unlock Level"
                        bind=max_power_shard_level
                    />
                </div>
            </div>

        </CardInset>
    }
}

pub fn affix_group_id(affix_id: &str) -> &str {
    affix_id
        .rsplit_once("_t")
        .filter(|(_, tier)| !tier.is_empty() && tier.chars().all(|c| c.is_ascii_digit()))
        .map(|(group_id, _)| group_id)
        .unwrap_or(affix_id)
}

#[derive(Clone, PartialEq)]
struct AffixGroupOption {
    label: String,
    levels: Vec<AreaLevel>,
}

fn affix_effects_label(
    affix: &ItemAffixBlueprint,
    affix_base_category: AffixBaseCategory,
    show_fixed_values: bool,
) -> String {
    let effects = affix
        .effects
        .iter()
        .map(|effect| {
            if show_fixed_values && effect.value.min == effect.value.max {
                format_stat(&StatEffect {
                    stat: effect.stat.clone(),
                    modifier: effect.modifier,
                    value: effect.value.min,
                    bypass_ignore: effect.bypass_ignore,
                })
            } else {
                format_stat_filter(&effect.stat, effect.modifier)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let meta = if let Some(category) = affix_base_category_str(affix_base_category) {
        format!("({category} · {})", affix_type_str(affix.affix_type))
    } else {
        format!("({})", affix_type_str(affix.affix_type))
    };

    format!(" {meta} {effects}",)
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AffixBaseCategory {
    Item,
    Rune,
    Map,
    Other,
}

fn affix_base_category(affix_id: &str) -> AffixBaseCategory {
    let prefix = affix_id
        .split_once('_')
        .map(|(prefix, _)| prefix)
        .unwrap_or(affix_id);
    match prefix {
        "item" => AffixBaseCategory::Item,
        "rune" => AffixBaseCategory::Rune,
        "map" => AffixBaseCategory::Map,
        _ => AffixBaseCategory::Other,
    }
}

fn affix_base_category_str(affix_base_category: AffixBaseCategory) -> Option<&'static str> {
    match affix_base_category {
        AffixBaseCategory::Item => Some("Item"),
        AffixBaseCategory::Rune => Some("Rune"),
        AffixBaseCategory::Map => Some("Edict"),
        AffixBaseCategory::Other => None,
    }
}

fn visible_filter_slots(occupied: &[bool]) -> usize {
    let Some(last_occupied) = occupied.iter().rposition(|occupied| *occupied) else {
        return usize::from(!occupied.is_empty());
    };

    let occupied_slots = last_occupied + 1;
    if occupied_slots < occupied.len()
        && occupied[..occupied_slots].iter().all(|occupied| *occupied)
    {
        occupied_slots + 1
    } else {
        occupied_slots
    }
}

fn affix_groups(
    data_context: DataContext,
    category: Option<ItemCategory>,
) -> IndexMap<String, AffixGroupOption> {
    let item_affixes = data_context.item_affixes.read_untracked();
    let mut groups =
        BTreeMap::<(AffixBaseCategory, AffixType, String), Vec<&ItemAffixBlueprint>>::new();

    let intersected_categories = category
        .map(|category| {
            data_context
                .item_bases
                .read_untracked()
                .values()
                .filter(|item_base| item_base.categories.contains(&category))
                .fold(BTreeSet::new(), |mut acc, item_base| {
                    if acc.is_empty() {
                        acc.extend(item_base.categories.iter());
                    } else {
                        acc.retain(|c| item_base.categories.contains(c));
                    }

                    acc
                })
        })
        .unwrap_or_default();

    for (affix_id, affix) in item_affixes.iter().filter(|(_, affix)| {
        matches!(
            affix.affix_type,
            AffixType::Prefix | AffixType::Suffix | AffixType::Soul
        ) && (category.is_none()
            || (category == Some(ItemCategory::Remnant) && affix.affix_type == AffixType::Soul)
            || !intersected_categories.is_disjoint(&affix.restrictions))
    }) {
        groups
            .entry((
                affix_base_category(affix_id),
                affix.affix_type,
                affix_group_id(affix_id).to_string(),
            ))
            .or_default()
            .push(affix);
    }

    groups
        .into_iter()
        .map(|((affix_base_category, _, group_id), group)| {
            let label = affix_effects_label(
                group
                    .first()
                    .copied()
                    .expect("aggregated affix groups cannot be empty"),
                affix_base_category,
                group.len() == 1,
            );
            let mut levels = group
                .iter()
                .map(|affix| affix.item_level)
                .collect::<Vec<_>>();
            levels.sort_unstable();

            (group_id, AffixGroupOption { label, levels })
        })
        .collect()
}
