use leptos::prelude::*;

use crate::{
    assets::img_asset,
    components::settings::{GraphicsQuality, SettingsContext},
};

#[component]
pub fn BaseHeaderMenu(children: Children) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let stone_texture = img_asset("ui/dark_stone.webp");

    view! {
        <div
            class=move || {
                format!(
                    "relative z-50 flex justify-between items-center p-1 xl:p-2 h-auto border-b {}",
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "border-[#6c5734]/45 shadow-[0_8px_18px_rgba(0,0,0,0.45),inset_0_-1px_0_rgba(0,0,0,0.18)]"
                        }
                        GraphicsQuality::Medium => {
                            "border-[#6c5734]/50 shadow-[0_8px_18px_rgba(0,0,0,0.45)]"
                        }
                        GraphicsQuality::Low => "border-[#4f4532] bg-zinc-800",
                    },
                )
            }
            style=move || {
                match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        format!(
                            "
                        background-image:
                            linear-gradient(180deg, rgba(214, 165, 102, 0.04), rgba(0,0,0,0)),
                            url('{}');
                        background-blend-mode: screen, normal;
                        ",
                            stone_texture,
                        )
                    }
                    GraphicsQuality::Medium => {
                        format!(
                            "
                        background-image: url('{}');
                        ",
                            stone_texture,
                        )
                    }
                    GraphicsQuality::Low => String::new(),
                }
            }
        >
            {children()}
        </div>
    }
}
