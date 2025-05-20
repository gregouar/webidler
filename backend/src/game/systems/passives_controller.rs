use shared::data::{
    passive::{PassiveNodeId, PassivesTreeSpecs, PassivesTreeState},
    player::PlayerResources,
};

pub fn purchase_node(
    player_resources: &mut PlayerResources,
    passives_tree_specs: &PassivesTreeSpecs,
    passives_tree_state: &mut PassivesTreeState,
    node_id: PassiveNodeId,
) {
    if player_resources.passive_points == 0 {
        return;
    }

    if let Some(node_specs) = passives_tree_specs.nodes.get(&node_id) {
        if (node_specs.initial_node || passives_tree_specs
                .connections
                .iter()
                .filter(|connection| {
                    passives_tree_state
                        .purchased_nodes
                        .get(&connection.from)
                        .is_some()
                        || passives_tree_state
                            .purchased_nodes
                            .get(&connection.to)
                            .is_some()
                })
                .any(|connection| connection.from == node_id || connection.to == node_id)) && passives_tree_state.purchased_nodes.insert(node_id) {
            player_resources.passive_points -= 1;
        }
    }
}
