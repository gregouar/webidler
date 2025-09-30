use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{data::loot::LootState, messages::client::PickUpLootMessage};

use crate::components::{
    accessibility::AccessibilityContext, game::GameContext, shared::item_card::ItemCard,
    ui::tooltip::DynamicTooltipPosition, websocket::WebsocketContext,
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
                if let Some(loot) = queued_loot.get_mut(loot_identifier as usize) {
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
            if let Some(loot) = queued_loot.get_mut(loot_identifier as usize) {
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

    view! {
        <div class="relative w-full z-0 pr-4">
            <style>
                "
                @keyframes loot-drop {
                    0% { transform: translateY(-100px); opacity: 0; }
                    100% { transform: translateY(0px); opacity: 1; }
                }
                @keyframes loot-float {
                    0%, 100% { transform: translateY(0); }
                    50% { transform: translateY(-4%); }
                }
                @keyframes loot-pickup {
                    99% {
                        opacity: 0;
                        transform: scale(1.5) translateY(-200%);
                    }
                    100% {
                        opacity: 0;
                        transform: scale(1.5) translateY(-200%);
                        visibility: hidden;
                    }
                }
                @keyframes loot-vibrate {
                0%, 100% { transform: translate(0, 0); }
                20% { transform: translate(-1px, 1px); }
                40% { transform: translate(1px, -1px); }
                60% { transform: translate(-1px, -1px); }
                80% { transform: translate(1px, 1px); }
                }
                "
            </style>
            <For
                each=move || game_context.queued_loot.get().into_iter()
                key=|loot| loot.identifier
                let(loot)
            >
                <div style="animation: loot-drop 1.3s ease forwards;">
                    <div
                        class="
                        absolute bottom-0 w-[12%] aspect-[2/3]
                        transition-all duration-500 ease
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
                                    if !accessibility.is_on_mobile() {
                                        sell_loot(loot.identifier);
                                    }
                                }
                            }
                        >
                            <ItemCard
                                item_specs=Arc::new(loot.item_specs)
                                tooltip_position=DynamicTooltipPosition::TopLeft
                                class:shadow-lg
                            />
                        </div>
                    </div>
                </div>
            </For>
        </div>
    }
}
