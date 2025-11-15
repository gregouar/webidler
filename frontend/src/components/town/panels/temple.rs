use leptos::{html::*, prelude::*, task::spawn_local};

use std::sync::Arc;

use shared::{
    data::passive::{PassiveNodeId, PassiveNodeSpecs, PassivesTreeAscension},
    http::client::AscendPassivesRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    game::panels::passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
    town::TownContext,
    ui::{
        buttons::{CloseButton, MenuButton},
        confirm::ConfirmContext,
        menu_panel::{MenuPanel, PanelTitle},
        pannable::Pannable,
        toast::*,
    },
};

#[component]
pub fn TemplePanel(
    open: RwSignal<bool>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    view! {
        <MenuPanel open=open>
            <div class="w-full h-full">
                <div class="bg-zinc-800 rounded-md p-1 xl:p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-1 xl:gap-2 max-h-full">
                    <div class="px-2 xl:px-4 flex items-center justify-between">
                        <PanelTitle>"Temple"</PanelTitle>
                        <CloseButton on:click=move |_| open.set(false) />
                    </div>
                </div>
            </div>
        </MenuPanel>
    }
}
