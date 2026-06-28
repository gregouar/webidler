use shared::data::{
    item_affix::AffixEffectScope,
    player::PlayerBaseSpecs,
    skill::SkillSpecs,
    skill_mastery::{
        PlayerSkillMasteries, SkillMasterySpecs, SkillMasteryState, SkillMasteryUpgradeEffectType,
    },
    stat_effect::StatEffect,
    user::UserCharacterId,
};
use sqlx::Transaction;

use crate::{
    app_state::MasterStore,
    db::{self, pool::Database},
    game::{
        data::master_store::{SkillMasteriesStore, StatusesStore},
        systems::skills_updater,
    },
    rest::AppError,
};

pub async fn update_skill_masteries(
    tx: &mut Transaction<'_, Database>,
    master_store: &MasterStore,
    character_id: &UserCharacterId,
    requested_skill_masteries: &PlayerSkillMasteries,
) -> Result<(), AppError> {
    let (_, _, _, prev_skill_masteries) =
        db::characters_data::load_character_data(&mut **tx, character_id)
            .await?
            .unwrap_or_default();

    if requested_skill_masteries.favorite_skills.len() > 4 {
        return Err(anyhow::anyhow!("invalid favorite skills").into());
    }

    validate_skill_masteries(
        &master_store.skill_masteries_store,
        &prev_skill_masteries,
        requested_skill_masteries,
    )?;

    db::characters_data::save_character_skill_masteries(
        &mut **tx,
        character_id,
        requested_skill_masteries,
    )
    .await?;

    Ok(())
}

pub fn validate_skill_masteries(
    skill_masteries_store: &SkillMasteriesStore,
    prev_skill_masteries: &PlayerSkillMasteries,
    requested_skill_masteries: &PlayerSkillMasteries,
) -> anyhow::Result<()> {
    for (skill_id, requested_mastery) in requested_skill_masteries.masteries.iter() {
        let prev_mastery = prev_skill_masteries
            .masteries
            .get(skill_id)
            .cloned()
            .unwrap_or_default();

        let mastery_specs = skill_masteries_store
            .get(skill_id)
            .cloned()
            .unwrap_or_default();

        if prev_mastery.experience != requested_mastery.experience {
            return Err(anyhow::anyhow!("invalid skill mastery"));
        }

        let mastery_level = prev_mastery.level(mastery_specs.max_level);
        let mut spent_points = 0u16;

        for (upgrade_id, upgrade_level) in requested_mastery.upgrades_bought.iter() {
            let Some(upgrade_specs) = mastery_specs.upgrades.get(upgrade_id) else {
                return Err(anyhow::anyhow!("invalid skill mastery"));
            };

            if *upgrade_level > upgrade_specs.max_level {
                return Err(anyhow::anyhow!("invalid skill mastery"));
            }

            spent_points = spent_points.saturating_add(upgrade_specs.compute_cost(*upgrade_level));
        }

        if spent_points > mastery_level {
            return Err(anyhow::anyhow!("invalid skill mastery"));
        }
    }

    Ok(())
}

pub fn apply_skill_mastery(
    statuses_store: &StatusesStore,
    skill_specs: &mut SkillSpecs,
    skill_mastery_specs: &SkillMasterySpecs,
    skill_mastery_state: &SkillMasteryState,
) {
    let upgrade_effects = skill_mastery_specs
        .upgrades
        .iter()
        .filter_map(|(upgrade_id, mastery_upgrade)| {
            let upgrade_level = skill_mastery_state
                .upgrades_bought
                .get(upgrade_id)
                .copied()
                .unwrap_or_default();

            if upgrade_level > 0 {
                Some((mastery_upgrade, upgrade_level))
            } else {
                None
            }
        })
        .flat_map(|(mastery_upgrade, upgrade_level)| {
            itertools::iproduct!(
                mastery_upgrade.effects.iter(),
                std::iter::once(upgrade_level)
            )
        });

    for (upgrade_effect, _) in upgrade_effects.clone() {
        match &upgrade_effect.effect_type {
            SkillMasteryUpgradeEffectType::StatEffect { .. }
            // | SkillMasteryUpgradeEffectType::PlayerStatEffect { .. } 
            => {}
            SkillMasteryUpgradeEffectType::SkillEffect {
                skill_effect,
                target_index,
            } => {
                if let Some(target_group) = skill_specs.targets.get_mut(*target_index) {
                    target_group.effects.push(skill_effect.clone());
                }
            }
            SkillMasteryUpgradeEffectType::Trigger(trigger_specs) => {
                skill_specs.triggers.push(trigger_specs.clone());
            }
        }
    }

    let stat_effects: Vec<_> = upgrade_effects
        .filter(|(effect, _)| {
            matches!(
                effect.effect_type,
                SkillMasteryUpgradeEffectType::StatEffect {
                    scope: AffixEffectScope::Local,
                    ..
                }
            )
        })
        .filter_map(|(effect, upgrade_level)| effect.compute_stat_effect(upgrade_level))
        .collect();

    skills_updater::apply_effects_to_skill_specs(statuses_store, skill_specs, stat_effects.iter());
}

pub fn generate_player_stat_effects(
    skill_masteries_store: &SkillMasteriesStore,
    player_base_specs: &PlayerBaseSpecs,
) -> Vec<StatEffect> {
    player_base_specs
        .skills
        .keys()
        .take(player_base_specs.max_skills as usize)
        .filter_map(|skill_id| {
            skill_masteries_store
                .get(skill_id)
                .zip(player_base_specs.skill_masteries.masteries.get(skill_id))
        })
        .flat_map(|(mastery_specs, mastery_state)| {
            mastery_specs
                .upgrades
                .iter()
                .flat_map(|(upgrade_id, mastery_upgrade)| {
                    let upgrade_level = mastery_state
                        .upgrades_bought
                        .get(upgrade_id)
                        .copied()
                        .unwrap_or_default();

                    mastery_upgrade
                        .effects
                        .iter()
                        .filter(|effect| {
                            matches!(
                                effect.effect_type,
                                SkillMasteryUpgradeEffectType::StatEffect {
                                    scope: AffixEffectScope::Global,
                                    ..
                                }
                            )
                        })
                        .filter_map(move |effect| effect.compute_stat_effect(upgrade_level))
                })
        })
        .collect()
}
