use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{
    data::skill::{BaseSkillSpecs, SkillSpecs, SkillType},
    messages::client::BuySkillMessage,
};

use crate::{
    assets::img_asset,
    components::{
        game::{game_context::GameContext, tooltips::SkillTooltip},
        ui::{
            buttons::{CloseButton, FancyButton},
            menu_panel::MenuPanel,
            number::format_number,
            tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
        },
        websocket::WebsocketContext,
    },
};

#[component]
pub fn SkillsPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open=open>
            <div class="w-full p-4">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-2">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                            "Buy New Skill "
                        </span>
                        <CloseButton on:click=move |_| open.set(false) />
                    </div>
                    <SkillShop open=open />
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
pub fn SkillShop(open: RwSignal<bool>) -> impl IntoView {
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

    let game_context = expect_context::<GameContext>();
    view! {
        <div class="flex flex-col gap-4 p-4">
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4
                            bg-neutral-900 p-4 rounded-md shadow-[inset_0_0_32px_rgba(0,0,0,0.6)] flex-1 overflow-auto max-h-[65vh]">
                    <For
                        each=move || get_all_available_skills().into_iter().enumerate()
                        key=|(i, _)| *i
                        let:((_, (skill_id,skill_specs)))
                    >
                        <SkillCard skill_id skill_specs selected=selected_skill />
                    </For>
                </div>

                <FancyButton
                    disabled=disable_confirm
                    on:click=buy_skill
                >
                    {move || format!("Confirm buying selected skill for {} Gold", format_number(game_context.player_specs.read().buy_skill_cost))}
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
    let is_selected = Signal::derive({
        let skill_id = skill_id.clone();
        move || selected.get().map(|s| s == skill_id).unwrap_or(false)
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let show_tooltip = {
        let skill_specs = Arc::new(skill_specs.clone());
        move |_| {
            let skill_specs = skill_specs.clone();
            tooltip_context.set_content(
                move || view! { <SkillTooltip skill_specs=skill_specs.clone() /> }.into_any(),
                DynamicTooltipPosition::Auto,
            );
        }
    };

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let hide_tooltip = move |_| tooltip_context.hide();

    view! {
        <div
            class=move || {
                format!(
                    "relative group bg-zinc-800 border rounded-md p-4 flex flex-col items-center
                transition-all shadow cursor-pointer hover:ring-2 hover:ring-amber-400 {}",
                    if is_selected.get() {
                        "border-amber-400 ring-2 ring-amber-500"
                    } else {
                        "border-zinc-700"
                    },
                )
            }
            on:click=move |_| selected.set(Some(skill_id.clone()))
            on:mouseenter=show_tooltip
            on:mouseleave=hide_tooltip
        >
            <img
                src=img_asset(&skill_specs.base.icon)
                alt=skill_specs.base.name.clone()
                class="w-full h-full flex-no-shrink fill-current
                drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert"
            />
            <div class="mt-2 text-lg font-bold text-white text-center">
                {skill_specs.base.name.clone()}
            </div>
            <div class="text-sm text-gray-400 text-center line-clamp-3">
                {skill_specs.base.description.clone()}
            </div>
        </div>
    }
}

// async fn get_all_available_skills() -> Vec<(String, SkillSpecs)> {
//     let skills_response = serde_json::from_str(
//         &reqwest::get("https://webidler.gregoirenaisse.be/game/skills")
//             .await?
//             .error_for_status()?
//             .text()
//             .await?,
//     )?;
// }

fn get_all_available_skills() -> Vec<(String, SkillSpecs)> {
    vec![
        (
            "fireball".to_string(),
            SkillSpecs {
                base: BaseSkillSpecs {
                    name: "Flame Burst".to_string(),
                    icon: "skills/fireball.svg".to_string(),
                    description: "Deals fire damage to all enemies in a cone.".to_string(),
                    skill_type: SkillType::Spell,
                    cooldown: 3.0,
                    mana_cost: 20.0,
                    upgrade_cost: 150.0,
                    upgrade_effects: vec![],
                    targets: vec![],
                },
                cooldown: 3.0,
                mana_cost: 20.0,
                upgrade_level: 0,
                next_upgrade_cost: 150.0,
                targets: vec![],
                item_slot: None,
            },
        ),
        // More...
    ]
}
