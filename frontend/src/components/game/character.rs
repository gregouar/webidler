use leptos::html::*;
use leptos::prelude::*;

use crate::assets::img_asset;

#[component]
pub fn CharacterPortrait(
    image_uri: String,
    character_name: String,
    #[prop(into)] just_hurt: Signal<bool>,
    #[prop(into)] is_dead: Signal<bool>,
) -> impl IntoView {
    let just_hurt_class = move || {
        if just_hurt.get() {
            "transition-all ease duration-100 just_hurt_effect"
        } else {
            "transition-all ease duration-1000"
        }
    };

    let is_dead_img_effect = move || {
        if is_dead.get() {
            "transition-all duration-1000 saturate-0 brightness-1"
        } else {
            ""
        }
    };

    view! {
        <style>
            // TODO
            // "
            // .just_hurt_effect {
            // --shadow-size: calc(10%);
            // box-shadow: inset 0 0 var(--shadow-size) rgba(192, 0, 0, 1.0);
            // }
            // "
            "
                .just_hurt_effect {
                    box-shadow: inset 0 0 32px rgba(192, 0, 0, 1.0);
                }
            "
        </style>
        <div class="flex-1 h-full w-full relative">
            <img
                src=img_asset(&image_uri)
                alt=character_name
                class=move || {
                    format!(
                        "border-8 border-double border-stone-500 object-cover aspect-square {}",
                        is_dead_img_effect(),
                    )
                }
            />
            <div class=move || {
                format!("absolute inset-0 pointer-events-none  {}", just_hurt_class())
            }></div>
        </div>
    }
}
