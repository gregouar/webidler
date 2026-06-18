use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use leptos::{html::*, prelude::*};

use shared::{
    data::{
        player::PlayerBaseSkill,
        skill::{BaseSkillSpecs, SkillType},
    },
    messages::client::BuySkillMessage,
};
use strum::IntoEnumIterator;

use crate::components::{
    data_context::DataContext,
    game::{game_context::GameContext, websocket::WebsocketContext},
    settings::{GraphicsQuality, SettingsContext},
    shared::{
        resources::GoldCounter,
        skills::{SkillBadge, skill_specs_from_base},
        tooltips::SkillTooltip,
    },
    ui::{
        buttons::FancyButton,
        card::{CardHeader, CardInset, MenuCard},
        menu_panel::MenuPanel,
        tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
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
    let selected_skill_name = Signal::derive(move || {
        selected_skill.get().and_then(|skill_id| {
            data_context
                .skill_specs
                .with(|skill_specs| skill_specs.get(&skill_id).map(|skill| skill.name.clone()))
        })
    });

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
        let mut skills = data_context
            .skill_specs
            .get()
            .into_iter()
            .filter(|(_, base_skill_specs)| !base_skill_specs.hidden)
            .fold(HashMap::<_, Vec<_>>::new(), |mut acc, skill| {
                acc.entry(skill.1.skill_type).or_default().push(skill);
                acc
            });

        for section in skills.values_mut() {
            section.sort_by_key(|(_, skill_specs)| skill_specs.name.clone());
        }

        skills
    });
    let skill_sections = Memo::new(move |_| {
        let bought_skills = game_context.player_base_specs.with(|player_base_specs| {
            player_base_specs
                .skills
                .keys()
                .cloned()
                .collect::<HashSet<_>>()
        });

        available_skills.with(|available_skills| {
            SkillType::iter()
                .filter_map(|skill_type| {
                    let unbought_skills = available_skills
                        .get(&skill_type)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|(skill_id, _)| !bought_skills.contains(skill_id))
                        .collect::<Vec<_>>();

                    (!unbought_skills.is_empty()).then_some((skill_type, unbought_skills))
                })
                .collect::<Vec<_>>()
        })
    });
    let next_favorite_skill = Memo::new(move |_| {
        game_context.player_base_specs.with(|player_base_specs| {
            player_base_specs
                .skill_masteries
                .favorite_skills
                .iter()
                .find(|skill_id| !player_base_specs.skills.contains_key(*skill_id))
                .cloned()
        })
    });

    Effect::new(move || {
        if open.get() {
            selected_skill.set(next_favorite_skill.get());
        }
    });

    view! {
        <CardInset>
            {move || {
                skill_sections
                    .get()
                    .into_iter()
                    .map(move |(skill_type, unbought_skills)| {
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

                                <div class="grid grid-cols-2 md:grid-cols-4 gap-2 xl:gap-3">
                                    <For
                                        each=move || unbought_skills.clone().into_iter()
                                        key=|(skill_id, _)| skill_id.clone()
                                        let:((skill_id, skill_specs))
                                    >
                                        <SkillCard
                                            skill_id=skill_id.clone()
                                            base_skill_specs=skill_specs.clone()
                                            selected=selected_skill
                                        />
                                    </For>
                                </div>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </CardInset>

        <div class="flex items-center justify-center">
            <FancyButton disabled=disable_confirm on:click=buy_skill class="py-2 px-4">
                <span class="flex items-center gap-2 text-zinc-300">
                    "Confirm buying "
                    {move || {
                        selected_skill_name
                            .get()
                            .map(|skill_name| {
                                view! { <span class="font-display text-white">{skill_name}</span> }
                            })
                    }} " for"
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
    base_skill_specs: BaseSkillSpecs,
    selected: RwSignal<Option<String>>,
) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let settings = expect_context::<SettingsContext>();

    let skill_type = base_skill_specs.skill_type;
    let skill_name = base_skill_specs.name.clone();
    let skill_icon = base_skill_specs.icon.clone();

    let is_selected = Signal::derive({
        let skill_id = skill_id.clone();
        move || selected.get().map(|s| s == skill_id).unwrap_or(false)
    });

    let is_favorite = Memo::new({
        let skill_id = skill_id.clone();
        move |_| {
            game_context
                .player_base_specs
                .read()
                .skill_masteries
                .favorite_skills
                .contains(&skill_id)
        }
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let tooltip_id = RwSignal::new(0);
    let show_tooltip = {
        let skill_specs = Arc::new(skill_specs_from_base(skill_id.clone(), &base_skill_specs));
        let player_base_skill = Some(Arc::new(PlayerBaseSkill {
            next_upgrade_cost: base_skill_specs.upgrade_cost,
            base_skill_specs,
            item_slot: None,
            upgrade_level: 0,
        }));
        move || {
            let skill_specs = skill_specs.clone();
            let player_base_skill = player_base_skill.clone();
            tooltip_id.set(tooltip_context.set_content(
                move || {
                    view! {
                        <SkillTooltip
                            skill_specs=skill_specs.clone()
                            player_base_skill=player_base_skill.clone()
                        />
                    }
                    .into_any()
                },
                DynamicTooltipPosition::Auto,
            ));
        }
    };

    let hide_tooltip = move || {
        tooltip_context.hide(tooltip_id.get_untracked());
    };
    on_cleanup(hide_tooltip);

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
                let selected_quality_class = match quality {
                    GraphicsQuality::High => {
                        "bg-[linear-gradient(180deg,rgba(214,177,102,0.24),rgba(103,74,34,0.18)_42%,rgba(30,24,17,0.52)_100%),linear-gradient(135deg,rgba(62,50,34,0.98),rgba(26,22,18,1))]
                        shadow-[0_6px_14px_rgba(0,0,0,0.3),inset_0_0_22px_rgba(214,177,102,0.14)]"
                    }
                    GraphicsQuality::Medium => {
                        "bg-[linear-gradient(180deg,rgba(200,164,96,0.2),rgba(96,69,34,0.15)_44%,rgba(29,24,18,0.46)_100%),linear-gradient(135deg,rgba(57,47,34,0.98),rgba(25,22,18,1))]
                        shadow-[0_5px_12px_rgba(0,0,0,0.26),inset_0_0_18px_rgba(200,164,96,0.11)]"
                    }
                    GraphicsQuality::Low => {
                        "bg-[linear-gradient(180deg,rgba(177,143,85,0.18),rgba(92,67,36,0.14)_44%,rgba(28,24,19,0.44)_100%),linear-gradient(135deg,rgba(52,44,34,0.98),rgba(25,22,19,1))]"
                    }
                };
                if is_selected.get() {
                    format!(
                        "{} {} {}",
                        base,
                        selected_quality_class,
                        match quality {
                            GraphicsQuality::High => "border-[#b28a4f] -translate-y-[1px]",
                            GraphicsQuality::Medium => "border-[#9d7b45] -translate-y-[1px]",
                            GraphicsQuality::Low => "border-[#8a6d40]",
                        },
                    )
                } else if is_favorite.get() {
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
                move |_| show_tooltip()
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

            <SkillBadge
                skill_type
                icon=Some(skill_icon.clone())
                alt=skill_name.clone()
                selected=is_selected
            />

            <div class="text-center">
                <div class="text-sm xl:text-base font-bold text-white text-center font-display text-shadow-lg/100 shadow-gray-950 leading-tight">
                    {skill_name}
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

fn skill_type_top_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "via-red-400/40",
        SkillType::Spell => "via-sky-400/40",
        SkillType::Curse => "via-purple-400/40",
        SkillType::Blessing => "via-amber-300/40",
        SkillType::Other => "via-slate-300/40",
    }
}
