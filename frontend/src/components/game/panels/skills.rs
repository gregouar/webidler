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
            card::{Card, CardHeader, CardInset},
            menu_panel::MenuPanel,
            tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
        },
    },
};

#[component]
pub fn SkillsPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open=open w_full=false h_full=false class:items-center>
            <Card class="max-w-6xl mx-auto">
                <CardHeader title="Buy New Skill" on_close=move || open.set(false) />
                // flex-1 overflow-auto max-h-[65vh]
                <SkillShop open=open />
            </Card>
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
                        view! {
                            <div class="grid grid-cols-6 xl:grid-cols-6 gap-2 xl:gap-3">
                    <For
                        each=move || available_skills.clone().into_iter()
                        key=|(skill_id,_)| skill_id.clone()
                        let:((skill_id, skill_specs))
                    >
                    {
                        move || {
                (!game_context
                        .player_specs
                        .read()
                        .bought_skills
                        .contains(&skill_id)).then(||
                        view!{
                        <SkillCard
                            skill_id=skill_id.clone()
                            skill_specs=skill_specs.clone()
                            selected=selected_skill
                        />})
                    }}
                    </For>
                </div>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </CardInset>

        <div class="flex items-center justify-center">
            <FancyButton disabled=disable_confirm on:click=buy_skill class:py-2 class:px-4>
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

    view! {
        <div
            class=move || {
                let base = format!(
                    "relative group  border rounded-md p-4 flex flex-col items-center
                    transition-all shadow cursor-pointer {}",
                    skill_type_color(skill_specs.base.skill_type),
                );
                if is_selected.get() {
                    format!("{} ring-4 bg-neutral-600", base)
                } else if was_last_bought.get() {
                    format!("{} hover:ring-2 ring-1 bg-mauve-800", base)
                } else {
                    format!("{} hover:ring-2 bg-neutral-800", base)
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
            <div class="w-full h-auto aspect-square">
                <img
                    draggable="false"
                    src=img_asset(&skill_specs.base.icon)
                    alt=skill_specs.base.name.clone()
                    class=move || {
                        format!(
                            "w-full h-full flex-no-shrink fill-current
                            drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert
                            transition-all ease-in-out
                            {}",
                            if is_selected.get() {
                                "scale-105 brightness-110"
                            } else {
                                "
                                group-hover:scale-105 group-hover:brightness-110
                                group-active:scale-90 group-active:brightness-90
                                "
                            },
                        )
                    }
                />
            </div>
            <div class="mt-2 text-lg font-bold text-white text-center">
                {skill_specs.base.name.clone()}
            </div>
        </div>
    }
}

fn skill_type_color(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "ring-red-600 border-red-400/50",
        SkillType::Spell => "ring-blue-600 border-blue-400/50",
        SkillType::Curse => "ring-purple-600 border-purple-400/50",
        SkillType::Blessing => "ring-yellow-600 border-yellow-400/50",
        SkillType::Other => "ring-slate-700 border-slate-400/50",
    }
}
