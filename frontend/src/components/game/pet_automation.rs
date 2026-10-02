use leptos::prelude::*;
use leptos_use::{UseIntervalOptions, use_interval_with_options};

use shared::data::pets::PetButton;

use crate::components::{game::GameContext, shared::pets::PetSprite};

pub const PET_CLICK_DURATION_MS: u64 = 560;
pub const PET_CLICK_DELAY_MS: u64 = 1000;

#[component]
pub fn PetAutomation(
    pet_button: PetButton,
    #[prop(default = false)] flipped: bool,
    callback: Callback<()>,
    #[prop(into)] disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let game_context: GameContext = expect_context();
    let pet_clicking = RwSignal::new(false);

    let _ = use_interval_with_options(
        PET_CLICK_DELAY_MS,
        UseIntervalOptions::default().callback(move |_| {
            if game_context
                .player_pets
                .read_untracked()
                .contains_key(&pet_button)
                && !disabled.get_untracked()
                && !game_context.area_specs.read_untracked().training
            {
                pet_clicking.set(true);
                set_timeout(
                    move || {
                        if !disabled.get_untracked() {
                            callback.run(());
                        }
                        pet_clicking.set(false);
                    },
                    std::time::Duration::from_millis(PET_CLICK_DURATION_MS),
                );
            }
        }),
    );

    view! {
        <div class="relative" class:button-auto-pressed=pet_clicking>
            <AssignedPet pet_button flipped pressed=Signal::from(pet_clicking) />
            {children()}
        </div>
    }
}

#[component]
pub fn AssignedPet(
    pet_button: PetButton,
    #[prop(default = false)] flipped: bool,
    #[prop(default = Signal::derive(|| false), into)] pressed: Signal<bool>,
) -> impl IntoView {
    let game_context: GameContext = expect_context();

    view! {
        {move || {
            game_context
                .player_pets
                .read()
                .get(&pet_button)
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
