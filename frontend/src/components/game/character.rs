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
    #[prop(into)] just_blocked: Signal<bool>,
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

    let show_block_effect = RwSignal::new(false);

    Effect::new(move |_| {
        if just_blocked.get() {
            show_block_effect.set(true);
        }
    });

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
            
            @keyframes shield_flash {
                0% { opacity: 0; transform: scale(0.35) translateY(60%); }
                50% { opacity: 0.8; transform: scale(.55) translateY(40%); }
                100% { opacity: 0; transform: scale(.65) translateY(60%); }
            }
            "
        </style>
        <div class=move || {
            format!(
                "flex items-center justify-center h-full w-full relative overflow-hidden {}",
                is_dead_img_effect(),
            )
        }>
            <div class="h-full w-full" style=crit_animation_style>
                <img
                    src=img_asset(&image_uri)
                    alt=character_name
                    class=move || {
                        format!(
                            "border-8 border-double border-stone-500 object-cover h-full w-full",
                        )
                    }
                />
                <div class=move || {
                    format!("absolute inset-0 -none  {}", just_hurt_class())
                }></div>
            </div>

            {move || {
                if show_block_effect.get() {
                    Some(
                        view! {
                            <img
                                src=img_asset("effects/block.svg")
                                class="absolute inset-0 w-object-contain pointer-events-none"
                                on:animationend=move |_| show_block_effect.set(false)
                                style="animation: shield_flash 0.5s ease-out"
                            />
                        },
                    )
                } else {
                    None
                }
            }}
        </div>
    }
}
