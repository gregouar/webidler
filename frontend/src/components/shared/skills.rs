use leptos::prelude::*;

use shared::data::skill::{BaseSkillSpecs, SkillType};

use crate::{assets::img_asset, components::ui::progress_bars::CircularProgressBar};

#[component]
pub fn SkillProgressBar(
    skill_specs_base: BaseSkillSpecs,
    #[prop(into)] value: Signal<f64>,
    #[prop(into, default = 4)] bar_width: u8,
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(into,default = Signal::derive(|| false))] disabled: Signal<bool>,
    #[prop(optional)] icon_class: Option<&'static str>,
) -> impl IntoView {
    let tint_background = skill_type_progress_tint(skill_specs_base.skill_type);
    let icon_class = icon_class.unwrap_or("w-full h-full flex-no-shrink fill-current invert");

    view! {
        <CircularProgressBar
            value=value
            bar_color="oklch(55.4% 0.135 66.442)"
            reset=reset
            disabled=disabled
            bar_width=bar_width
            tint_background=tint_background
        >
            <img
                draggable="false"
                src=img_asset(&skill_specs_base.icon)
                alt=skill_specs_base.name.clone()
                class=icon_class
            />
        </CircularProgressBar>
    }
}

fn skill_type_progress_tint(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "from-[oklch(0.44_0.08_27.0)]",
        SkillType::Spell => "from-[oklch(0.44_0.07_240.0)]",
        SkillType::Curse => "from-[oklch(0.44_0.14_315.0)]",
        SkillType::Blessing => "from-[oklch(0.60_0.07_85.0)]",
        SkillType::Other => "from-stone-600",
    }
}
