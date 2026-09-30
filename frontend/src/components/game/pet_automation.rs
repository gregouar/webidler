use leptos::prelude::*;

use shared::data::pets::PetButton;

use crate::components::{game::GameContext, shared::pets::PetSprite};

pub const PET_CLICK_DURATION_MS: u64 = 560;

#[component]
pub fn AssignedPet(
    button: PetButton,
    #[prop(default = false)] flipped: bool,
    #[prop(default = Signal::derive(|| false), into)] pressed: Signal<bool>,
) -> impl IntoView {
    let game_context: GameContext = expect_context();

    view! {
        {move || {
            game_context
                .player_pets
                .read()
                .get(&button)
                .cloned()
                .map(|pet_id| {
                    view! {
                        <div class=move || {
                            format!(
                                "absolute z-2 bottom-0 xl:bottom-1 transition-transform duration-300 {} {}",
                                if pressed.get() { "translate-y-[2px]" } else { "" },
                                if flipped { "left-0 xl:left-1" } else { "right-0 xl:right-1" },
                            )
                        }>
                            <PetSprite pet_id flipped />
                        </div>
                    }
                })
        }}
    }
}
