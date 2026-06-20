use std::{sync::Arc, time::Duration};

use leptos::{html::*, prelude::*};

use shared::data::{
    skill::{BaseSkillSpecs, SkillSpecs, SkillType},
    skill_mastery::SkillMasteryState,
};

use crate::{
    assets::img_asset,
    components::{
        data_context::DataContext,
        settings::{GraphicsQuality, SettingsContext},
        shared::tooltips::SkillTooltip,
        ui::{
            number::format_number,
            progress_bars::{CircularProgressBar, HorizontalProgressBar},
            tooltip::{
                DynamicTooltipContext, DynamicTooltipPosition, StaticTooltip, StaticTooltipPosition,
            },
        },
    },
};

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
pub fn SkillMasteryCard(
    #[prop(optional)] skill_specs: Option<SkillSpecs>,
    #[prop(optional)] skill_mastery_state: Option<SkillMasteryState>,
    #[prop(default = 0)] level_delta: u16,
    #[prop(optional)] experience_gained: Option<f64>,
    #[prop(optional)] empty_label: Option<String>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let settings = expect_context::<SettingsContext>();
    let data_context = expect_context::<DataContext>();

    let max_level = skill_specs.as_ref().and_then(|skill_specs| {
        data_context
            .skill_mastery_specs
            .read()
            .get(&skill_specs.skill_id)
            .map(|specs| specs.max_level)
    });
    let level = skill_mastery_state.as_ref().map(SkillMasteryState::level);
    let is_max_level = level
        .zip(max_level)
        .map(|(level, max_level)| level >= max_level)
        .unwrap_or(true);
    let has_progress = skill_mastery_state.is_some();
    let relative_experience = skill_mastery_state
        .as_ref()
        .map(SkillMasteryState::relative_experience);
    let next_level_cost = skill_mastery_state
        .as_ref()
        .map(SkillMasteryState::next_level_cost);
    let current_progress =
        relative_experience
            .zip(next_level_cost)
            .map(|(relative_experience, next_level_cost)| {
                mastery_progress(relative_experience, next_level_cost)
            });
    let current_progress = if is_max_level {
        Some(100.0)
    } else {
        current_progress
    };
    let previous_progress = if is_max_level {
        100.0
    } else {
        match (skill_mastery_state.as_ref(), experience_gained) {
            (Some(skill_mastery_state), Some(experience_gained)) => {
                let mut previous_skill_mastery_state = skill_mastery_state.clone();
                previous_skill_mastery_state.experience =
                    (previous_skill_mastery_state.experience - experience_gained).max(0.0);
                if max_level
                    .map(|max_level| previous_skill_mastery_state.level() >= max_level)
                    .unwrap_or_default()
                {
                    100.0
                } else {
                    mastery_progress(
                        previous_skill_mastery_state.relative_experience(),
                        previous_skill_mastery_state.next_level_cost(),
                    )
                }
            }
            (Some(_), None) => current_progress.unwrap_or_default(),
            (None, _) => 0.0,
        }
    };
    let progress_reset = RwSignal::new(false);
    let progress = RwSignal::new(previous_progress);
    if let (Some(current_progress), Some(_)) = (current_progress, experience_gained) {
        if level_delta > 0 && !is_max_level {
            set_timeout(move || progress.set(100.0), Duration::from_millis(500));
            set_timeout(
                move || {
                    progress.set(0.0);
                    progress_reset.set(true);
                },
                Duration::from_millis(800),
            );
            set_timeout(
                move || progress.set(current_progress),
                Duration::from_millis(900),
            );
        } else {
            set_timeout(
                move || progress.set(current_progress),
                Duration::from_millis(500),
            );
        }
    }

    let skill_type = skill_specs
        .as_ref()
        .map(|skill| skill.skill_type)
        .unwrap_or(SkillType::Other);
    let skill_name = skill_specs
        .as_ref()
        .map(|skill| skill.name.clone())
        .unwrap_or_else(|| empty_label.unwrap_or_else(|| "Empty Slot".to_string()));
    let skill_icon = skill_specs.as_ref().map(|skill| skill.icon.clone());
    let tooltip_specs = skill_specs.map(Arc::new);
    let is_empty = skill_icon.is_none();
    let cursor_class = if on_click.is_some() {
        "cursor-pointer"
    } else {
        "cursor-default"
    };

    view! {
        <div
            class=move || {
                let quality = settings.graphics_quality();
                let base = format!(
                    "relative h-full {} overflow-clip group border rounded-[9px]
                    {} flex flex-col items-center {}
                    transition-[border-color,background-color,box-shadow,transform] duration-150 {}",
                    if compact { "min-h-0" } else { "min-h-[13.5rem]" },
                    if compact { "px-1 py-2" } else { "px-3 py-3 xl:px-4 xl:py-4" },
                    if compact { "gap-1" } else { "gap-3" },
                    cursor_class,
                );
                let quality_class = match quality {
                    GraphicsQuality::High => {
                        "bg-[linear-gradient(180deg,rgba(226,193,122,0.045),rgba(0,0,0,0.02)_32%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
                        shadow-[0_4px_10px_rgba(0,0,0,0.26)]"
                    }
                    GraphicsQuality::Medium => {
                        "bg-[linear-gradient(180deg,rgba(200,164,96,0.04),rgba(0,0,0,0.02)_34%,rgba(0,0,0,0.12)_100%),linear-gradient(135deg,rgba(38,37,43,0.98),rgba(18,18,22,1))]
                        shadow-md"
                    }
                    GraphicsQuality::Low => {
                        "bg-[linear-gradient(180deg,rgba(177,143,85,0.035),rgba(0,0,0,0.03)_35%,rgba(0,0,0,0.12)_100%),linear-gradient(135deg,rgba(36,36,41,0.98),rgba(19,19,23,1))]"
                    }
                };
                let interactive_class = if on_click.is_some() {
                    match quality {
                        GraphicsQuality::High => {
                            "hover:border-[#7b6440] hover:-translate-y-[1px] active:translate-y-[2px]"
                        }
                        GraphicsQuality::Medium => {
                            "hover:border-[#715a38] hover:-translate-y-[1px] active:translate-y-[2px]"
                        }
                        GraphicsQuality::Low => "hover:border-[#675236] active:translate-y-[1px]",
                    }
                } else {
                    ""
                };
                format!(
                    "{} {} {} {}",
                    base,
                    quality_class,
                    match quality {
                        GraphicsQuality::High => "border-[#3b3428]",
                        GraphicsQuality::Medium => "border-[#4a3e2b]",
                        GraphicsQuality::Low => "border-[#554631]",
                    },
                    interactive_class,
                )
            }
            on:click=move |_| {
                if let Some(on_click) = on_click {
                    on_click.run(());
                }
            }
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <div class="pointer-events-none absolute inset-[1px] rounded-[8px] border border-white/5"></div>
            </Show>
            <Show when=move || settings.uses_heavy_effects()>
                <div class=format!(
                    "pointer-events-none absolute inset-x-4 top-[1px] h-px bg-gradient-to-r from-transparent {} to-transparent",
                    skill_type_top_glow(skill_type),
                )></div>
            </Show>

            <SkillBadge skill_type icon=skill_icon alt=skill_name.clone() tooltip_specs />

            <div class=if compact {
                "w-full min-w-0 text-center space-y-1"
            } else {
                "w-full min-w-0 text-center space-y-2"
            }>
                {(!compact)
                    .then(|| {
                        view! {
                            <div class=move || {
                                format!(
                                    "text-sm xl:text-base font-bold text-center font-display text-shadow-lg/100 shadow-gray-950 leading-tight {}",
                                    if is_empty { "text-zinc-500" } else { "text-zinc-100" },
                                )
                            }>{skill_name}</div>
                        }
                    })}
                <div class="min-h-4 text-xs xl:text-sm font-semibold text-violet-300">
                    {level
                        .map(|level| {
                            let level_delta = level_delta.min(level);
                            view! {
                                "Level "
                                {(level_delta > 0)
                                    .then(|| {
                                        view! {
                                            <span class="ml-1 text-zinc-500">
                                                {level - level_delta}" → "
                                            </span>
                                        }
                                    })}
                                {level}
                            }
                                .into_any()
                        })}
                </div>
                <div class="min-h-3">
                    {has_progress
                        .then(|| {
                            view! {
                                <StaticTooltip
                                    tooltip=move || {
                                        mastery_card_xp_tooltip(
                                            relative_experience.unwrap_or_default(),
                                            next_level_cost.unwrap_or_default(),
                                            is_max_level,
                                            experience_gained,
                                        )
                                    }
                                    position=StaticTooltipPosition::Top
                                >
                                    <HorizontalProgressBar
                                        class="h-3"
                                        bar_color=if is_max_level {
                                            "bg-gradient-to-b from-zinc-400 to-zinc-600"
                                        } else {
                                            "bg-gradient-to-b from-violet-200 to-violet-600"
                                        }
                                        value=Signal::derive(move || progress.get())
                                        reset=progress_reset
                                    />
                                </StaticTooltip>
                            }
                        })}
                </div>
            </div>
        </div>
    }
}

fn mastery_progress(relative_experience: f64, next_level_cost: f64) -> f32 {
    if next_level_cost > 0.0 {
        (relative_experience / next_level_cost * 100.0).clamp(0.0, 100.0) as f32
    } else {
        0.0
    }
}

#[component]
pub fn SkillBadge(
    skill_type: SkillType,
    icon: Option<String>,
    alt: String,
    #[prop(into,default = Signal::derive(|| false))] selected: Signal<bool>,
    #[prop(default = None)] tooltip_specs: Option<Arc<SkillSpecs>>,
) -> impl IntoView {
    let settings = expect_context::<SettingsContext>();
    let tooltip_context = use_context::<DynamicTooltipContext>();
    let empty = icon.is_none();
    let icon = icon.map(|icon| img_asset(&icon));
    let tooltip_id = RwSignal::new(0);
    let show_tooltip = {
        let tooltip_specs = tooltip_specs.clone();
        move || {
            if let (Some(tooltip_context), Some(tooltip_specs)) =
                (tooltip_context, tooltip_specs.clone())
            {
                tooltip_id.set(tooltip_context.set_content(
                    move || view! { <SkillTooltip skill_specs=tooltip_specs.clone() /> }.into_any(),
                    DynamicTooltipPosition::Auto,
                ));
            }
        }
    };
    let hide_tooltip = move || {
        if let Some(tooltip_context) = tooltip_context {
            tooltip_context.hide(tooltip_id.get_untracked());
        }
    };
    on_cleanup(hide_tooltip);

    view! {
        <div
            class=move || {
                let quality = settings.graphics_quality();
                let frame_background = match quality {
                    GraphicsQuality::High => {
                        "bg-[linear-gradient(180deg,rgba(214,177,102,0.1),rgba(0,0,0,0.2)),linear-gradient(180deg,rgba(43,40,46,0.96),rgba(20,19,23,1))]"
                    }
                    GraphicsQuality::Medium => {
                        "bg-[linear-gradient(180deg,rgba(214,177,102,0.08),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(41,38,44,0.96),rgba(21,20,24,1))]"
                    }
                    GraphicsQuality::Low => {
                        "bg-[linear-gradient(180deg,rgba(39,37,42,0.98),rgba(20,19,23,1))]"
                    }
                };
                let frame_shadow = if selected.get() {
                    match quality {
                        GraphicsQuality::High | GraphicsQuality::Medium => {
                            skill_type_selected_frame_glow(skill_type)
                        }
                        GraphicsQuality::Low => skill_type_selected_frame_glow_low(skill_type),
                    }
                } else {
                    match quality {
                        GraphicsQuality::High => "shadow-[0_4px_12px_rgba(0,0,0,0.58)]",
                        GraphicsQuality::Medium => "shadow-[0_3px_10px_rgba(0,0,0,0.48)]",
                        GraphicsQuality::Low => "",
                    }
                };
                format!(
                    "relative flex aspect-square w-full max-w-20 xl:max-w-24 items-center justify-center rounded-full
                    overflow-clip border {} {} {}",
                    skill_type_frame_border(skill_type),
                    frame_background,
                    frame_shadow,
                )
            }
            on:touchstart={
                let show_tooltip = show_tooltip.clone();
                move |ev| {
                    ev.stop_propagation();
                    show_tooltip();
                }
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }
            on:mouseenter=move |_| show_tooltip()
            on:mouseleave=move |_| hide_tooltip()
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <div class="pointer-events-none absolute inset-[1px] rounded-full border border-[#d5b16d]/16"></div>
            </Show>
            <div class=move || {
                format!(
                    "pointer-events-none absolute inset-[3px] rounded-full border {} {}",
                    match settings.graphics_quality() {
                        GraphicsQuality::High => "border-[#6d532e]/70",
                        GraphicsQuality::Medium => "border-[#6b5430]/55",
                        GraphicsQuality::Low => "border-[#5a4628]/55",
                    },
                    if empty {
                        "bg-[radial-gradient(circle_at_50%_40%,rgba(38,37,43,0.72),rgba(15,15,18,0.98)_72%)]"
                    } else if selected.get() {
                        "bg-[radial-gradient(circle_at_50%_38%,rgba(142,132,118,0.9),rgba(56,47,41,0.88)_48%,rgba(20,18,24,0.98)_78%)]"
                    } else {
                        "bg-[radial-gradient(circle_at_50%_40%,rgba(92,88,98,0.72),rgba(20,18,24,0.98)_72%)]"
                    },
                )
            }></div>
            <div class=move || {
                format!(
                    "pointer-events-none absolute inset-[6px] rounded-full bg-radial {} to-transparent",
                    if selected.get() {
                        skill_type_selected_inner_glow(skill_type)
                    } else {
                        skill_type_inner_glow(skill_type)
                    },
                )
            }></div>
            {icon
                .map(|icon| {
                    view! {
                        <img
                            draggable="false"
                            src=icon
                            alt=alt.clone()
                            class=move || {
                                format!(
                                    "relative z-10 h-[58%] w-[58%] flex-no-shrink fill-current invert {}",
                                    if settings.uses_surface_effects() {
                                        "drop-shadow-[0_2px_2px_rgba(0,0,0,0.72)]"
                                    } else {
                                        ""
                                    },
                                )
                            }
                        />
                    }
                        .into_any()
                })
                .unwrap_or_else(|| {

                    view! {
                        <div class="relative z-10 text-3xl xl:text-4xl font-display text-zinc-700">
                            "+"
                        </div>
                    }
                        .into_any()
                })}
        </div>
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

fn mastery_card_xp_tooltip(
    relative_experience: f64,
    next_level_cost: f64,
    is_max_level: bool,
    experience_gained: Option<f64>,
) -> AnyView {
    view! {
        "Experience: "
        {if is_max_level {
            "Full".to_string()
        } else {
            format!("{}/{}", format_number(relative_experience), format_number(next_level_cost))
        }}
        {experience_gained
            .map(|experience_gained| {
                view! {
                    " (+"
                    {format_number(experience_gained)}
                    ")"
                }
            })}
    }
    .into_any()
}

fn skill_type_frame_border(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "border-[#8d5644]",
        SkillType::Spell => "border-[#536f95]",
        SkillType::Curse => "border-[#6f5697]",
        SkillType::Blessing => "border-[#8a6d33]",
        SkillType::Other => "border-[#5e6470]",
    }
}

fn skill_type_inner_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "from-red-400/18",
        SkillType::Spell => "from-sky-400/18",
        SkillType::Curse => "from-purple-400/18",
        SkillType::Blessing => "from-amber-300/18",
        SkillType::Other => "from-slate-300/14",
    }
}

fn skill_type_selected_frame_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "shadow-[0_0_18px_rgba(248,113,113,0.5),0_4px_14px_rgba(0,0,0,0.58)]",
        SkillType::Spell => "shadow-[0_0_18px_rgba(56,189,248,0.5),0_4px_14px_rgba(0,0,0,0.58)]",
        SkillType::Curse => "shadow-[0_0_18px_rgba(192,132,252,0.5),0_4px_14px_rgba(0,0,0,0.58)]",
        SkillType::Blessing => {
            "shadow-[0_0_18px_rgba(252,211,77,0.52),0_4px_14px_rgba(0,0,0,0.58)]"
        }
        SkillType::Other => "shadow-[0_0_18px_rgba(203,213,225,0.42),0_4px_14px_rgba(0,0,0,0.58)]",
    }
}

fn skill_type_selected_frame_glow_low(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "shadow-[0_0_12px_rgba(248,113,113,0.38)]",
        SkillType::Spell => "shadow-[0_0_12px_rgba(56,189,248,0.38)]",
        SkillType::Curse => "shadow-[0_0_12px_rgba(192,132,252,0.38)]",
        SkillType::Blessing => "shadow-[0_0_12px_rgba(252,211,77,0.4)]",
        SkillType::Other => "shadow-[0_0_12px_rgba(203,213,225,0.32)]",
    }
}

fn skill_type_selected_inner_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "from-red-300/42",
        SkillType::Spell => "from-sky-300/42",
        SkillType::Curse => "from-purple-300/42",
        SkillType::Blessing => "from-amber-200/44",
        SkillType::Other => "from-slate-200/34",
    }
}

fn skill_type_top_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "via-red-400/40",
        SkillType::Spell => "via-sky-400/40",
        SkillType::Curse => "via-purple-400/40",
        SkillType::Blessing => "via-amber-300/40",
        SkillType::Other => "via-slate-300/40",
    }
}
