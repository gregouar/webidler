use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{
    data::{
        area::AreaLevel,
        item::{ItemRarity, ItemSpecs},
        item_affix::AffixEffectScope,
        loot::LootState,
        player::EquippedSlot,
    },
    messages::client::PickUpLootMessage,
};

use crate::components::{
    accessibility::AccessibilityContext,
    game::{
        GameContext,
        panels::loot_filter::{FilterRule, FilterRuleType, LootFilter},
        websocket::WebsocketContext,
    },
    shared::item_card::ItemCard,
    ui::tooltip::DynamicTooltipPosition,
};

#[component]
pub fn LootQueue() -> impl IntoView {
    let conn: WebsocketContext = expect_context();
    let accessibility: AccessibilityContext = expect_context();
    let game_context = expect_context::<GameContext>();

    let pickup_loot = {
        let conn = conn.clone();
        move |loot_identifier| {
            game_context.queued_loot.update(|queued_loot| {
                if let Some(loot) = queued_loot
                    .iter_mut()
                    .find(|loot| loot.identifier == loot_identifier)
                {
                    loot.state = LootState::HasDisappeared
                }
            });

            conn.send(
                &PickUpLootMessage {
                    loot_identifier,
                    sell: false,
                }
                .into(),
            );
        }
    };

    let sell_loot = move |loot_identifier| {
        game_context.queued_loot.update(|queued_loot| {
            if let Some(loot) = queued_loot
                .iter_mut()
                .find(|loot| loot.identifier == loot_identifier)
            {
                loot.state = LootState::HasDisappeared
            }
        });

        conn.send(
            &PickUpLootMessage {
                loot_identifier,
                sell: true,
            }
            .into(),
        );
    };

    let position_style = move |loot_identifier| {
        let index = game_context
            .queued_loot
            .read()
            .iter()
            .filter(|l| l.state != LootState::HasDisappeared || l.identifier == loot_identifier)
            .rev()
            .position(|l| l.identifier == loot_identifier)
            .unwrap_or_default();
        format!("left: {}%;", 4 + index * 20)
    };

    let animation_style = move |loot_identifier| {
        let state = game_context
            .queued_loot
            .read()
            .iter()
            .find(|l| l.identifier == loot_identifier)
            .map(|l| l.state)
            .unwrap_or_default();
        match state {
            LootState::Normal => "animation: loot-float 2.5s ease-in-out infinite;",
            LootState::WillDisappear => "animation: loot-vibrate 0.3s linear infinite;",
            LootState::HasDisappeared => {
                "animation: loot-pickup 0.3s ease forwards; pointer-events: none;"
            }
        }
    };

    Effect::new({
        let loot_filter = game_context.loot_filter;
        let pickup_loot = pickup_loot.clone();
        let sell_loot = sell_loot.clone();
        let last_try = RwSignal::new(Default::default());
        move || {
            let queued_loot = game_context
                .queued_loot
                .read()
                .iter()
                .find(|l| l.state == LootState::WillDisappear)
                .cloned();

            if let Some(queued_loot) = queued_loot
                && last_try.try_get_untracked().unwrap_or_default() != queued_loot.identifier
            {
                match filter_loot(loot_filter, &queued_loot.item_specs) {
                    Some(FilterRuleType::Pickup) => pickup_loot(queued_loot.identifier),
                    Some(FilterRuleType::Sell) => sell_loot(queued_loot.identifier),
                    None => {}
                }
                last_try.set(queued_loot.identifier);
            }
        }
    });

    view! {
        <div class="relative w-full z-0 pr-4">
            <For
                each=move || game_context.queued_loot.get().into_iter()
                key=|loot| loot.identifier
                let(loot)
            >
                {
                    let item_rarity = loot.item_specs.modifiers.rarity;
                    view! {
                        <div style="animation: loot-drop 1.3s ease forwards;">
                            <div
                                class="
                                absolute bottom-0 w-[12%] aspect-[2/3]
                                transition-all duration-500 ease
                                will-change-opacity
                                will-change-transform
                                pointer-events-none
                                "
                                style=move || {
                                    format!(
                                        "{} {}",
                                        animation_style(loot.identifier),
                                        position_style(loot.identifier),
                                    )
                                }
                            >
                                <div
                                    class="
                                    relative
                                    transition-all duration-200 ease-in-out 
                                    translate-y-1/2 hover:translate-y-1/4
                                    pointer-events-auto
                                    "
                                    on:click={
                                        let pickup_loot = pickup_loot.clone();
                                        move |_| pickup_loot(loot.identifier)
                                    }

                                    on:contextmenu={
                                        let sell_loot = sell_loot.clone();
                                        move |_| {
                                            if !accessibility.is_on_mobile()
                                                && item_rarity != ItemRarity::Unique
                                            {
                                                sell_loot(loot.identifier);
                                            }
                                        }
                                    }
                                >
                                    <ItemCard
                                        comparable_item_specs=loot
                                            .item_specs
                                            .base
                                            .slot
                                            .and_then(|slot| {
                                                game_context
                                                    .player_inventory
                                                    .read()
                                                    .equipped
                                                    .get(&slot)
                                                    .and_then(|equipped_slot| match equipped_slot {
                                                        EquippedSlot::MainSlot(item_specs) => {
                                                            Some(Arc::from(item_specs.clone()))
                                                        }
                                                        EquippedSlot::ExtraSlot(_) => None,
                                                    })
                                            })
                                        item_specs=Arc::new(loot.item_specs)
                                        tooltip_position=DynamicTooltipPosition::TopLeft
                                        class:shadow-lg
                                        max_item_level=Signal::derive(|| AreaLevel::MAX)
                                    />
                                </div>
                            </div>
                        </div>
                    }
                }
            </For>
        </div>
    }
}

fn filter_loot(
    loot_filter: RwSignal<LootFilter>,
    item_specs: &ItemSpecs,
) -> Option<FilterRuleType> {
    for rule in loot_filter
        .read()
        .rules
        .values()
        .filter(|rule| rule.enabled)
    {
        if verify_filter_rule(rule, item_specs) {
            return Some(rule.rule_type);
        }
    }
    None
}

fn verify_filter_rule(filter_rule: &FilterRule, item_specs: &ItemSpecs) -> bool {
    let FilterRule {
        rule_type,
        rule_name: _,
        enabled,
        item_name,
        req_item_level,
        item_rarity,
        item_category,
        item_damages,
        item_damage_physical,
        item_damage_fire,
        item_damage_poison,
        item_damage_storm,
        item_crit_chance,
        item_crit_damage,
        item_armor,
        item_block,
        stat_filters,
    } = filter_rule;

    if !enabled {
        return true;
    }

    if item_name
        .as_ref()
        .map(|item_name| {
            !item_specs
                .base
                .name
                .to_lowercase()
                .contains(&item_name.to_lowercase())
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_category
        .map(|item_category| !item_specs.base.categories.contains(&item_category))
        .unwrap_or_default()
    {
        return false;
    }

    if item_rarity
        .map(|item_rarity| !match rule_type {
            FilterRuleType::Pickup => item_specs.modifiers.rarity >= item_rarity,
            FilterRuleType::Sell => item_specs.modifiers.rarity <= item_rarity,
        })
        .unwrap_or_default()
    {
        return false;
    }

    if req_item_level
        .map(|req_item_level| !match rule_type {
            FilterRuleType::Pickup => item_specs.required_level >= req_item_level,
            FilterRuleType::Sell => item_specs.required_level <= req_item_level,
        })
        .unwrap_or_default()
    {
        return false;
    }

    let effects = item_specs
        .modifiers
        .aggregate_effects(AffixEffectScope::Global)
        .0;
    for stat_filter in stat_filters {
        if let Some(((stat_type, stat_modifier), stat_value)) = stat_filter.as_ref() {
            if !effects
                .get(&(stat_type.clone(), *stat_modifier, false))
                .map(|value| match rule_type {
                    FilterRuleType::Pickup => stat_value
                        .map(|stat_value| *value >= stat_value)
                        .unwrap_or(true),
                    FilterRuleType::Sell => stat_value
                        .map(|stat_value| *value <= stat_value)
                        .unwrap_or(true),
                })
                .unwrap_or_default()
            {
                return false;
            }
        }
    }

    true
}
