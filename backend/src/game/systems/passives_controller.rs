use shared::data::{
    passive::{PassiveNodeId, PassivesTreeSpecs, PassivesTreeState},
    player::PlayerResources,
    stat_effect::EffectsMap,
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
        if (node_specs.initial_node
            || passives_tree_specs
                .connections
                .iter()
                .filter(|connection| {
                    passives_tree_state
                        .purchased_nodes
                        .contains(&connection.from)
                        || passives_tree_state.purchased_nodes.contains(&connection.to)
                })
                .any(|connection| connection.from == node_id || connection.to == node_id))
            && passives_tree_state.purchased_nodes.insert(node_id)
        {
            player_resources.passive_points -= 1;
        }
    }
}

pub fn generate_effects_map_from_passives<'a>(
    passives_tree_specs: &'a PassivesTreeSpecs,
    passives_tree_state: &'a PassivesTreeState,
) -> impl Iterator<Item = EffectsMap> + use<'a> {
    passives_tree_state
        .purchased_nodes
        .iter()
        .filter_map(|node_id| {
            passives_tree_specs.nodes.get(node_id).map(|node| {
                node.aggregate_effects(
                    passives_tree_state
                        .ascension
                        .ascended_nodes
                        .get(node_id)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
        })
}
