use crate::{auth::CurrentUser, db::characters::CharacterEntry, rest::AppError};

pub fn verify_character_user(
    character: &CharacterEntry,
    current_user: &CurrentUser,
) -> Result<(), AppError> {
    if character.user_id != current_user.user_details.user.user_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub fn verify_character_in_town(character: &CharacterEntry) -> Result<(), AppError> {
    if character.area_id.is_some() {
        return Err(AppError::UserError("character is grinding".to_string()));
    }
    Ok(())
}
