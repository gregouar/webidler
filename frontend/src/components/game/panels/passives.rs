use std::sync::Arc;

use leptos::{html::*, prelude::*, task::spawn_local};

use shared::{
    data::passive::{PassiveConnection, PassiveNodeId, PassiveNodeSpecs},
    http::client::SavePassivesRequest,
    messages::client::PurchasePassiveMessage,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    game::game_context::GameContext,
    shared::passives::{
        node_meta_status, Connection, MetaStatus, Node, NodeStatus, PurchaseStatus,
    },
    ui::{
        buttons::MenuButton,
        card::{Card, CardHeader, CardInset},
        confirm::ConfirmContext,
        menu_panel::MenuPanel,
        pannable::Pannable,
        toast::*,
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
    websocket::WebsocketContext,
};

#[component]
pub fn PassivesPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    view! {
        <MenuPanel open=open>
            <div class="w-full h-full">
                <Card>
                    <CardHeader title="Passive Skills" on_close=move || open.set(false)>

                        <div class="flex-1" />

                        <div class="flex items-center gap-2 mx-2">
                            <ExportButton />
                        </div>

                        <div class="flex-1" />

                        <span class="text-sm xl:text-base text-gray-400">
                            "Remaining Points: "
                            <span class="bold">
                                {move || { game_context.player_resources.read().passive_points }}
                            </span>
                        </span>

                        <div class="flex-1" />

                        <div class="flex items-center gap-2 mx-2">
                            <AutoButton />
                        </div>

                        <div class="flex-1" />

                    </CardHeader>
                    <CardInset pad=false>
                        <PassiveSkillTree />
                    </CardInset>
                </Card>
            </div>
        </MenuPanel>
    }
}

#[component]
fn AutoButton() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let points_available =
        Memo::new(move |_| game_context.player_resources.read().passive_points > 0);

    let next_node = Memo::new(move |_| {
        game_context
            .passives_tree_state
            .with(|passives_tree_state| {
                game_context
                    .passives_tree_build
                    .read()
                    .iter()
                    .find(|node_id| !passives_tree_state.purchased_nodes.contains(*node_id))
                    .cloned()
            })
    });

    let disabled = Signal::derive(move || !points_available.get() || next_node.read().is_none());

    let tooltip = move || {
        view! {
            <div class="flex flex-col space-y-1 text-sm max-w-xs">
                <span class="font-semibold text-white">
                    "Assign points following previously saved build."
                </span>
                <span class="text-xs italic text-gray-400">"Hold CTRL: +10"</span>
            </div>
        }
    };

    view! {
        <StaticTooltip tooltip position=StaticTooltipPosition::Left>
            <MenuButton on:click=move |_| {} disabled>
                "Auto"
            </MenuButton>
        </StaticTooltip>
    }
}

#[component]
fn ExportButton() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let do_export = Arc::new({
        let backend = expect_context::<BackendClient>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();

        let character_id = game_context.character_id.get_untracked();
        move || {
            spawn_local({
                async move {
                    match backend
                        .post_save_passives(
                            &auth_context.token(),
                            &SavePassivesRequest {
                                character_id,
                                purchased_nodes: game_context
                                    .passives_tree_state
                                    .read()
                                    .purchased_nodes
                                    .clone(),
                            },
                        )
                        .await
                    {
                        Ok(_) => show_toast(toaster, "Export Succeeded!", ToastVariant::Success),
                        Err(e) => show_toast(
                            toaster,
                            format!("Failed to export: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    });

    let try_export = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                "Exporting your build will erase the last version saved, are you sure?".into(),
                do_export.clone(),
            );
        }
    };

    let disabled = Signal::derive(move || {
        game_context
            .passives_tree_state
            .read()
            .purchased_nodes
            .is_empty()
    });

    view! {
        <MenuButton on:click=try_export disabled>
            "Export Build"
        </MenuButton>
    }
}

#[component]
fn PassiveSkillTree() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let points_available =
        Memo::new(move |_| game_context.player_resources.read().passive_points > 0);

    view! {
        <Pannable>
            <For
                each=move || {
                    game_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <InGameConnection connection=conn />
            </For>
            <For
                each=move || { game_context.passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| id.clone()
                let((id, node))
            >
                <InGameNode node_id=id node_specs=node points_available=points_available />
            </For>
        </Pannable>
    }
}

#[component]
fn InGameNode(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    points_available: Memo<bool>,
) -> impl IntoView {
    let game_context: GameContext = expect_context();
    let node_level = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            game_context
                .passives_tree_state
                .read()
                .ascension
                .ascended_nodes
                .get(&node_id)
                .copied()
                .unwrap_or_default()
        }
    });

    let connected_nodes: Vec<_> = game_context
        .passives_tree_specs
        .read_untracked()
        .connections
        .iter()
        .filter_map(|connection| {
            if connection.from == node_id {
                Some(connection.to.clone())
            } else if connection.to == node_id {
                Some(connection.from.clone())
            } else {
                None
            }
        })
        .collect();

    let node_status = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            let meta_status = node_meta_status(
                node_level.get(),
                node_specs.locked,
                // node_specs.max_upgrade_level,
            );

            let purchase_status = if game_context
                .passives_tree_state
                .read()
                .purchased_nodes
                .contains(&node_id)
            {
                PurchaseStatus::Purchased
            } else if meta_status != MetaStatus::Locked
                && points_available.get()
                && (node_specs.initial_node
                    || game_context
                        .passives_tree_state
                        .with(|passives_tree_state| {
                            connected_nodes.iter().any(|connected_node| {
                                passives_tree_state.purchased_nodes.contains(connected_node)
                            })
                        }))
            {
                PurchaseStatus::Purchaseable
            } else {
                PurchaseStatus::Inactive
            };

            NodeStatus {
                purchase_status,
                meta_status,
            }
        }
    });

    let purchase = {
        let conn = expect_context::<WebsocketContext>();

        move || {
            game_context.player_resources.write().passive_points -= 1;
            game_context
                .passives_tree_state
                .write()
                .purchased_nodes
                .insert(node_id.clone());
            conn.send(
                &PurchasePassiveMessage {
                    node_id: node_id.clone(),
                }
                .into(),
            );
        }
    };

    view! {
        <Node
            node_specs
            node_status
            node_level
            on_click=purchase
            on_right_click=|| {}
            show_upgrade=false
        />
    }
}

#[component]
fn InGameConnection(connection: PassiveConnection) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let amount_connections = Memo::new({
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            game_context
                .passives_tree_state
                .read()
                .purchased_nodes
                .contains(&connection_from) as usize
                + game_context
                    .passives_tree_state
                    .read()
                    .purchased_nodes
                    .contains(&connection_to) as usize
        }
    });

    let node_levels = Memo::new({
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            (
                game_context
                    .passives_tree_state
                    .read()
                    .ascension
                    .ascended_nodes
                    .get(&connection_from)
                    .cloned()
                    .unwrap_or_default(),
                game_context
                    .passives_tree_state
                    .read()
                    .ascension
                    .ascended_nodes
                    .get(&connection_to)
                    .cloned()
                    .unwrap_or_default(),
            )
        }
    });

    view! {
        <Connection
            connection
            passives_tree_specs=game_context.passives_tree_specs
            amount_connections
            node_levels
        />
    }
}
