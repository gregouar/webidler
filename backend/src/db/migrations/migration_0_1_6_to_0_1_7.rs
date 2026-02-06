use shared::data::{passive::PassivesTreeAscension, user::UserCharacterId};

use crate::{app_state::MasterStore, db::DbPool, game::systems::passives_controller};

pub async fn migrate(db_pool: &DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    // let old_entries = sqlx::query!(
    //     r#"
    //     SELECT
    //         characters.character_id as "character_id: UserCharacterId",
    //         characters.resource_shards
    //     FROM characters
    //     INNER JOIN characters_data ON characters_data.character_id = characters.character_id
    //     WHERE characters_data.data_version < '0.1.6'
    //     "#
    // )
    // .fetch_all(&mut *tx)
    // .await?;

    // for old_entry in old_entries {
    //     passives_controller::update_ascension(
    //         &mut tx,
    //         master_store,
    //         &old_entry.character_id,
    //         old_entry.resource_shards,
    //         &PassivesTreeAscension::default(),
    //     )
    //     .await?;
    // }

    tx.commit().await?;
    Ok(())
}

// TODO: NIGHTMARE CONVERT AFFIXES AWAWAWAWAWAWA
