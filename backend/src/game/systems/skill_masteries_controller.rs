use shared::data::{skill_mastery::PlayerSkillMasteries, user::UserCharacterId};
use sqlx::Transaction;

use crate::{
    app_state::MasterStore,
    db::{self, pool::Database},
    game::data::master_store::SkillMasteriesStore,
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
        &requested_skill_masteries,
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

        let mastery_level = prev_mastery.level().min(mastery_specs.max_level);
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
