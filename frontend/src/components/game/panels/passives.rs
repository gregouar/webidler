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
    events::{EventsContext, Key},
    game::{game_context::GameContext, websocket::WebsocketContext},
    shared::passives::{
        Connection, MetaStatus, Node, NodeStatus, PassiveSkillStats, PurchaseStatus,
        node_meta_status,
    },
    ui::{
        buttons::MenuButton,
        card::{CardHeader, CardInset, MenuCard},
        confirm::ConfirmContext,
        input::Input,
        menu_panel::MenuPanel,
        pannable::Pannable,
        toast::*,
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};

#[component]
pub fn PassivesPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let search_node = RwSignal::new(None);
    let search_node_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new({
        let events_context: EventsContext = expect_context();
        move || {
            if events_context.key_pressed(Key::Ctrl)
                && events_context.key_pressed(Key::Character('f'))
                && let Some(input) = search_node_ref.get()
            {
                input.focus().unwrap();
                input.select();
            }
        }
    });

    view! {
        <MenuPanel open=open>
            <div class="w-full h-full">
                <MenuCard>
                    <CardHeader title="Passive Skills" on_close=move || open.set(false)>
                        <div class="flex px-2 xl:px-4">
                            <Input
                                node_ref=search_node_ref
                                id="search_node"
                                input_type="text"
                                placeholder="Search for node..."
                                bind=search_node
                            />
                        </div>

                        <div class="flex-1" />

                        <span class="text-sm xl:text-base text-gray-400">
                            "Remaining Points: "
                            <span class="font-semibold text-white">
                                {move || { game_context.player_resources.read().passive_points }}
                            </span>
                        </span>

                        <div class="flex-1" />

                        <div class="flex items-center gap-2 mx-2">
                            <ExportButton />
                        </div>

                        <div class="flex-1" />

                        <div class="flex items-center gap-2 mx-2">
                            <AutoButton />
                        </div>

                    </CardHeader>
                    <CardInset pad=false class="relative">
                        <PassiveSkillTree search_node />
                    </CardInset>
                </MenuCard>
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
            <div class="flex flex-col xl:space-y-1 text-sm max-w-xs">
                <span class="text-white">"Assign points following previously saved build."</span>
                <span class="text-xs italic text-gray-400">
                    "Hold CTRL: +"{10.min(game_context.player_resources.read().passive_points)}
                </span>
            </div>
        }
    };

    let auto_assign = {
        let conn: WebsocketContext = expect_context();
        let events_context: EventsContext = expect_context();
        move |_| {
            let mut amount = if events_context.key_pressed(Key::Ctrl) {
                10.min(game_context.player_resources.read().passive_points)
            } else {
                1
            };

            while let Some(node_id) = next_node.get_untracked()
                && amount > 0
            {
                purchase_node(game_context, conn.clone(), node_id);
                amount -= 1;
            }
        }
    };

    view! {
        <StaticTooltip tooltip position=StaticTooltipPosition::Bottom>
            <MenuButton on:click=auto_assign disabled>
                "Auto Assign"
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
fn PassiveSkillTree(search_node: RwSignal<Option<String>>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let points_available =
        Memo::new(move |_| game_context.player_resources.read().passive_points > 0);

    view! {
        <PassiveSkillStats
            passives_tree_specs=game_context.passives_tree_specs
            passives_tree_ascension=Signal::derive(move || {
                game_context.passives_tree_state.read().ascension.clone()
            })
            purchased_nodes=Signal::derive(move || {
                game_context.passives_tree_state.read().purchased_nodes.clone()
            })
        />
        <Pannable>
            <For
                each=move || {
                    game_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from, conn.to)
                let(conn)
            >
                <InGameConnection connection=conn />
            </For>
            <For
                each=move || { game_context.passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| *id
                let((id, node))
            >
                <InGameNode
                    node_id=id
                    node_specs=node
                    points_available=points_available
                    search_node
                />
            </For>
        </Pannable>
    }
}

#[component]
fn InGameNode(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    points_available: Memo<bool>,
    search_node: RwSignal<Option<String>>,
) -> impl IntoView {
    let game_context: GameContext = expect_context();
    let node_level = Memo::new(move |_| {
        game_context
            .passives_tree_state
            .read()
            .ascension
            .ascended_nodes
            .get(&node_id)
            .copied()
            .unwrap_or_default()
    });

    let connected_nodes: Vec<_> = game_context
        .passives_tree_specs
        .read_untracked()
        .connections
        .iter()
        .filter_map(|connection| {
            if connection.from == node_id {
                Some(connection.to)
            } else if connection.to == node_id {
                Some(connection.from)
            } else {
                None
            }
        })
        .collect();

    let node_status = Memo::new(move |_| {
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
            && (node_specs.root_node
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
    });

    let purchase = {
        let conn = expect_context::<WebsocketContext>();
        move || purchase_node(game_context, conn.clone(), node_id)
    };

    view! {
        <Node
            node_specs
            node_status
            node_level
            on_click=purchase
            on_right_click=|| {}
            show_upgrade=false
            search_node
        />
    }
}

#[component]
fn InGameConnection(connection: PassiveConnection) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let amount_connections = Memo::new(move |_| {
        game_context
            .passives_tree_state
            .read()
            .purchased_nodes
            .contains(&connection.from) as usize
            + game_context
                .passives_tree_state
                .read()
                .purchased_nodes
                .contains(&connection.to) as usize
    });

    let node_levels = Memo::new(move |_| {
        (
            game_context
                .passives_tree_state
                .read()
                .ascension
                .ascended_nodes
                .get(&connection.from)
                .cloned()
                .unwrap_or_default(),
            game_context
                .passives_tree_state
                .read()
                .ascension
                .ascended_nodes
                .get(&connection.to)
                .cloned()
                .unwrap_or_default(),
        )
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

fn purchase_node(game_context: GameContext, conn: WebsocketContext, node_id: PassiveNodeId) {
    game_context.player_resources.write().passive_points -= 1;
    game_context
        .passives_tree_state
        .write()
        .purchased_nodes
        .insert(node_id);
    conn.send(&PurchasePassiveMessage { node_id }.into());
}
