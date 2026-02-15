use std::collections::{HashMap, VecDeque};

use shared::data::{
    area::AreaLevel,
    item::{ItemRarity, ItemSpecs},
    item_affix::AffixEffectScope,
    passive::{PassiveNodeId, PassivesTreeAscension, PassivesTreeSpecs, PassivesTreeState},
    player::PlayerResources,
    stat_effect::EffectsMap,
    user::UserCharacterId,
};
use sqlx::Transaction;

use crate::{
    app_state::MasterStore,
    db::{self, characters::CharacterAreaEntry, pool::Database},
    game::data::master_store::AreaBlueprintStore,
    rest::AppError,
};

pub fn purchase_node(
    player_resources: &mut PlayerResources,
    passives_tree_specs: &PassivesTreeSpecs,
    passives_tree_state: &mut PassivesTreeState,
    node_id: PassiveNodeId,
) -> Result<(), AppError> {
    if player_resources.passive_points == 0 {
        return Err(AppError::UserError("Not enough points!".into()));
    }

    if let Some(node_specs) = passives_tree_specs.nodes.get(&node_id) {
        if !node_specs.root_node
            && !(passives_tree_specs
                .connections
                .iter()
                .filter(|connection| {
                    passives_tree_state
                        .purchased_nodes
                        .contains(&connection.from)
                        || passives_tree_state.purchased_nodes.contains(&connection.to)
                })
                .any(|connection| connection.from == node_id || connection.to == node_id))
        {
            return Err(AppError::UserError("Missing connection to node!".into()));
        }

        if node_specs.locked
            && passives_tree_state
                .ascension
                .ascended_nodes
                .get(&node_id)
                .copied()
                .unwrap_or_default()
                == 0
        {
            return Err(AppError::UserError("Passive node is locked!".into()));
        }

        if !passives_tree_state.purchased_nodes.insert(node_id) {
            return Err(AppError::UserError(
                "Passive node was already purchased!".into(),
            ));
        }

        player_resources.passive_points -= 1;
        Ok(())
    } else {
        Err(AppError::UserError("Unknown passive node...".into()))
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

pub fn compute_passives_tree_specs(
    passives_tree_specs: &mut PassivesTreeSpecs,
    passives_tree_ascension: &PassivesTreeAscension,
) {
    // TODO: Could compute ascension effects here to have them ready?

    for (passive_node_id, item_specs) in passives_tree_ascension.socketed_nodes.iter() {
        if let Some(node_specs) = passives_tree_specs.nodes.get_mut(passive_node_id) {
            node_specs.icon = item_specs.base.icon.clone();
            node_specs.name = item_specs.modifiers.name.clone();

            node_specs.effects = (&(item_specs
                .modifiers
                .aggregate_effects(AffixEffectScope::Global)))
                .into(); // TODO: Better copy, don't aggregate?
            node_specs.triggers = item_specs.base.triggers.clone();
            node_specs.root_node |= item_specs
                .base
                .rune_specs
                .as_ref()
                .map(|rune_specs| rune_specs.root_node)
                .unwrap_or_default();
        }
    }
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

    let passives_tree_specs = master_store
        .passives_store
        .get("default")
        .ok_or(anyhow::anyhow!("passives tree not found"))?;

    let cost = validate_ascension(passives_tree_specs, passives_tree_ascension)?;
    let total_shards = compute_total_shards(&master_store.area_blueprints_store, &areas_completed);

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

pub fn compute_ascension_cost(passives_tree_ascension: &PassivesTreeAscension) -> f64 {
    passives_tree_ascension
        .ascended_nodes
        .values()
        .map(|v| *v as f64)
        .sum()
}

pub fn validate_ascension(
    passives_tree_specs: &PassivesTreeSpecs,
    passives_tree_ascension: &PassivesTreeAscension,
) -> Result<f64, AppError> {
    let mut cost = 0.0;

    let max_level_tree =
        compute_max_level_ascension_tree(passives_tree_specs, passives_tree_ascension);

    for (node_id, level) in passives_tree_ascension.ascended_nodes.iter() {
        let node_specs = passives_tree_specs
            .nodes
            .get(node_id)
            .ok_or(AppError::UserError(
                "invalid ascension: missing node".into(),
            ))?;

        if *level > node_specs.max_upgrade_level.unwrap_or(u8::MAX) {
            return Err(AppError::UserError(
                "invalid ascension: level too high".into(),
            ));
        }

        if *level > max_level_tree.get(node_id).copied().unwrap_or_default() {
            return Err(AppError::UserError(
                "invalid ascension: missing connection".into(),
            ));
        }

        cost += (*level) as f64;
    }

    Ok(cost)
}

fn compute_max_level_ascension_tree(
    passives_tree_specs: &PassivesTreeSpecs,
    passives_tree_ascension: &PassivesTreeAscension,
) -> HashMap<PassiveNodeId, u8> {
    let mut propagated_tree = HashMap::new();

    let mut queue: VecDeque<(PassiveNodeId, u8)> = passives_tree_specs
        .nodes
        .iter()
        .filter(|(_, node_specs)| node_specs.root_node)
        .map(|(node_id, _)| {
            (
                *node_id,
                passives_tree_ascension
                    .ascended_nodes
                    .get(node_id)
                    .copied()
                    .unwrap_or_default(),
            )
        })
        .collect();

    while let Some((node_id, level)) = queue.pop_front() {
        let level = passives_tree_ascension
            .ascended_nodes
            .get(&node_id)
            .copied()
            .unwrap_or_default()
            .min(level);

        let entry = propagated_tree.entry(node_id).or_default();
        if level <= *entry {
            continue;
        }
        *entry = level;

        // TODO: Could split connections in 2 hashmap or something
        for connection in &passives_tree_specs.connections {
            if connection.from == node_id {
                queue.push_back((connection.to, level));
            } else if connection.to == node_id {
                queue.push_back((connection.from, level));
            }
        }
    }

    propagated_tree
}

fn compute_total_shards(
    area_blueprints_store: &AreaBlueprintStore,
    areas_completed: &[CharacterAreaEntry],
) -> f64 {
    areas_completed
        .iter()
        .map(|area| {
            if area_blueprints_store
                .get(&area.area_id)
                .map(|area_blueprint| !area_blueprint.specs.disable_shards)
                .unwrap_or_default()
            {
                (area.max_area_level / 10) as f64
            } else {
                0.0
            }
        })
        .sum()
}

pub fn socket_node(
    master_store: &MasterStore,
    max_item_level: AreaLevel,
    passives_tree_ascension: &mut PassivesTreeAscension,
    passive_node_id: PassiveNodeId,
    item_specs: Option<ItemSpecs>,
) -> Result<Option<ItemSpecs>, AppError> {
    // TODO: Check it is Rune and level is enough
    let passives_tree_specs = master_store
        .passives_store
        .get("default")
        .ok_or(anyhow::anyhow!("passives tree not found"))?;

    if let Some(item_specs) = item_specs {
        if item_specs.base.rune_specs.is_none() {
            return Err(AppError::UserError(
                "Only Runes can be socketed into Passives Tree".into(),
            ));
        }

        if item_specs.required_level > max_item_level {
            return Err(AppError::UserError("level too low".into()));
        }

        if !passives_tree_specs
            .nodes
            .get(&passive_node_id)
            .map(|node_specs| node_specs.socket)
            .unwrap_or_default()
        {
            return Err(AppError::UserError("node is not a socket".into()));
        }

        if item_specs.modifiers.rarity == ItemRarity::Unique
            && passives_tree_ascension.socketed_nodes.iter().any(
                |(socket_node_id, socketed_item_specs)| {
                    socketed_item_specs.modifiers.base_item_id == item_specs.modifiers.base_item_id
                        && *socket_node_id != passive_node_id
                },
            )
        {
            return Err(AppError::UserError(
                "cannot socket twice the same Unique Rune".into(),
            ));
        }

        Ok(passives_tree_ascension
            .socketed_nodes
            .insert(passive_node_id, item_specs))
    } else {
        Ok(passives_tree_ascension
            .socketed_nodes
            .remove(&passive_node_id))
    }
}
