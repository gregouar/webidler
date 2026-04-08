use std::{collections::HashMap, sync::Arc};

use leptos::{html::*, prelude::*};

use shared::{
    data::skill::{SkillSpecs, SkillType},
    messages::client::BuySkillMessage,
};
use strum::IntoEnumIterator;

use crate::{
    assets::img_asset,
    components::{
        data_context::DataContext,
        game::{game_context::GameContext, websocket::WebsocketContext},
        shared::{resources::GoldCounter, tooltips::SkillTooltip},
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
                acc.entry(skill.1.base.skill_type).or_default().push(skill);
                acc
            },
        );

        for section in skills.values_mut() {
            section.sort_by_key(|(_, skill_specs)| skill_specs.base.name.clone());
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
                                    .player_specs
                                    .read_untracked()
                                    .bought_skills
                                    .contains(skill_id)
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
                        game_context.player_specs.read().buy_skill_cost
                    }) />
                </span>
            </FancyButton>
        </div>
    }
}
#[component]
fn SkillCard(
    skill_id: String,
    skill_specs: SkillSpecs,
    selected: RwSignal<Option<String>>,
) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

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
        let skill_specs = Arc::new(skill_specs.clone());
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
    let skill_type = skill_specs.base.skill_type;

    view! {
        <div
            class=move || {
                let base = format!(
                    "relative isolate overflow-hidden group border rounded-[9px]
                    px-3 py-3 xl:px-4 xl:py-4 flex flex-col items-center gap-3
                    bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
                    shadow-[0_5px_14px_rgba(0,0,0,0.28),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)]
                    transition-all duration-150 cursor-pointer",
                );
                if is_selected.get() {
                    format!(
                        "{} border-[#b28a4f] shadow-[0_8px_18px_rgba(0,0,0,0.34),inset_0_1px_0_rgba(244,225,181,0.08),inset_0_0_0_1px_rgba(214,177,102,0.18)] -translate-y-[1px]",
                        base,
                    )
                } else if was_last_bought.get() {
                    format!(
                        "{} border-fuchsia-700/70 hover:border-[#8c6a3b] hover:-translate-y-[1px] active:translate-y-[2px]",
                        base,
                    )
                } else {
                    format!(
                        "{} border-[#3b3428] hover:border-[#7b6440] hover:-translate-y-[1px] active:translate-y-[2px]",
                        base,
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
            <div class="pointer-events-none absolute inset-[1px] rounded-[8px] border border-white/5"></div>
            <div class=format!(
                "pointer-events-none absolute inset-x-4 top-[1px] h-px bg-gradient-to-r from-transparent {} to-transparent",
                skill_type_top_glow(skill_type),
            )></div>

            <div
                class=format!(
                    "relative flex h-20 w-20 xl:h-24 xl:w-24 items-center justify-center rounded-full
                    overflow-hidden
                    border {}
                    bg-stone-900
                    shadow-[0_0_15px_rgba(0,0,0,0.85),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45),inset_0_0_10px_rgba(0,0,0,0.75)]",
                    skill_type_frame_border(skill_type),
                )
                style="background-image:
                linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)),
                linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1));
                background-size: auto, auto;
                background-position: center, center;
                background-blend-mode: screen, normal;"
            >
                <div class="pointer-events-none absolute inset-[1px] rounded-full border border-[#d5b16d]/18"></div>
                <div class="pointer-events-none absolute inset-[3px] rounded-full border border-[#6d532e]/70 bg-[radial-gradient(circle_at_50%_40%,rgba(92,88,98,0.75),rgba(20,18,24,0.98)_72%)] shadow-[inset_0_2px_6px_rgba(0,0,0,0.55),inset_0_1px_0_rgba(236,210,148,0.14),0_1px_2px_rgba(0,0,0,0.35)]"></div>
                <div class=format!(
                    "pointer-events-none absolute inset-[6px] rounded-full bg-radial {} to-transparent",
                    skill_type_inner_glow(skill_type),
                )></div>
                <img
                    draggable="false"
                    src=img_asset(&skill_specs.base.icon)
                    alt=skill_specs.base.name.clone()
                    class=move || {
                        format!(
                            "relative z-10 h-11 w-11 xl:h-14 xl:w-14 flex-no-shrink fill-current
                            drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert
                            transition-all ease-in-out duration-150
                            {}",
                            if is_selected.get() {
                                "scale-105 brightness-110"
                            } else {
                                "group-hover:scale-105 group-hover:brightness-110"
                            },
                        )
                    }
                />
            </div>

            <div class="text-center">
                <div class="text-sm xl:text-base font-bold text-white text-center font-display text-shadow-lg/100 shadow-gray-950 leading-tight">
                    {skill_specs.base.name.clone()}
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
