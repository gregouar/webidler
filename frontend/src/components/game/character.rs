use std::time::Duration;

use leptos::html::*;
use leptos::prelude::*;

use crate::assets::img_asset;

#[component]
pub fn CharacterPortrait(
    image_uri: String,
    character_name: String,
    #[prop(into)] just_hurt: Signal<bool>,
    #[prop(into)] just_hurt_crit: Signal<bool>,
    #[prop(into)] is_dead: Signal<bool>,
) -> impl IntoView {
    let just_hurt_class = move || {
        if just_hurt.get() {
            "transition-all ease duration-100 just_hurt_effect"
        } else {
            "transition-all ease duration-1000"
        }
    };

    // TODO: just critically hurt

    let is_dead_img_effect = move || {
        if is_dead.get() {
            "transition-all duration-1000 saturate-0 brightness-1
            [transform:rotateX(180deg)]"
        } else {
            "transition-all duration-1000"
        }
    };

    let crit_hit = RwSignal::new(false);

    Effect::new(move |_| {
        if just_hurt_crit.get() {
            crit_hit.set(true);
            set_timeout(move || crit_hit.set(false), Duration::from_millis(500));
        }
    });

    let crit_animation_style = move || {
        if crit_hit.get() {
            "animation: shake 0.5s linear infinite;"
        } else {
            ""
        }
    };

    view! {
        <style>
            "
            .just_hurt_effect {
                box-shadow: inset 0 0 32px rgba(192, 0, 0, 1.0);
            }
            
            @keyframes shake {
                0%, 100% { transform: translate(0, 0) rotate(0); }
                25% { transform: translate(-4px, 2px) rotate(-2deg); }
                50% { transform: translate(4px, -2px) rotate(2deg); }
                75% { transform: translate(-3px, 1px) rotate(-1deg); }
            }
            "
        </style>
        <div
            class=move || {
                format!(
                    "flex items-center justify-center h-full w-full relative {}",
                    is_dead_img_effect(),
                )
            }
            style=crit_animation_style
        >
            <img
                src=img_asset(&image_uri)
                alt=character_name
                class=move || {
                    format!("border-8 border-double border-stone-500 object-cover h-full w-full")
                }
            />
            <div class=move || { format!("absolute inset-0 -none  {}", just_hurt_class()) }></div>
        </div>
    }
}
