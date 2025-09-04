use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{data::skill::SkillSpecs, messages::client::BuySkillMessage};

use crate::{
    assets::img_asset,
    components::{
        backend_client::BackendClient,
        game::{game_context::GameContext, tooltips::SkillTooltip},
        ui::{
            buttons::{CloseButton, FancyButton},
            menu_panel::{MenuPanel, PanelTitle},
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
            <div class="w-full">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-2">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <PanelTitle>"Buy New Skill "</PanelTitle>
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

    let skills_response = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move { backend.get_skills().await.unwrap_or_default() }
    });

    // TODO: sort by types, add dropdown to filter by type, etc
    let available_skills = Signal::derive({
        let game_context = expect_context::<GameContext>();
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

    let game_context = expect_context::<GameContext>();
    view! {
        <div class="flex flex-col gap-2 lg:gap-4 p-2 lg:p-4">
            <div class="grid grid-cols-6 lg:grid-cols-10 gap-2 lg:gap-4
            bg-neutral-900 p-2 lg:p-4 rounded-md shadow-[inset_0_0_32px_rgba(0,0,0,0.6)] flex-1 overflow-auto max-h-[65vh]">

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
        // <div class="text-sm text-gray-400 text-center line-clamp-3">
        // {skill_specs.base.description.clone()}
        // </div>
        </div>
    }
}
