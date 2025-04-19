use leptos::html::*;
use leptos::prelude::*;

#[component]
pub fn CharacterPortrait(
    image_asset: String,
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
            "saturate-0 brightness-1"
        } else {
            ""
        }
    };

    view! {
        <div class="flex-1 h-full relative">
            <style>"
                .just_hurt_effect {
                    box-shadow: inset 0 0 64px rgba(192, 0, 0, 1.0);
                }
            "</style>
            <img
                src={format!("./assets/{}",image_asset)}
                alt=character_name
                class=move || format!("border-8 border-double border-stone-500 transition object-cover aspect-square duration-1000 {}", is_dead_img_effect())
            />
            <div
                class=move || format!("absolute inset-0 pointer-events-none  {}",just_hurt_class())
            >
            </div>
        </div>
    }
}
