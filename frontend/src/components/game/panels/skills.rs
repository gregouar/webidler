use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{data::skill::SkillSpecs, messages::client::BuySkillMessage};

use crate::{
    assets::img_asset,
    components::{
        backend_client::BackendClient,
        game::game_context::GameContext,
        shared::tooltips::SkillTooltip,
        ui::{
            buttons::FancyButton,
            card::{Card, CardHeader, CardInset},
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
            <Card>
                <CardHeader title="Buy New Skill" on_close=move || open.set(false) />
                // flex-1 overflow-auto max-h-[65vh]
                <SkillShop open=open />
            </Card>
        </MenuPanel>
    }
}

#[component]
pub fn SkillShop(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

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

    let skills_response = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move { backend.get_skills().await.unwrap_or_default() }
    });

    // TODO: sort by types, add dropdown to filter by type, etc
    let available_skills = Signal::derive({
        move || {
            let mut skills = skills_response
                .get()
                .map(|skills_response| {
                    skills_response
                        .skills
                        .clone()
                        .into_iter()
                        .filter(|(skill_id, _)| {
                            !game_context
                                .player_specs
                                .read()
                                .bought_skills
                                .contains(skill_id)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            skills.sort_by_key(|(_, skill_specs)| skill_specs.base.name.clone());
            skills
        }
    });

    view! {
        <CardInset>
            <div class="grid grid-cols-6 xl:grid-cols-10 gap-2 xl:gap-4
            ">
                <Suspense fallback=move || {
                    view! { "Loading..." }
                }>
                    {move || {
                        view! {
                            <For
                            each=move || available_skills.get().into_iter()
                            key=|(skill_id, _)| skill_id.clone()
                            let:((skill_id,skill_specs))
                            >
                                <SkillCard skill_id skill_specs selected=selected_skill />
                            </For>
                        }
                    }}

                </Suspense>
            </div>
        </CardInset>

        <div class="flex items-center justify-center">
            <FancyButton disabled=disable_confirm on:click=buy_skill class:py-2 class:px-4>
                {move || {
                    format!(
                        "Confirm buying selected skill for {} Gold",
                        format_number(game_context.player_specs.read().buy_skill_cost),
                    )
                }}
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
                format!(
                    "relative group bg-neutral-800 border rounded-md p-4 flex flex-col items-center
                transition-all shadow cursor-pointer hover:ring-2 hover:ring-amber-400 {}",
                    if is_selected.get() {
                        "border-amber-400 ring-2 ring-amber-500"
                    } else if was_last_bought.get() {
                        "border-slate-400 ring-1 ring-slate-500"
                    } else {
                        "border-zinc-700"
                    },
                )
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
                    class="w-full h-full flex-no-shrink fill-current
                    drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert"
                />
            </div>
            <div class="mt-2 text-lg font-bold text-white text-center">
                {skill_specs.base.name.clone()}
            </div>
        // <div class="text-sm text-gray-400 text-center line-clamp-3">
        // {skill_specs.base.description.clone()}
        // </div>
        </div>
    }
}
