use leptos::html::*;
use leptos::prelude::*;
use shared::data::item::LootState;

use crate::components::game::GameContext;
use crate::components::ui::tooltip::DynamicTooltipPosition;

use super::item_card::ItemCard;

#[component]
pub fn LootQueue() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    view! {
        <div class="relative w-full z-0">
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
                    to {
                        opacity: 0;
                        transform: scale(1.5) translateY(-200%);
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
                children=move |loot| {
                    let game_context = expect_context::<GameContext>();
                    let position_style = move || {
                        let index = game_context
                            .queued_loot
                            .read()
                            .iter()
                            .filter(|l| l.state != LootState::HasDisappeared)
                            .rev()
                            .position(|l| l.identifier == loot.identifier)
                            .unwrap_or_default();
                        format!("left: {}%;", 4 + index * 20)
                    };
                    let game_context = expect_context::<GameContext>();
                    let animation_style = move || {
                        let state = game_context
                            .queued_loot
                            .read()
                            .iter()
                            .find(|l| l.identifier == loot.identifier)
                            .map(|l| l.state)
                            .unwrap_or_default();
                        match state {
                            LootState::Normal => "animation: loot-float 2.5s ease-in-out infinite",
                            LootState::WillDisappear => {
                                "animation: loot-vibrate 0.3s linear infinite"
                            }
                            LootState::HasDisappeared => {
                                "animation: loot-pickup 0.3s ease forwards; pointer-events: none;"
                            }
                        }
                    };

                    view! {
                        <div style="animation: loot-drop 1.3s ease forwards;">
                            <div
                                class="
                                absolute bottom-0 w-[12%] shrink-0 transition-all duration-200 ease-in-out 
                                translate-y-1/2 hover:translate-y-1/4 
                                "
                                style=position_style
                            >
                                <div class="relative  " style=animation_style>
                                    <ItemCard
                                        item_specs=loot.item_specs
                                        tooltip_position=DynamicTooltipPosition::TopLeft
                                        class:shadow-lg
                                    />
                                </div>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}
