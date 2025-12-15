use std::collections::{HashMap, VecDeque};

use shared::data::{
    passive::{PassiveNodeId, PassivesTreeAscension, PassivesTreeSpecs, PassivesTreeState},
    player::PlayerResources,
    stat_effect::EffectsMap,
    user::UserCharacterId,
};
use sqlx::Transaction;

use crate::{
    app_state::MasterStore,
    db::{self, characters::CharacterAreaEntry, pool::Database},
    rest::AppError,
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

pub fn refund_missing(
    passives_tree_specs: &PassivesTreeSpecs,
    passives_tree_state: &mut PassivesTreeState,
    player_resources: &mut PlayerResources,
) {
    passives_tree_state.purchased_nodes.retain(|node_id| {
        let keep = passives_tree_specs.nodes.contains_key(node_id);
        if !keep {
            player_resources.passive_points += 1;
        }
        keep
    });
}

pub async fn update_ascension(
    tx: &mut Transaction<'_, Database>,
    master_store: &MasterStore,
    character_id: &UserCharacterId,
    resource_shards: f64,
    passives_tree_ascension: &PassivesTreeAscension,
) -> Result<(), AppError> {
    let areas_completed =
        db::characters::read_character_areas_completed(&mut **tx, character_id).await?;

    let passive_tree_specs = master_store
        .passives_store
        .get("default")
        .ok_or(anyhow::anyhow!("passives tree not found"))?;

    let cost = validate_ascension(passive_tree_specs, passives_tree_ascension)?;
    let total_shards = compute_total_shards(&areas_completed);

    if cost > total_shards {
        return Err(AppError::UserError("not enough power shards".to_string()));
    }

    db::characters::update_character_resources(
        &mut **tx,
        character_id,
        0.0,
        (total_shards - cost) - resource_shards,
        0.0,
    )
    .await?;

    db::characters_data::save_character_passives(&mut **tx, character_id, passives_tree_ascension)
        .await?;

    Ok(())
}

pub fn compute_ascension_cost(passive_tree_ascension: &PassivesTreeAscension) -> f64 {
    passive_tree_ascension
        .ascended_nodes
        .values()
        .map(|v| *v as f64)
        .sum()
}

pub fn validate_ascension(
    passives_tree_specs: &PassivesTreeSpecs,
    passive_tree_ascension: &PassivesTreeAscension,
) -> anyhow::Result<f64> {
    let mut cost = 0.0;

    let max_level_tree =
        compute_max_level_ascension_tree(passives_tree_specs, passive_tree_ascension);

    for (node_id, level) in passive_tree_ascension.ascended_nodes.iter() {
        let node_specs = passives_tree_specs
            .nodes
            .get(node_id)
            .ok_or(anyhow::anyhow!("invalid ascension: missing node"))?;

        if *level > node_specs.max_upgrade_level.unwrap_or(u8::MAX) {
            return Err(anyhow::anyhow!("invalid ascension: level too high"));
        }

        if *level > max_level_tree.get(node_id).copied().unwrap_or_default() {
            return Err(anyhow::anyhow!("invalid ascension: missing connection"));
        }

        cost += (*level) as f64;
    }

    Ok(cost)
}

fn compute_max_level_ascension_tree(
    passives_tree_specs: &PassivesTreeSpecs,
    passive_tree_ascension: &PassivesTreeAscension,
) -> HashMap<PassiveNodeId, u8> {
    let mut propagated_tree = HashMap::new();

    let mut queue: VecDeque<(PassiveNodeId, u8)> = passives_tree_specs
        .nodes
        .iter()
        .filter(|(_, node_specs)| node_specs.initial_node)
        .map(|(node_id, _)| {
            (
                node_id.clone(),
                passive_tree_ascension
                    .ascended_nodes
                    .get(node_id)
                    .copied()
                    .unwrap_or_default(),
            )
        })
        .collect();

    while let Some((node_id, level)) = queue.pop_front() {
        let level = passive_tree_ascension
            .ascended_nodes
            .get(&node_id)
            .copied()
            .unwrap_or_default()
            .min(level);

        let entry = propagated_tree.entry(node_id.clone()).or_default();
        if level <= *entry {
            continue;
        }
        *entry = level;

        // TODO: Could split connections in 2 hashmap or something
        for connection in &passives_tree_specs.connections {
            if connection.from == node_id {
                queue.push_back((connection.to.clone(), level));
            } else if connection.to == node_id {
                queue.push_back((connection.from.clone(), level));
            }
        }
    }

    propagated_tree
}

fn compute_total_shards(areas_completed: &[CharacterAreaEntry]) -> f64 {
    areas_completed
        .iter()
        .map(|area| (area.max_area_level / 10) as f64)
        .sum()
}
