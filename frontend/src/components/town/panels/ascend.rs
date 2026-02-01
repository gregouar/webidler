use leptos::{html::*, prelude::*, task::spawn_local};

use std::sync::Arc;

use shared::{
    data::{
        item::ItemCategory,
        passive::{
            PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassivesTreeAscension,
            PassivesTreeSpecs,
        },
    },
    http::client::AscendPassivesRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
    town::TownContext,
    ui::{
        buttons::MenuButton,
        card::{Card, CardHeader, CardInset},
        confirm::ConfirmContext,
        menu_panel::MenuPanel,
        pannable::Pannable,
        toast::*,
    },
};

#[component]
pub fn AscendPanel(
    open: RwSignal<bool>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let ascension_cost = RwSignal::new(0.0);
    let passives_tree_ascension = RwSignal::new(PassivesTreeAscension::default());

    let reset = move || {
        let mut initial_cost = 0.0;
        passives_tree_ascension.update(|passives_tree_ascension| {
            *passives_tree_ascension = town_context.passives_tree_ascension.get_untracked();
            passives_tree_ascension.ascended_nodes.retain(|node_id, v| {
                let keep = town_context
                    .passives_tree_specs
                    .read_untracked()
                    .nodes
                    .contains_key(node_id);
                if !keep {
                    initial_cost -= *v as f64;
                }
                keep
            });
        });

        ascension_cost.set(initial_cost.round());
    };

    let has_changed = Memo::new(move |_| {
        town_context
            .passives_tree_ascension
            .with(|base_ascension| !passives_tree_ascension.read().eq(base_ascension))
    });

    // Reset temporary ascension on opening
    Effect::new(move || {
        if open.get() {
            reset();
        }
    });

    view! {
        <MenuPanel open=open>
            <div class="w-full h-full">
                <Card>
                    <CardHeader title="Ascend Passive Skills" on_close=move || open.set(false)>
                        {(!view_only)
                            .then(|| {
                                view! {
                                    <div class="flex-1" />

                                    <div class="px-2 xl:px-4 relative z-10 flex items-center justify-between">
                                        <div class="flex items-center gap-2">
                                            <ResetButton passives_tree_ascension ascension_cost />
                                        </div>
                                    </div>

                                    <div class="flex-1" />

                                    <span class="text-sm xl:text-base text-gray-400">
                                        "Ascension Cost: "
                                        <span class="text-cyan-300">
                                            {ascension_cost}" Power Shards"
                                        </span>
                                    </span>

                                    <div class="flex-1" />

                                    <div class="flex items-center gap-2">
                                        <MenuButton
                                            on:click=move |_| reset()
                                            disabled=Signal::derive(move || !has_changed.get())
                                        >
                                            "Cancel"
                                        </MenuButton>
                                        <ConfirmButton
                                            passives_tree_ascension
                                            ascension_cost
                                            has_changed
                                            open
                                        />
                                    </div>
                                }
                            })}
                    </CardHeader>
                    <CardInset pad=false>
                        <PassiveSkillTree passives_tree_ascension ascension_cost view_only />
                    </CardInset>
                </Card>
            </div>
        </MenuPanel>
    }
}

#[component]
fn ConfirmButton(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
    has_changed: Memo<bool>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let do_ascend = Arc::new({
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();

        let character_id = town_context.character.read_untracked().character_id;
        move || {
            spawn_local({
                async move {
                    match backend
                        .post_ascend_passives(
                            &auth_context.token(),
                            &AscendPassivesRequest {
                                character_id,
                                passives_tree_ascension: passives_tree_ascension.get_untracked(),
                            },
                        )
                        .await
                    {
                        Ok(response) => {
                            town_context.character.set(response.character);
                            town_context.passives_tree_ascension.set(response.ascension);
                            open.set(false);
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("failed to ascend: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    });

    let try_ascend = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                format! {"Do you confirm Ascension for {} Power Shards?",ascension_cost.get() },
                do_ascend.clone(),
            );
        }
    };

    let disabled = Signal::derive(move || !has_changed.get());

    view! {
        <MenuButton on:click=try_ascend disabled=disabled>
            "Confirm Ascension"
        </MenuButton>
    }
}

#[component]
fn ResetButton(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
) -> impl IntoView {
    let do_reset = Arc::new({
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            spawn_local({
                async move {
                    match backend
                        .post_ascend_passives(
                            &auth_context.token(),
                            &AscendPassivesRequest {
                                character_id,
                                passives_tree_ascension: PassivesTreeAscension::default(),
                            },
                        )
                        .await
                    {
                        Ok(response) => {
                            passives_tree_ascension.set(response.ascension.clone());
                            ascension_cost.set(0.0);
                            town_context.character.set(response.character);
                            town_context.passives_tree_ascension.set(response.ascension);
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("failed to refund: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    });

    let try_reset = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                "Do you confirm fully Refund Ascension and reclaim all Power Shards?".to_string(),
                do_reset.clone(),
            );
        }
    };

    view! { <MenuButton on:click=try_reset>"Refund Ascension"</MenuButton> }
}

#[component]
fn PassiveSkillTree(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
    view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let points_available = Memo::new(move |_| {
        if view_only {
            0.0
        } else {
            (town_context.character.read().resource_shards - ascension_cost.get()).round()
        }
    });

    let selected_socket_node = RwSignal::new(None);

    Effect::new(move || {
        if let Some(passive_node_id) = selected_socket_node.get_untracked() {
            if let Some(item_index) = town_context.selected_item_index.get() {
                if let Some(item_specs) = town_context.inventory.read().bag.get(item_index as usize)
                {
                    selected_socket_node.set(None);
                    town_context.selected_item_index.set(None);

                    // TODO: Use backend directly? => but then socket might be not unlocked yet =/

                    passives_tree_ascension
                        .write()
                        .socketed_nodes
                        .insert(passive_node_id, item_specs.modifiers.clone());
                }
            }
        }
    });

    view! {
        <Pannable>
            <For
                each=move || {
                    town_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <AscendConnection
                    connection=conn
                    passives_tree_specs=town_context.passives_tree_specs
                    passives_tree_ascension
                />
            </For>
            <For
                each=move || { town_context.passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| id.clone()
                let((id, node))
            >
                <AscendNode
                    node_id=id
                    node_specs=node
                    points_available
                    ascension_cost
                    passives_tree_ascension
                    selected_socket_node
                    view_only
                />
            </For>
        </Pannable>
    }
}

#[component]
fn AscendNode(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    points_available: Memo<f64>,
    ascension_cost: RwSignal<f64>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    selected_socket_node: RwSignal<Option<PassiveNodeId>>,
    view_only: bool,
) -> impl IntoView {
    let town_context: TownContext = expect_context();

    let node_level = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&node_id)
                .copied()
                .unwrap_or_default()
        }
    });

    let max_node_level = if node_specs.upgrade_effects.is_empty() {
        node_specs.locked as u8
    } else {
        node_specs.max_upgrade_level.unwrap_or(u8::MAX)
    };

    let max_upgrade_level = Memo::new({
        let node_id = node_id.clone();
        move |_| {
            let max_connection_level = if node_specs.initial_node {
                u8::MAX
            } else {
                town_context
                    .passives_tree_specs
                    .read()
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
                    .map(|node_id| {
                        passives_tree_ascension
                            .read()
                            .ascended_nodes
                            .get(&node_id)
                            .copied()
                            .unwrap_or_default()
                    })
                    .max()
                    .unwrap_or_default()
            };

            max_node_level.min(max_connection_level)
        }
    });

    let node_status = Memo::new({
        move |_| {
            let upgradable = max_upgrade_level.get() > node_level.get();
            // let maxed = node_level.get() >= max_upgrade_level && node_level.get() > 0;

            let purchase_status =
            //  if maxed {
            //     PurchaseStatus::Inactive
            // } else 
            if (view_only|| (node_level.get() == max_node_level && !node_specs.socket )) && node_level.get() > 0  {
                PurchaseStatus::Purchased
            } else if (points_available.get() > 0.0 && upgradable) || (node_specs.socket && (! node_specs.locked || node_level.get() > 0 ))
                // && (upgradable || (node_specs.locked && node_level.get() == 0))
            {
                PurchaseStatus::Purchaseable
            } else {
                PurchaseStatus::Inactive
            };

            let meta_status = if node_level.get() > 0 {
                // if node_specs.locked && node_level.get() == 1 {
                //     MetaStatus::Normal
                // } else {
                //     MetaStatus::Ascended
                // }
                MetaStatus::Ascended
            } else if node_specs.locked {
                MetaStatus::Locked
            } else {
                MetaStatus::Normal
            };

            NodeStatus {
                purchase_status,
                meta_status,
            }
        }
    });

    let purchase = {
        let node_id = node_id.clone();
        move || {
            if node_specs.socket && (!node_specs.locked || node_level.get() > 0) {
                selected_socket_node.set(Some(node_id.clone()));
                town_context.selected_item_index.set(None);
                town_context
                    .use_item_category_filter
                    .set(Some(ItemCategory::Rune));
                town_context.open_inventory.set(true);
            } else {
                passives_tree_ascension.update(|passives_tree_ascension| {
                    let entry = passives_tree_ascension
                        .ascended_nodes
                        .entry(node_id.clone())
                        .or_default();
                    *entry = entry.saturating_add(1);
                });
                ascension_cost.update(|ascension_cost| *ascension_cost += 1.0); // TODO: Ascend cost?
            }
        }
    };

    let refund = {
        let node_id = node_id.clone();
        move || {
            passives_tree_ascension.update(|passives_tree_ascension| {
                let entry = passives_tree_ascension
                    .ascended_nodes
                    .entry(node_id.clone())
                    .or_default();
                if *entry > 0 {
                    *entry = entry.saturating_sub(1);
                    ascension_cost.update(|ascension_cost| *ascension_cost -= 1.0);
                }
            });
        }
    };

    view! {
        <Node
            node_specs
            node_status
            node_level
            on_click=purchase
            on_right_click=refund
            show_upgrade=true
        />
    }
}

#[component]
fn AscendConnection(
    connection: PassiveConnection,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
) -> impl IntoView {
    let amount_connections = Memo::new({
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&connection_from)
                .map(|x| (*x > 0) as usize)
                .unwrap_or_default()
                + passives_tree_ascension
                    .read()
                    .ascended_nodes
                    .get(&connection_to)
                    .map(|x| (*x > 0) as usize)
                    .unwrap_or_default()
        }
    });

    let node_levels = Memo::new({
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            (
                passives_tree_ascension
                    .read()
                    .ascended_nodes
                    .get(&connection_from)
                    .cloned()
                    .unwrap_or_default(),
                passives_tree_ascension
                    .read()
                    .ascended_nodes
                    .get(&connection_to)
                    .cloned()
                    .unwrap_or_default(),
            )
        }
    });

    view! { <Connection connection passives_tree_specs amount_connections node_levels /> }
}
