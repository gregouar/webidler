use axum::{
    http::header,
    response::{IntoResponse, Response},
};

use shared::data::user::User;

use crate::{db::characters::CharacterEntry, rest::AppError};

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

pub fn verify_ssf(character: &CharacterEntry) -> Result<(), AppError> {
    if character.is_ssf {
        return Err(AppError::UserError(
            "SSF character cannot do that".to_string(),
        ));
    }
    Ok(())
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
