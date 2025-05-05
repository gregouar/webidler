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
            <div
                class="absolute left-4 bottom-0 flex flex-row justify-around gap-2 overflow-visible z-20"
                style="transform: translateY(60%);"
            >
                <For
                    each=move || game_context.queued_loot.get().into_iter().rev()
                    key=|loot| loot.identifier
                    children=move |loot| {
                        view! {
                            <div
                                class="
                                relative w-[12%] shrink-0 transition-transform duration-200 ease-in-out 
                                hover:z-20 hover:-translate-y-1/2
                                "
                                style=move || match loot.state {
                                    LootState::Normal => {
                                        "animation: loot-float 2.5s ease-in-out infinite"
                                    }
                                    LootState::WillDisappear => {
                                        "animation: loot-vibrate 0.3s linear infinite"
                                    }
                                    LootState::HasDisappeared => {
                                        "animation: loot-pickup 0.3s ease forwards"
                                    }
                                }
                            >
                                <div>
                                    <ItemCard
                                        item_specs=loot.item_specs
                                        tooltip_position=DynamicTooltipPosition::TopLeft
                                        class:shadow-lg
                                    />
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
