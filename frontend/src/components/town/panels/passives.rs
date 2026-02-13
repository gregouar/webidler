use leptos::{html::*, prelude::*, task::spawn_local};

use std::sync::Arc;

use shared::{
    data::{
        item::ItemCategory,
        item_affix::AffixEffectScope,
        passive::{
            PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassivesTreeAscension,
            PassivesTreeSpecs, PurchasedNodes,
        },
    },
    http::client::{AscendPassivesRequest, SavePassivesRequest, SocketPassiveRequest},
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    events::{EventsContext, Key},
    shared::{
        passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
        resources::ShardsCounter,
    },
    town::TownContext,
    ui::{
        buttons::{MenuButton, TabButton},
        card::{Card, CardHeader, CardInset},
        confirm::ConfirmContext,
        input::Input,
        menu_panel::MenuPanel,
        pannable::Pannable,
        toast::*,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PassivesTab {
    Ascend,
    Build,
}

#[component]
pub fn PassivesPanel(
    open: RwSignal<bool>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let ascension_cost = RwSignal::new(0.0);
    let passives_tree_ascension = RwSignal::new(PassivesTreeAscension::default());
    let passives_tree_build = RwSignal::new(PurchasedNodes::default());

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

        passives_tree_build.set(town_context.passives_tree_build.get());

        ascension_cost.set(initial_cost.round());
    };

    // Reset temporary ascension on opening
    Effect::new(move || {
        if open.get() {
            reset();
        }
    });

    let active_tab = RwSignal::new(PassivesTab::Ascend);

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
                <Card>
                    <CardHeader title="Passive Skills" on_close=move || open.set(false)>
                        <div class="flex-1 flex self-end justify-center h-full ml-2 xl:ml-4 gap-2 xl:gap-4 -mb-2 w-xl overflow-hidden">
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == PassivesTab::Ascend
                                })
                                on:click=move |_| { active_tab.set(PassivesTab::Ascend) }
                            >
                                "Ascend"
                            </TabButton>
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == PassivesTab::Build
                                })
                                on:click=move |_| { active_tab.set(PassivesTab::Build) }
                            >
                                "Plan"
                            </TabButton>
                        </div>

                        <div class="flex-1"></div>

                        <div class="flex gap-2 ">
                            <Input
                                node_ref=search_node_ref
                                id="search_node"
                                input_type="text"
                                placeholder="Search for node..."
                                bind=search_node
                            />
                        </div>

                        <div class="flex-1"></div>

                        {move || match active_tab.get() {
                            PassivesTab::Ascend => {
                                view! {
                                    <AscendPanelHeader
                                        passives_tree_ascension
                                        ascension_cost
                                        view_only
                                    />
                                }
                                    .into_any()
                            }
                            PassivesTab::Build => {
                                view! { <BuildPanelHeader passives_tree_build view_only /> }
                                    .into_any()
                            }
                        }}

                    </CardHeader>
                    <CardInset pad=false>
                        <PassiveSkillTree
                            active_tab
                            passives_tree_ascension
                            passives_tree_build
                            search_node
                            ascension_cost
                            view_only
                        />
                    </CardInset>
                </Card>
            </div>
        </MenuPanel>
    }
}

// Ascend Header
// -------------

#[component]
pub fn AscendPanelHeader(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

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

    view! {
        {(!view_only)
            .then(|| {
                view! {
                    <div class="text-sm xl:text-base text-gray-400 flex items-center">
                        "Ascension Cost:" <ShardsCounter value=ascension_cost.into() />
                    </div>

                    <div class="flex-1" />

                    <div class="px-2 xl:px-4 relative z-10 flex items-center gap-2">
                        <RefundAscendButton passives_tree_ascension ascension_cost />
                    </div>

                    <div class="flex-1" />

                    <div class="flex items-center gap-2">
                        <MenuButton
                            on:click=move |_| reset()
                            disabled=Signal::derive(move || !has_changed.get())
                        >
                            "Cancel"
                        </MenuButton>
                        <ConfirmAscendButton passives_tree_ascension ascension_cost has_changed />
                    </div>
                }
            })}
    }
}

#[component]
fn ConfirmAscendButton(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
    has_changed: Memo<bool>,
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
                                ascended_nodes: passives_tree_ascension
                                    .read_untracked()
                                    .ascended_nodes
                                    .clone(),
                            },
                        )
                        .await
                    {
                        Ok(response) => {
                            town_context.character.set(response.character);
                            town_context.passives_tree_ascension.set(response.ascension);
                            ascension_cost.set(0.0);
                            show_toast(toaster, "Ascension successful!", ToastVariant::Success)
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("Failed to ascend: {e}"),
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
fn RefundAscendButton(
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
                                ascended_nodes: Default::default(),
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
                            format!("Failed to refund: {e}"),
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

// Build Header
// -------------

#[component]
pub fn BuildPanelHeader(
    passives_tree_build: RwSignal<PurchasedNodes>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let cancel = move || {
        passives_tree_build.set(town_context.passives_tree_build.get());
    };

    let has_changed = Memo::new(move |_| {
        town_context
            .passives_tree_build
            .with(|base_build| !passives_tree_build.read().eq(base_build))
    });

    view! {
        {(!view_only)
            .then(|| {
                view! {
                    <div class="text-sm xl:text-base text-gray-400">
                        "Required Player Level:"
                        <span class="text-white font-semibold ml-2">
                            {move || passives_tree_build.read().len()}
                        </span>
                    </div>

                    <div class="flex-1" />

                    <div class="px-2 xl:px-4 relative z-10 flex items-center gap-2">
                        <ResetBuildButton passives_tree_build />
                    </div>

                    <div class="flex-1" />

                    <div class="flex items-center gap-2">
                        <MenuButton
                            on:click=move |_| cancel()
                            disabled=Signal::derive(move || !has_changed.get())
                        >
                            "Cancel"
                        </MenuButton>
                        <ConfirmBuildButton passives_tree_build has_changed />
                    </div>
                }
            })}
    }
}

#[component]
fn ConfirmBuildButton(
    passives_tree_build: RwSignal<PurchasedNodes>,
    has_changed: Memo<bool>,
) -> impl IntoView {
    let do_save = Arc::new({
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();

        let character_id = town_context.character.read_untracked().character_id;
        move || {
            spawn_local({
                async move {
                    match backend
                        .post_save_passives(
                            &auth_context.token(),
                            &SavePassivesRequest {
                                character_id,
                                purchased_nodes: passives_tree_build.get(),
                            },
                        )
                        .await
                    {
                        Ok(_) => {
                            town_context
                                .passives_tree_build
                                .set(passives_tree_build.get());
                            show_toast(toaster, "Export successful!", ToastVariant::Success);
                        }
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

    let try_save = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                "This will erase your last saved build. Do you confirm saving planned build?"
                    .into(),
                do_save.clone(),
            );
        }
    };

    let disabled = Signal::derive(move || !has_changed.get());

    view! {
        <MenuButton on:click=try_save disabled=disabled>
            "Export Build"
        </MenuButton>
    }
}

#[component]
fn ResetBuildButton(passives_tree_build: RwSignal<PurchasedNodes>) -> impl IntoView {
    let redo_stack = RwSignal::new(Vec::new());

    let undo = move |_| {
        if let Some(node_id) = passives_tree_build.write().pop() {
            redo_stack.write().push(node_id);
        }
    };

    let disable_undo = Signal::derive(move || passives_tree_build.read().is_empty());

    let redo = move |_| {
        if let Some(node_id) = redo_stack.write().pop() {
            passives_tree_build.write().insert(node_id);
        }
    };

    let disable_redo = Signal::derive(move || redo_stack.read().is_empty());

    let reset = move |_| {
        passives_tree_build.set(Default::default());
        redo_stack.set(Default::default());
    };

    view! {
        <MenuButton on:click=undo disabled=disable_undo>
            "Undo"
        </MenuButton>
        <MenuButton on:click=redo disabled=disable_redo>
            "Redo"
        </MenuButton>
        <MenuButton on:click=reset>"Reset"</MenuButton>
    }
}

// Tree (shared for performances)
// ------------------------------

#[component]
fn PassiveSkillTree(
    active_tab: RwSignal<PassivesTab>,
    search_node: RwSignal<Option<String>>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    passives_tree_build: RwSignal<PurchasedNodes>,
    ascension_cost: RwSignal<f64>,
    view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();
    let backend = expect_context::<BackendClient>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let character_id = town_context.character.read_untracked().character_id;

    let points_available = Memo::new(move |_| {
        if view_only {
            0.0
        } else {
            (town_context.character.read().resource_shards - ascension_cost.get()).round()
        }
    });

    let selected_socket_node = RwSignal::new(None);

    Effect::new(move || {
        if let Some(item_index) = town_context.selected_item_index.get()
            && let Some(passive_node_id) = selected_socket_node.get_untracked()
        {
            selected_socket_node.set(None);
            town_context.selected_item_index.set(None);
            spawn_local({
                async move {
                    match backend
                        .post_socket_passive(
                            &auth_context.token(),
                            &SocketPassiveRequest {
                                character_id,
                                passive_node_id,
                                item_index: Some(item_index),
                            },
                        )
                        .await
                    {
                        Ok(response) => {
                            passives_tree_ascension.write().socketed_nodes =
                                response.ascension.socketed_nodes.clone();
                            town_context.passives_tree_ascension.set(response.ascension);
                            town_context.inventory.set(response.inventory);
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("Failed to socket: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    });

    view! {
        <Pannable>
            <For
                each=move || {
                    town_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from, conn.to)
                let(conn)
            >
                <AscendConnection
                    connection=conn
                    passives_tree_specs=town_context.passives_tree_specs
                    passives_tree_ascension
                    passives_tree_build
                    active_tab
                />
            </For>
            <For
                each=move || { town_context.passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| *id
                let((id, node))
            >
                <AscendNode
                    node_id=id
                    node_specs=node
                    points_available
                    ascension_cost
                    passives_tree_ascension
                    passives_tree_build
                    selected_socket_node
                    active_tab
                    search_node
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
    active_tab: RwSignal<PassivesTab>,
    selected_socket_node: RwSignal<Option<PassiveNodeId>>,
    search_node: RwSignal<Option<String>>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    passives_tree_build: RwSignal<PurchasedNodes>,
    view_only: bool,
) -> impl IntoView {
    let town_context: TownContext = expect_context();
    let backend = expect_context::<BackendClient>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let socket = Memo::new(move |_| {
        passives_tree_ascension
            .read()
            .socketed_nodes
            .get(&node_id)
            .cloned()
    });

    let derived_node_specs = Memo::new({
        let node_specs = node_specs.clone();
        move |_| {
            if let Some(item_specs) = socket.get() {
                let mut node_specs = node_specs.clone();
                node_specs.icon = item_specs.base.icon.clone();
                node_specs.effects = (&(item_specs
                    .modifiers
                    .aggregate_effects(AffixEffectScope::Global)))
                    .into(); // TODO: Better copy, don't aggregate?
                node_specs.triggers = item_specs.base.triggers.clone();
                node_specs.initial_node |= item_specs
                    .base
                    .rune_specs
                    .as_ref()
                    .map(|rune_specs| rune_specs.root_node)
                    .unwrap_or_default();
                node_specs
            } else {
                node_specs.clone()
            }
        }
    });

    let node_level = Memo::new(move |_| match active_tab.get() {
        PassivesTab::Ascend => passives_tree_ascension
            .read()
            .ascended_nodes
            .get(&node_id)
            .copied()
            .unwrap_or_default(),
        PassivesTab::Build => town_context
            .passives_tree_ascension
            .read()
            .ascended_nodes
            .get(&node_id)
            .copied()
            .unwrap_or_default(),
    });

    let max_node_level = if node_specs.upgrade_effects.is_empty() {
        node_specs.locked as u8
    } else {
        node_specs.max_upgrade_level.unwrap_or(u8::MAX)
    };

    let connected_nodes: Vec<_> = town_context
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

    let max_upgrade_level = Memo::new({
        move |_| {
            let max_connection_level = if node_specs.initial_node
                || (active_tab.get() == PassivesTab::Build
                    && derived_node_specs.read().initial_node)
            {
                match active_tab.get() {
                    PassivesTab::Ascend => u8::MAX,
                    PassivesTab::Build => (!node_specs.locked || node_level.get() > 0) as u8,
                }
            } else {
                match active_tab.get() {
                    PassivesTab::Ascend => connected_nodes
                        .iter()
                        .map(|connected_node_id| {
                            passives_tree_ascension
                                .read()
                                .ascended_nodes
                                .get(connected_node_id)
                                .copied()
                                .unwrap_or_default()
                        })
                        .max()
                        .unwrap_or_default(),
                    PassivesTab::Build => {
                        (connected_nodes.iter().any(|connected_node_id| {
                            passives_tree_build.read().contains(connected_node_id)
                        }) && (!node_specs.locked || node_level.get() > 0))
                            as u8
                    }
                }
            };

            max_node_level.min(max_connection_level)
        }
    });

    let node_status = Memo::new(move |_| {
        let meta_status = if node_level.get() > 0 {
            MetaStatus::Ascended
        } else if node_specs.locked {
            MetaStatus::Locked
        } else {
            MetaStatus::Normal
        };

        let purchase_status = match active_tab.get() {
            PassivesTab::Ascend => {
                let upgradable = max_upgrade_level.get() > node_level.get();

                if (view_only || (node_level.get() == max_node_level && !node_specs.socket))
                    && node_level.get() > 0
                {
                    PurchaseStatus::Purchased
                } else if (points_available.get() > 0.0 && upgradable)
                    || (node_specs.socket && (!node_specs.locked || node_level.get() > 0))
                {
                    PurchaseStatus::Purchaseable
                } else {
                    PurchaseStatus::Inactive
                }
            }
            PassivesTab::Build => {
                if passives_tree_build.read().contains(&node_id) {
                    PurchaseStatus::Purchased
                } else if !view_only && max_upgrade_level.get() > 0 {
                    PurchaseStatus::Purchaseable
                } else {
                    PurchaseStatus::Inactive
                }
            }
        };
        NodeStatus {
            purchase_status,
            meta_status,
        }
    });

    let purchase = move || {
        match active_tab.get() {
            PassivesTab::Ascend => {
                if node_specs.socket && (!node_specs.locked || node_level.get() > 0) {
                    selected_socket_node.set(Some(node_id));
                    town_context.selected_item_index.set(None);
                    town_context
                        .use_item_category_filter
                        .set(Some(ItemCategory::Rune));
                    town_context.open_inventory.set(true);
                } else {
                    passives_tree_ascension.update(|passives_tree_ascension| {
                        let entry = passives_tree_ascension
                            .ascended_nodes
                            .entry(node_id)
                            .or_default();
                        *entry = entry.saturating_add(1);
                    });
                    ascension_cost.update(|ascension_cost| *ascension_cost += 1.0); // TODO: Ascend cost?
                }
            }
            PassivesTab::Build => {
                passives_tree_build.write().insert(node_id);
            }
        }
    };

    let refund = {
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            if let PassivesTab::Build = active_tab.get() {
            } else if passives_tree_ascension
                .read_untracked()
                .socketed_nodes
                .contains_key(&node_id)
            {
                spawn_local({
                    async move {
                        match backend
                            .post_socket_passive(
                                &auth_context.token(),
                                &SocketPassiveRequest {
                                    character_id,
                                    passive_node_id: node_id,
                                    item_index: None,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                passives_tree_ascension.write().socketed_nodes =
                                    response.ascension.socketed_nodes.clone();
                                town_context.passives_tree_ascension.set(response.ascension);
                                town_context.inventory.set(response.inventory);
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to socket: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            } else {
                passives_tree_ascension.update(|passives_tree_ascension| {
                    let entry = passives_tree_ascension
                        .ascended_nodes
                        .entry(node_id)
                        .or_default();
                    if *entry > 0 {
                        *entry = entry.saturating_sub(1);
                        ascension_cost.update(|ascension_cost| *ascension_cost -= 1.0);
                    }
                });
            }
        }
    };

    view! {
        {move || {
            view! {
                <Node
                    node_specs=derived_node_specs.get()
                    node_status
                    node_level
                    on_click=purchase
                    on_right_click=refund
                    show_upgrade=true
                    search_node
                />
            }
        }}
    }
}

#[component]
fn AscendConnection(
    connection: PassiveConnection,
    active_tab: RwSignal<PassivesTab>,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    passives_tree_build: RwSignal<PurchasedNodes>,
) -> impl IntoView {
    let town_context: TownContext = expect_context();

    let amount_connections = Memo::new(move |_| match active_tab.get() {
        PassivesTab::Ascend => {
            passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&connection.from)
                .map(|x| (*x > 0) as usize)
                .unwrap_or_default()
                + passives_tree_ascension
                    .read()
                    .ascended_nodes
                    .get(&connection.to)
                    .map(|x| (*x > 0) as usize)
                    .unwrap_or_default()
        }
        PassivesTab::Build => {
            passives_tree_build.read().contains(&connection.from) as usize
                + passives_tree_build.read().contains(&connection.to) as usize
        }
    });

    let node_levels = Memo::new(move |_| match active_tab.get() {
        PassivesTab::Ascend => (
            passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&connection.from)
                .cloned()
                .unwrap_or_default(),
            passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&connection.to)
                .cloned()
                .unwrap_or_default(),
        ),
        PassivesTab::Build => (
            town_context
                .passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&connection.from)
                .cloned()
                .unwrap_or_default(),
            town_context
                .passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&connection.to)
                .cloned()
                .unwrap_or_default(),
        ),
    });

    view! { <Connection connection passives_tree_specs amount_connections node_levels /> }
}
