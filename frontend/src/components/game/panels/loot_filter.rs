use indexmap::IndexMap;
use leptos::{html::*, prelude::*};

use shared::{
    data::{
        area::AreaLevel,
        item::{ItemCategory, ItemRarity},
    },
    types::ItemName,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::components::{
    game::GameContext,
    shared::inventory::loot_filter_category_to_str,
    town::panels::market::item_rarity_str,
    ui::{
        buttons::{MenuButton, Toggle},
        card::{Card, CardHeader, CardInset},
        dropdown::{DropdownMenu, SearchableDropdownMenu},
        input::ValidatedInput,
        menu_panel::MenuPanel,
    },
};

#[derive(Debug, Clone, Copy, Default)]
pub struct LootFilter {
    pub rules: RwSignal<IndexMap<Uuid, FilterRule>>,
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub enum FilterRuleType {
    #[default]
    Pickup,
    Sell,
}

#[derive(Debug, Clone, Default)]
pub struct FilterRule {
    pub rule_id: Uuid,
    pub rule_type: FilterRuleType,
    pub rule_name: String,

    pub enabled: bool,

    pub item_name: Option<ItemName>,
    pub req_item_level: Option<AreaLevel>,

    pub item_rarity: Option<ItemRarity>,
    pub item_category: Option<ItemCategory>,
}

impl FilterRule {
    fn new() -> Self {
        Self {
            rule_id: Uuid::new_v4(),
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
        let new_rule = FilterRule::new();
        game_context
            .loot_filter
            .rules
            .write()
            .insert(new_rule.rule_id, new_rule);
    };

    view! {
        <MenuPanel open=open class:items-center>
            <Card class="w-full h-full">
                <CardHeader title="Loot Filter" on_close=move || open.set(false)>
                    <div class="flex gap-2 mx-4">
                        <MenuButton on:click=move |_| new_rule()>"New Rule"</MenuButton>
                    </div>

                    <div class="flex-1" />

                    <div class="flex gap-2 mx-4">
                        <MenuButton>"Import"</MenuButton>
                        <MenuButton>"Export"</MenuButton>
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
pub fn RulesList(loot_filter: LootFilter, selected_rule: RwSignal<Option<Uuid>>) -> impl IntoView {
    view! {
        <CardInset>
            <div class="gap-2 flex flex-col">
                <For
                    each=move || {
                        let keys = loot_filter.rules.read().keys().copied().collect::<Vec<_>>();
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
    loot_filter: LootFilter,
    selected_rule: RwSignal<Option<Uuid>>,
) -> impl IntoView {
    let rule_name = move || {
        loot_filter
            .rules
            .read()
            .get(&rule_id)
            .map(|rule| rule.rule_name.clone())
            .unwrap_or_default()
    };

    let is_enabled = loot_filter
        .rules
        .read_untracked()
        .get(&rule_id)
        .map(|rule| rule.enabled)
        .unwrap_or_default();

    let enable_toggle = move |value| {
        if let Some(rule) = loot_filter.rules.write().get_mut(&rule_id) {
            rule.enabled = value;
        }
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
                                .rules
                                .read()
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

                    // <div class="text-sm text-gray-400">
                    // {move || {
                    // if stash.read().max_items > 0 {
                    // format!("{}/{}", stash.read().items_amount, stash.read().max_items)
                    // } else {
                    // "Click to buy!".into()
                    // }
                    // }}
                    // </div>
                    <Toggle initial=is_enabled toggle_callback=enable_toggle>
                        "Enabled"
                    </Toggle>
                // <MenuButton on:click=move |_| {}>"❌"</MenuButton>
                </div>

            </div>
        </div>
    }
}

fn with_selected_rule<F>(selected_rule: RwSignal<Option<Uuid>>, loot_filter: LootFilter, f: F)
where
    F: FnOnce(&FilterRule),
{
    if let Some(id) = selected_rule.get() {
        if let Some(rule) = loot_filter.rules.read_untracked().get(&id) {
            f(rule);
        }
    }
}

fn update_selected_rule<F>(selected_rule: RwSignal<Option<Uuid>>, loot_filter: LootFilter, f: F)
where
    F: FnOnce(&mut FilterRule),
{
    if let Some(id) = selected_rule.get() {
        if let Some(rule) = loot_filter.rules.write().get_mut(&id) {
            f(rule);
        }
    }
}

#[component]
pub fn EditRule(loot_filter: LootFilter, selected_rule: RwSignal<Option<Uuid>>) -> impl IntoView {
    let rule_name = RwSignal::new(None);
    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, |rule| {
            rule_name.set(Some(rule.rule_name.clone()))
        });
    });
    Effect::new(move |_| {
        if let Some(rule_name) = rule_name.get() {
            update_selected_rule(selected_rule, loot_filter, |rule| {
                rule.rule_name = rule_name
            });
        }
    });

    let item_name = RwSignal::new(None);
    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, |rule| {
            item_name.set(Some(rule.item_name.clone()))
        });
    });
    Effect::new(move |_| {
        if let Some(item_name) = item_name.get() {
            update_selected_rule(selected_rule, loot_filter, |rule| {
                rule.item_name = item_name
            });
        }
    });

    let item_level = RwSignal::new(None);
    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, |rule| {
            item_level.set(Some(rule.req_item_level.clone()))
        });
    });
    Effect::new(move |_| {
        if let Some(item_level) = item_level.get() {
            update_selected_rule(selected_rule, loot_filter, |rule| {
                rule.req_item_level = item_level
            });
        }
    });

    //Dropdowns
    let rule_type = RwSignal::new(FilterRuleType::Pickup);
    Effect::new(move |_| {
        with_selected_rule(selected_rule, loot_filter, |rule| {
            rule_type.set(rule.rule_type.clone())
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
            item_rarity.set(rule.item_rarity.clone())
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
            item_category.set(rule.item_category.clone())
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

                <MenuButton on:click=move |_| {}>"❌"</MenuButton>
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
                        label="Min Required Level:"
                        input_type="number"
                        placeholder="Enter minimum required level"
                        bind=item_level
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
        </CardInset>
    }
}
