use axum::{
    http::header,
    response::{IntoResponse, Response},
};

use shared::data::{
    player::PlayerInventory,
    user::{User, UserCharacterId},
};
use sqlx::Transaction;

use crate::{
    app_state::MasterStore,
    db::{self, characters::CharacterEntry, pool::Database},
    game::data::inventory_data::inventory_data_to_player_inventory,
    rest::AppError,
};

pub fn verify_character_user(character: &CharacterEntry, user: &User) -> Result<(), AppError> {
    if character.user_id != user.user_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub fn verify_character_not_deleted(character: &CharacterEntry) -> Result<(), AppError> {
    if character.deleted_at.is_some() {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn verify_character_in_town(character: &CharacterEntry) -> Result<(), AppError> {
    verify_character_not_deleted(character)?;
    if character.area_id.is_some() {
        return Err(AppError::UserError("character is grinding".to_string()));
    }
    Ok(())
}

pub fn verify_not_ssf(character: &CharacterEntry) -> Result<(), AppError> {
    if character.is_ssf {
        return Err(AppError::UserError(
            "SSF character cannot do that".to_string(),
        ));
    }
    Ok(())
}

pub async fn load_player_town_inventory(
    tx: &mut Transaction<'_, Database>,
    master_store: &MasterStore,
    user: &User,
    character_id: &UserCharacterId,
) -> Result<(CharacterEntry, PlayerInventory), AppError> {
    let character = db::characters::read_character(&mut **tx, character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, user)?;
    verify_character_in_town(&character)?;

    let inventory_data = db::characters_data::load_inventory_data(&mut **tx, character_id)
        .await?
        .ok_or(AppError::UserError("newbies can't do that".into()))?;

    Ok((
        character,
        inventory_data_to_player_inventory(&master_store.items_store, inventory_data),
    ))
}

pub async fn buy_with_gems(
    tx: &mut Transaction<'_, Database>,
    character_id: &UserCharacterId,
    price: f64,
) -> Result<f64, AppError> {
    let character_resources =
        db::characters::update_character_resources(&mut **tx, character_id, -price, 0.0, 0.0, 0.0)
            .await?;

    if character_resources.resource_gems < 0.0 {
        return Err(AppError::UserError("not enough gems".into()));
    }

    Ok(character_resources.resource_gems)
}

pub struct MsgPack<T>(pub T);

impl<T> IntoResponse for MsgPack<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        let bytes = rmp_serde::to_vec(&self.0).expect("msgpack serialization failed");

        ([(header::CONTENT_TYPE, "application/msgpack")], bytes).into_response()
    }
}
