use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{
    data::{
        area::AreaLevel,
        item::{ItemRarity, ItemSpecs},
        item_affix::AffixEffectScope,
        loot::LootState,
        player::EquippedSlot,
        skill::DamageType,
    },
    messages::client::PickUpLootMessage,
};

use crate::{
    assets::img_asset,
    components::{
        accessibility::AccessibilityContext,
        game::{GameContext, websocket::WebsocketContext},
        settings::SettingsContext,
        shared::{
            item_card::ItemCard,
            loot_filter::{FilterRule, FilterRuleType, LootFilter},
        },
        ui::{number::format_number, tooltip::DynamicTooltipPosition},
    },
};

#[component]
pub fn LootQueue() -> impl IntoView {
    let conn: WebsocketContext = expect_context();
    let accessibility: AccessibilityContext = expect_context();
    let settings: SettingsContext = expect_context();
    let game_context: GameContext = expect_context();

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
                loot.state = LootState::Sold
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

    let hover_lock = RwSignal::new(false);

    let loot_state = move |loot_identifier| {
        game_context
            .queued_loot
            .read()
            .iter()
            .find(|l| l.identifier == loot_identifier)
            .map(|l| l.state)
            .unwrap_or_default()
    };

    let position_style = move |loot_identifier| {
        let index = game_context
            .queued_loot
            .read_untracked()
            .iter()
            .filter(|l| !l.state.has_disappeared() || l.identifier == loot_identifier)
            .rev()
            .position(|l| l.identifier == loot_identifier)
            .unwrap_or_default();
        format!("left: {}%;", 4 + index * 20)
    };

    let animation_style = move |loot_identifier| {
        let state = loot_state(loot_identifier);
        match state {
            LootState::Normal => "animation: loot-float 2.5s ease-in-out infinite;",
            LootState::WillDisappear => "animation: loot-vibrate 0.3s linear infinite;",
            LootState::HasDisappeared | LootState::Sold => {
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
                    let gold_price = loot.item_specs.gold_price;
                    let can_sell = !matches!(loot.item_specs.modifiers.rarity, ItemRarity::Unique);
                    let saved_position_style = RwSignal::new(position_style(loot.identifier));
                    Effect::new(move || {
                        if !hover_lock.get() {
                            let _ = game_context.queued_loot.read();
                            saved_position_style.set(position_style(loot.identifier));
                        }
                    });
                    view! {
                        <div style="animation: loot-drop 1.3s ease forwards;">
                            <div
                                class="
                                absolute bottom-0 w-[12%] aspect-[2/3]
                                transition-all duration-500 ease
                                pointer-events-none
                                will-change-transform
                                hover:z-10
                                "
                                style=move || {
                                    format!(
                                        "{} {}",
                                        animation_style(loot.identifier),
                                        saved_position_style.get(),
                                    )
                                }
                            >
                                <div
                                    class=move || {
                                        format!(
                                            "
                                            relative
                                            transition-all duration-200 ease-in-out 
                                            translate-y-1/2 hover:translate-y-1/4
                                            pointer-events-auto
                                            {}
                                            ",
                                            if settings.uses_surface_effects() {
                                                "drop-shadow-[0_0_10px_rgba(0,0,0,0.45)]"
                                            } else {
                                                ""
                                            },
                                        )
                                    }
                                    on:click={
                                        let pickup_loot = pickup_loot.clone();
                                        move |_| pickup_loot(loot.identifier)
                                    }
                                    on:mouseenter=move |_| hover_lock.set(true)
                                    on:mouseleave=move |_| hover_lock.set(false)

                                    on:contextmenu={
                                        let sell_loot = sell_loot.clone();
                                        move |_| {
                                            if !accessibility.is_on_mobile()
                                                && item_rarity != ItemRarity::Unique
                                            {
                                                sell_loot(loot.identifier);
                                                hover_lock.set(false);
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
                                        can_sell
                                    />
                                </div>
                            </div>

                            <div
                                class="absolute bottom-0 w-[12%] aspect-[2/3] z-30 pointer-events-none"
                                style=saved_position_style
                            >
                                <Show when=move || {
                                    matches!(loot_state(loot.identifier), LootState::Sold)
                                }>
                                    <div class="
                                    reward-float gold-text text-amber-400 text-lg xl:text-2xl text-shadow-md
                                    absolute left-1/2 top-[45%] transform -translate-y-1/2 -translate-x-1/2
                                    flex items-center gap-1 font-number">
                                        <span>+{format_number(gold_price)}</span>
                                        <img
                                            draggable="false"
                                            src=img_asset("ui/gold.webp")
                                            alt="Gold"
                                            class="h-[2em] aspect-square"
                                        />
                                    </div>
                                </Show>
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
        item_level,
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
        item_cooldown,
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

    if item_level
        .map(|item_level| !match rule_type {
            FilterRuleType::Pickup => item_specs.modifiers.level >= item_level,
            FilterRuleType::Sell => item_specs.modifiers.level <= item_level,
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

    if item_damages
        .map(|item_damages| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_damage = weapon_specs.average_damages();
                    match rule_type {
                        FilterRuleType::Pickup => weapon_damage < item_damages,
                        FilterRuleType::Sell => weapon_damage > item_damages,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_damage_physical
        .map(|item_damage_physical| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_damage = weapon_specs.average_damage_type(DamageType::Physical);
                    match rule_type {
                        FilterRuleType::Pickup => weapon_damage < item_damage_physical,
                        FilterRuleType::Sell => weapon_damage > item_damage_physical,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_damage_fire
        .map(|item_damage_fire| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_damage = weapon_specs.average_damage_type(DamageType::Fire);
                    match rule_type {
                        FilterRuleType::Pickup => weapon_damage < item_damage_fire,
                        FilterRuleType::Sell => weapon_damage > item_damage_fire,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_damage_poison
        .map(|item_damage_poison| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_damage = weapon_specs.average_damage_type(DamageType::Poison);
                    match rule_type {
                        FilterRuleType::Pickup => weapon_damage < item_damage_poison,
                        FilterRuleType::Sell => weapon_damage > item_damage_poison,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_damage_storm
        .map(|item_damage_storm| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_damage = weapon_specs.average_damage_type(DamageType::Storm);
                    match rule_type {
                        FilterRuleType::Pickup => weapon_damage < item_damage_storm,
                        FilterRuleType::Sell => weapon_damage > item_damage_storm,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_crit_chance
        .map(|item_crit_chance| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_crit_chance = weapon_specs.crit_chance.value.get() as f64;
                    match rule_type {
                        FilterRuleType::Pickup => weapon_crit_chance < item_crit_chance,
                        FilterRuleType::Sell => weapon_crit_chance > item_crit_chance,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_crit_damage
        .map(|item_crit_damage| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_crit_damage = *weapon_specs.crit_damage;
                    match rule_type {
                        FilterRuleType::Pickup => weapon_crit_damage < item_crit_damage,
                        FilterRuleType::Sell => weapon_crit_damage > item_crit_damage,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_cooldown
        .map(|item_cooldown| {
            item_specs
                .weapon_specs
                .as_ref()
                .map(|weapon_specs| {
                    let weapon_cooldown = weapon_specs.cooldown.get();
                    match rule_type {
                        FilterRuleType::Pickup => weapon_cooldown > item_cooldown,
                        FilterRuleType::Sell => weapon_cooldown < item_cooldown,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_armor
        .map(|item_armor| {
            item_specs
                .armor_specs
                .as_ref()
                .map(|armor_specs| {
                    let armor = *armor_specs.armor;
                    match rule_type {
                        FilterRuleType::Pickup => armor < item_armor,
                        FilterRuleType::Sell => armor > item_armor,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    if item_block
        .map(|item_block| {
            item_specs
                .armor_specs
                .as_ref()
                .map(|armor_specs| {
                    let block = armor_specs.block.get() as f64;
                    match rule_type {
                        FilterRuleType::Pickup => block < item_block,
                        FilterRuleType::Sell => block > item_block,
                    }
                })
                .unwrap_or(true)
        })
        .unwrap_or_default()
    {
        return false;
    }

    let effects = item_specs
        .modifiers
        .aggregate_effects(AffixEffectScope::Global, true)
        .0;
    for stat_filter in stat_filters {
        if let Some(((stat_type, stat_modifier), stat_value)) = stat_filter.as_ref()
            && !effects
                .get(&(stat_type.clone(), *stat_modifier, false))
                .map(|value| {
                    if *value == 0.0 {
                        return false;
                    };
                    stat_value
                        .map(|stat_value| match rule_type {
                            FilterRuleType::Pickup => *value >= stat_value,
                            FilterRuleType::Sell => *value <= stat_value,
                        })
                        .unwrap_or(true)
                })
                .unwrap_or_default()
        {
            return false;
        }
    }

    true
}
