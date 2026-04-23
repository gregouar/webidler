use std::{collections::HashMap, sync::Arc};

use leptos::{html::*, prelude::*};

use shared::{
    data::skill::{BaseSkillSpecs, SkillType},
    messages::client::BuySkillMessage,
};
use strum::IntoEnumIterator;

use crate::{
    assets::img_asset,
    components::{
        data_context::DataContext,
        game::{game_context::GameContext, websocket::WebsocketContext},
        settings::{GraphicsQuality, SettingsContext},
        shared::{
            resources::GoldCounter,
            skills::skill_specs_from_base,
            tooltips::SkillTooltip,
        },
        ui::{
            buttons::FancyButton,
            card::{CardHeader, CardInset, MenuCard},
            menu_panel::MenuPanel,
            tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
        },
    },
};

#[component]
pub fn SkillsPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open=open w_full=false h_full=false class:items-center>
            <MenuCard class="max-w-6xl mx-auto">
                <CardHeader title="Buy New Skill" on_close=move || open.set(false) />
                // flex-1 overflow-auto max-h-[65vh]
                <SkillShop open=open />
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
pub fn SkillShop(open: RwSignal<bool>) -> impl IntoView {
    let game_context: GameContext = expect_context();
    let data_context: DataContext = expect_context();

    let selected_skill = RwSignal::new(None::<String>);
    let disable_confirm = Signal::derive(move || selected_skill.get().is_none());

    let buy_skill = {
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if let Some(skill_id) = selected_skill.get() {
                conn.send(&BuySkillMessage { skill_id }.into());
            }
            open.set(false);
        }
    };

    let available_skills = Memo::new(move |_| {
        let mut skills = data_context.skill_specs.get().into_iter().fold(
            HashMap::<_, Vec<_>>::new(),
            |mut acc, skill| {
                acc.entry(skill.1.skill_type).or_default().push(skill);
                acc
            },
        );

        for section in skills.values_mut() {
            section.sort_by_key(|(_, skill_specs)| skill_specs.name.clone());
        }

        skills
    });

    view! {
        <CardInset>
            {move || {
                SkillType::iter()
                    .map(move |skill_type| {
                        let available_skills = available_skills
                            .read()
                            .get(&skill_type)
                            .cloned()
                            .unwrap_or_default();
                        let unbought_skills = available_skills
                            .into_iter()
                            .filter(|(skill_id, _)| {
                                !game_context
                                    .player_base_specs
                                    .read_untracked()
                                    .skills
                                    .contains_key(skill_id)
                            })
                            .collect::<Vec<_>>();
                        (!unbought_skills.is_empty())
                            .then(|| {
                                view! {
                                    <div class="space-y-3 xl:space-y-4">
                                        <div class="flex items-center gap-3 px-1">
                                            <div class=format!(
                                                "h-[2px] flex-1 rounded-full bg-gradient-to-r from-transparent {} to-transparent",
                                                skill_type_glow(skill_type),
                                            )></div>
                                            <h3 class=format!(
                                                "font-display text-sm xl:text-base tracking-[0.14em] uppercase {}",
                                                skill_type_title_color(skill_type),
                                            )>
                                                {skill_type_title(skill_type)}
                                            </h3>
                                            <div class=format!(
                                                "h-[2px] flex-1 rounded-full bg-gradient-to-r from-transparent {} to-transparent",
                                                skill_type_glow(skill_type),
                                            )></div>
                                        </div>

                                        <div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-2 xl:gap-3">
                                            <For
                                                each=move || unbought_skills.clone().into_iter()
                                                key=|(skill_id, _)| skill_id.clone()
                                                let:((skill_id, skill_specs))
                                            >
                                                <SkillCard
                                                    skill_id=skill_id.clone()
                                                    skill_specs=skill_specs.clone()
                                                    selected=selected_skill
                                                />
                                            </For>
                                        </div>
                                    </div>
                                }
                            })
                    })
                    .collect::<Vec<_>>()
            }}
        </CardInset>

        <div class="flex items-center justify-center">
            <FancyButton disabled=disable_confirm on:click=buy_skill class="py-2 px-4">
                <span class="flex items-center gap-2">
                    "Confirm buying selected skill for"
                    <GoldCounter value=Signal::derive(move || {
                        game_context.player_base_specs.read().buy_skill_cost
                    }) />
                </span>
            </FancyButton>
        </div>
    }
}
#[component]
fn SkillCard(
    skill_id: String,
    skill_specs: BaseSkillSpecs,
    selected: RwSignal<Option<String>>,
) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let settings = expect_context::<SettingsContext>();

    let is_selected = Signal::derive({
        let skill_id = skill_id.clone();
        move || selected.get().map(|s| s == skill_id).unwrap_or(false)
    });

    let was_last_bought = Memo::new({
        let skill_id = skill_id.clone();
        move |_| game_context.last_skills_bought.read().contains(&skill_id)
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let show_tooltip = {
        let skill_specs = Arc::new(skill_specs_from_base(skill_id.clone(), &skill_specs));
        move || {
            let skill_specs = skill_specs.clone();
            tooltip_context.set_content(
                move || view! { <SkillTooltip skill_specs=skill_specs.clone() /> }.into_any(),
                DynamicTooltipPosition::Auto,
            );
        }
    };

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let hide_tooltip = move || tooltip_context.hide();
    let skill_type = skill_specs.skill_type;

    view! {
        <div
            class=move || {
                let quality = settings.graphics_quality();
                let base = "relative overflow-clip group border rounded-[9px]
                    px-3 py-3 xl:px-4 xl:py-4 flex flex-col items-center gap-3
                    transition-[border-color,background-color,box-shadow,transform] duration-150 cursor-pointer";
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

                if is_selected.get() {
                    format!(
                        "{} {} {}",
                        base,
                        quality_class,
                        match quality {
                            GraphicsQuality::High => {
                                "border-[#b28a4f] shadow-[0_6px_14px_rgba(0,0,0,0.3)] -translate-y-[1px]"
                            }
                            GraphicsQuality::Medium => {
                                "border-[#9d7b45] shadow-[0_5px_12px_rgba(0,0,0,0.26)] -translate-y-[1px]"
                            }
                            GraphicsQuality::Low => "border-[#8a6d40]",
                        },
                    )
                } else if was_last_bought.get() {
                    format!(
                        "{} {} {}",
                        base,
                        quality_class,
                        match quality {
                            GraphicsQuality::High => {
                                "border-fuchsia-700/70 hover:border-[#8c6a3b] hover:-translate-y-[1px] active:translate-y-[2px]"
                            }
                            GraphicsQuality::Medium => {
                                "border-fuchsia-700/70 hover:border-[#83653b] hover:-translate-y-[1px] active:translate-y-[2px]"
                            }
                            GraphicsQuality::Low => {
                                "border-fuchsia-700/70 hover:border-[#7b6039] active:translate-y-[1px]"
                            }
                        },
                    )
                } else {
                    format!(
                        "{} {} {}",
                        base,
                        quality_class,
                        match quality {
                            GraphicsQuality::High => {
                                "border-[#3b3428] hover:border-[#7b6440] hover:-translate-y-[1px] active:translate-y-[2px]"
                            }
                            GraphicsQuality::Medium => {
                                "border-[#4a3e2b] hover:border-[#715a38] hover:-translate-y-[1px] active:translate-y-[2px]"
                            }
                            GraphicsQuality::Low => {
                                "border-[#554631] hover:border-[#675236] active:translate-y-[1px]"
                            }
                        },
                    )
                }
            }
            on:click=move |_| {
                hide_tooltip();
                selected.set(Some(skill_id.clone()))
            }
            on:touchstart={
                let show_tooltip = show_tooltip.clone();
                move |_| { show_tooltip() }
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }
            on:mouseenter=move |_| show_tooltip()
            on:mouseleave=move |_| hide_tooltip()
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

            <div
                class=move || {
                    format!(
                        "relative flex h-20 w-20 xl:h-24 xl:w-24 items-center justify-center rounded-full
                        overflow-clip border {} {}",
                        skill_type_frame_border(skill_type),
                        match settings.graphics_quality() {
                            GraphicsQuality::High => {
                                "bg-[linear-gradient(180deg,rgba(214,177,102,0.1),rgba(0,0,0,0.2)),linear-gradient(180deg,rgba(43,40,46,0.96),rgba(20,19,23,1))] shadow-[0_4px_12px_rgba(0,0,0,0.58)]"
                            }
                            GraphicsQuality::Medium => {
                                "bg-[linear-gradient(180deg,rgba(214,177,102,0.08),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(41,38,44,0.96),rgba(21,20,24,1))] shadow-[0_3px_10px_rgba(0,0,0,0.48)]"
                            }
                            GraphicsQuality::Low => {
                                "bg-[linear-gradient(180deg,rgba(39,37,42,0.98),rgba(20,19,23,1))]"
                            }
                        },
                    )
                }
            >
                <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                    <div class="pointer-events-none absolute inset-[1px] rounded-full border border-[#d5b16d]/16"></div>
                </Show>
                <div
                    class=move || {
                        format!(
                            "pointer-events-none absolute inset-[3px] rounded-full border {} bg-[radial-gradient(circle_at_50%_40%,rgba(92,88,98,0.72),rgba(20,18,24,0.98)_72%)]",
                            match settings.graphics_quality() {
                                GraphicsQuality::High => "border-[#6d532e]/70",
                                GraphicsQuality::Medium => "border-[#6b5430]/55",
                                GraphicsQuality::Low => "border-[#5a4628]/55",
                            },
                        )
                    }
                ></div>
                <div class=format!(
                    "pointer-events-none absolute inset-[6px] rounded-full bg-radial {} to-transparent",
                    skill_type_inner_glow(skill_type),
                )></div>
                <img
                    draggable="false"
                    src=img_asset(&skill_specs.icon)
                    alt=skill_specs.name.clone()
                    class=move || {
                        format!(
                            "relative z-10 h-11 w-11 xl:h-14 xl:w-14 flex-no-shrink fill-current invert {}",
                            if settings.uses_surface_effects() {
                                "drop-shadow-[0_2px_2px_rgba(0,0,0,0.72)]"
                            } else {
                                ""
                            },
                        )
                    }
                />
            </div>

            <div class="text-center">
                <div class="text-sm xl:text-base font-bold text-white text-center font-display text-shadow-lg/100 shadow-gray-950 leading-tight">
                    {skill_specs.name.clone()}
                </div>
            </div>
        </div>
    }
}

fn skill_type_title(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "Attacks",
        SkillType::Spell => "Spells",
        SkillType::Curse => "Curses",
        SkillType::Blessing => "Blessings",
        SkillType::Other => "Others",
    }
}

fn skill_type_title_color(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "text-red-300",
        SkillType::Spell => "text-sky-300",
        SkillType::Curse => "text-purple-300",
        SkillType::Blessing => "text-amber-200",
        SkillType::Other => "text-slate-300",
    }
}

fn skill_type_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "via-red-400/70",
        SkillType::Spell => "via-sky-400/70",
        SkillType::Curse => "via-purple-400/70",
        SkillType::Blessing => "via-amber-300/75",
        SkillType::Other => "via-slate-300/60",
    }
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

fn skill_type_top_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "via-red-400/40",
        SkillType::Spell => "via-sky-400/40",
        SkillType::Curse => "via-purple-400/40",
        SkillType::Blessing => "via-amber-300/40",
        SkillType::Other => "via-slate-300/40",
    }
}
