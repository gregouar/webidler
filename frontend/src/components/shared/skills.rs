use leptos::prelude::*;

use shared::data::skill::{BaseSkillSpecs, SkillSpecs, SkillType};

use crate::{assets::img_asset, components::ui::progress_bars::CircularProgressBar};

pub const SKILL_PROGRESS_RING_COLOR: &str = "#a65f00";

pub fn skill_specs_from_base(skill_id: String, base_skill_specs: &BaseSkillSpecs) -> SkillSpecs {
    SkillSpecs {
        skill_id,
        name: base_skill_specs.name.clone(),
        icon: base_skill_specs.icon.clone(),
        description: base_skill_specs.description.clone(),
        skill_type: base_skill_specs.skill_type,
        cooldown: base_skill_specs.cooldown.into(),
        mana_cost: base_skill_specs.mana_cost.into(),
        targets: base_skill_specs.targets.clone(),
        triggers: base_skill_specs.triggers.clone(),
        level_modifier: 0,
        ignore_stat_effects: base_skill_specs.ignore_stat_effects.clone(),
    }
}

#[component]
pub fn SkillProgressBar(
    skill_type: SkillType,
    skill_icon: String,
    #[prop(into)] value: Signal<f64>,
    #[prop(into, default = 4)] bar_width: u8,
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(into,default = Signal::derive(|| false))] disabled: Signal<bool>,
    #[prop(optional)] icon_class: Option<&'static str>,
) -> impl IntoView {
    let tint_background = skill_type_progress_tint(skill_type);
    let icon_class = icon_class.unwrap_or("w-full h-full flex-no-shrink fill-current invert");

    view! {
        <CircularProgressBar
            value=value
            bar_color=SKILL_PROGRESS_RING_COLOR
            reset=reset
            disabled=disabled
            bar_width=bar_width
            tint_background=tint_background
        >
            <img draggable="false" src=img_asset(&skill_icon) alt="skill" class=icon_class />
        </CircularProgressBar>
    }
}

fn skill_type_progress_tint(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "from-[#783f42]",
        SkillType::Spell => "from-[#3e5667]",
        SkillType::Curse => "from-[#6f3486]",
        SkillType::Blessing => "from-[#967d46]",
        SkillType::Other => "from-stone-600",
    }
}
