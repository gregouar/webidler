use std::{sync::Arc, time::Duration};

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    routing::{delete, get, post},
};
use backend_shared::profanities_checker::ProfanitiesChecker;
use shared::{
    data::{
        area::AreaLevel,
        cosmetics::{CharacterCosmetics, CosmeticType},
        realms::Realm,
        stash::StashType,
        user::{UserCharacter, UserCharacterActivity, UserCharacterId, UserGrindArea, UserId},
    },
    http::{
        client::{CreateCharacterRequest, UpdateCharacterPetsRequest, UpdateCharacterRequest},
        server::{
            CreateCharacterResponse, DeleteCharacterResponse, GetCharacterDetailsResponse,
            GetUserCharactersResponse, ReconcileAchievementsResponse, UpdateCharacterPetsResponse,
            UpdateCharacterResponse,
        },
    },
    types::Username,
};

use crate::{
    app_state::{AppState, MasterStore},
    auth::{self, User},
    db,
    game::{
        data::{
            inventory_data::inventory_data_to_player_inventory,
            passives::ascension_data_to_passives_tree_ascension,
        },
        systems::{
            achievements_controller::{self, AchievementContext},
            skills_updater,
        },
    },
    rest::utils::{
        MsgPack, verify_character_in_town, verify_character_not_deleted, verify_character_user,
    },
};

use super::AppError;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let auth_routes = Router::new()
        .route("/users/{user_id}/characters", post(post_create_character))
        .route("/characters/{character_id}", get(get_character_details))
        .route("/characters/{character_id}", post(post_update_character))
        .route(
            "/characters/{character_id}/pets",
            post(post_update_character_pets),
        )
        .route(
            "/characters/{character_id}/achievements/reconcile",
            post(post_reconcile_achievements),
        )
        .route("/characters/{character_id}", delete(delete_character))
        .layer(middleware::from_fn_with_state(
            app_state,
            auth::authorization_middleware,
        ));

    Router::new()
        .route("/users/{user_id}/characters", get(get_user_characters))
        .route(
            "/view-character/{character_name}",
            get(get_character_by_name),
        )
        .merge(auth_routes)
    // .route("characters/{character_id}", get(get_character))
}

async fn post_create_character(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    State(profanities_checker): State<Arc<ProfanitiesChecker>>,
    Path(user_id): Path<UserId>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<CreateCharacterRequest>,
) -> Result<Json<CreateCharacterResponse>, AppError> {
    // TODO: better access management
    if current_user.user_id != user_id {
        return Err(AppError::Forbidden);
    }

    if profanities_checker.find_profanity(&payload.name).is_some() {
        return Err(AppError::UserError(
            "this name contains inappropriate language, please choose a different name".into(),
        ));
    }

    verify_portrait_unlocked(&db_pool, &master_store, &user_id, &payload.portrait).await?;

    let realm = match (payload.legacy, payload.is_ssf) {
        (true, true) => Realm::LegacySSF,
        (true, false) => Realm::Legacy,
        (false, true) => Realm::StandardSSF,
        (false, false) => Realm::Standard,
    };

    match db::characters::create_character(
        &db_pool,
        &user_id,
        &payload.name,
        &payload.portrait,
        realm,
        payload.is_ssf,
    )
    .await?
    {
        Some(character_id) => Ok(Json(CreateCharacterResponse { character_id })),
        None => Err(AppError::UserError("name already taken".to_string())),
    }
}

async fn get_user_characters(
    State(db_pool): State<db::DbPool>,
    Path(user_id): Path<UserId>,
) -> Result<Json<GetUserCharactersResponse>, AppError> {
    Ok(Json(GetUserCharactersResponse {
        characters: db::characters::read_all_user_characters(&db_pool, &user_id)
            .await?
            .into_iter()
            .map(|c| c.into())
            .collect(),
    }))
}

async fn get_character_details(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Path(character_id): Path<UserId>,
) -> Result<MsgPack<GetCharacterDetailsResponse>, AppError> {
    read_character_details(db_pool, master_store, character_id).await
}

async fn get_character_by_name(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Path(character_name): Path<Username>,
) -> Result<MsgPack<GetCharacterDetailsResponse>, AppError> {
    let character_id = db::characters::get_character_by_name(&db_pool, &character_name)
        .await?
        .ok_or(AppError::UserError(format!(
            "character '{}' not found",
            character_name.into_inner()
        )))?;

    read_character_details(db_pool, master_store, character_id).await
}

async fn read_character_details(
    db_pool: db::DbPool,
    master_store: MasterStore,
    character_id: UserCharacterId,
) -> Result<MsgPack<GetCharacterDetailsResponse>, AppError> {
    let character = db::characters::read_character(&db_pool, &character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let (
        user_unlocks,
        pets,
        areas_completed,
        character_data,
        passives_build,
        character_stash,
        user_stash,
        market_stash,
    ) = tokio::join!(
        db::user_unlocks::load_user_unlocks(&db_pool, &character.user_id),
        db::characters_data::load_character_pets(&db_pool, &character_id),
        db::characters::read_character_areas_completed(&db_pool, &character_id),
        db::characters_data::load_character_data(&db_pool, &character_id),
        db::characters_builds::load_character_build(&db_pool, &character_id),
        db::stashes::get_character_stash_by_type(&db_pool, &character, StashType::Character),
        db::stashes::get_character_stash_by_type(&db_pool, &character, StashType::User),
        db::stashes::get_character_stash_by_type(&db_pool, &character, StashType::Market),
    );

    let user_unlocks = user_unlocks?;
    let pets = pets?;
    let areas_completed = areas_completed?;
    let (inventory_data, ascension_data, benedictions, mut skill_masteries) =
        character_data?.unwrap_or_default();
    // let last_grind_data = last_grind_data?;
    let character_stash = character_stash?.map(|x| x.into());
    let user_stash = user_stash?.map(|x| x.into());
    let market_stash = market_stash?.map(|x| x.into());

    let areas: Vec<UserGrindArea> = master_store
        .area_blueprints_store
        .keys()
        .map(|area_id| {
            let (max_level_reached, max_power_shard_level, quest_completed) = areas_completed
                .iter()
                .find(|area_completed| area_completed.area_id.eq(area_id))
                .map(|area_completed| {
                    (
                        area_completed.max_area_level as AreaLevel,
                        area_completed.max_power_shard_level as AreaLevel,
                        area_completed.quest_completed,
                    )
                })
                .unwrap_or_default();

            UserGrindArea {
                area_id: area_id.clone(),
                max_level_reached,
                max_power_shard_level,
                quest_completed,
            }
        })
        .collect();

    let inventory = inventory_data_to_player_inventory(&master_store.items_store, inventory_data);
    let ascension =
        ascension_data_to_passives_tree_ascension(&master_store.items_store, ascension_data);
    let passives_build = passives_build?.unwrap_or_default();
    skill_masteries
        .favorite_skills
        .retain(|skill_id| master_store.skills_store.contains_key(skill_id));
    let skill_mastery_skill_specs = skills_updater::compute_skill_mastery_skill_specs(
        &master_store.statuses_store,
        &master_store.skills_store,
        &master_store.skill_masteries_store,
        &skill_masteries,
    );

    let character: UserCharacter = character.into();

    Ok(MsgPack(GetCharacterDetailsResponse {
        character,
        areas,
        inventory,
        ascension,
        passives_build,
        benedictions,
        // last_grind,
        character_stash,
        user_stash,
        market_stash,
        skill_masteries,
        skill_mastery_skill_specs,
        user_unlocks,
        pets,
    }))
}

async fn post_reconcile_achievements(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Path(character_id): Path<UserCharacterId>,
    Extension(user): Extension<User>,
) -> Result<Json<ReconcileAchievementsResponse>, AppError> {
    let mut tx = db_pool.begin().await?;
    let character = db::characters::read_character(&mut *tx, &character_id)
        .await?
        .ok_or(AppError::NotFound)?;
    verify_character_not_deleted(&character)?;
    verify_character_user(&character, &user)?;
    verify_character_in_town(&character)?;

    let areas = db::characters::read_character_areas_completed(&mut *tx, &character_id).await?;
    let (inventory_data, ascension_data, _, skill_masteries) =
        db::characters_data::load_character_data(&mut *tx, &character_id)
            .await?
            .ok_or(AppError::NotFound)?;
    let already_unlocked =
        db::user_unlocks::read_achievements(&mut *tx, &character.user_id).await?;

    let area_levels = areas
        .into_iter()
        .map(|area| (area.area_id, area.max_area_level as AreaLevel))
        .collect();
    let inventory = inventory_data_to_player_inventory(&master_store.items_store, inventory_data);
    let ascension =
        ascension_data_to_passives_tree_ascension(&master_store.items_store, ascension_data);

    let newly_unlocked_achievements = achievements_controller::check_achievements(
        &master_store,
        &already_unlocked,
        &AchievementContext {
            area_levels: &area_levels,
            power_level: character.max_area_level as AreaLevel,
            player_level: 0,
            skill_masteries: &skill_masteries,
            ascension: &ascension,
            inventory: &inventory,
        },
    );

    achievements_controller::unlock_achievements(
        &mut tx,
        &master_store,
        character.user_id,
        &newly_unlocked_achievements,
    )
    .await?;

    tx.commit().await?;

    let user_unlocks = db::user_unlocks::load_user_unlocks(&db_pool, &user.user_id).await?;

    Ok(Json(ReconcileAchievementsResponse {
        user_unlocks,
        newly_unlocked_achievements,
    }))
}

async fn post_update_character_pets(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    Path(character_id): Path<UserCharacterId>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateCharacterPetsRequest>,
) -> Result<Json<UpdateCharacterPetsResponse>, AppError> {
    let mut tx = db_pool.begin().await?;
    let character = db::characters::read_character(&mut *tx, &character_id)
        .await?
        .ok_or(AppError::NotFound)?;
    verify_character_not_deleted(&character)?;
    verify_character_user(&character, &user)?;

    let unlocked = db::user_unlocks::read_pets(&mut *tx, &user.user_id).await?;
    if payload
        .pets
        .values()
        .any(|id| !unlocked.contains(id) || !master_store.pets_store.contains_key(id))
    {
        return Err(AppError::UserError("pet is not unlocked".into()));
    }
    let mut unique_pets = std::collections::HashSet::new();
    if !payload.pets.values().all(|pet| unique_pets.insert(pet)) {
        return Err(AppError::UserError(
            "a pet can only be assigned to one button per character".into(),
        ));
    }

    db::characters_data::save_character_pets(&mut *tx, &character_id, &payload.pets).await?;
    tx.commit().await?;
    Ok(Json(UpdateCharacterPetsResponse {}))
}

async fn post_update_character(
    State(db_pool): State<db::DbPool>,
    State(master_store): State<MasterStore>,
    State(profanities_checker): State<Arc<ProfanitiesChecker>>,
    Path(character_id): Path<UserCharacterId>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateCharacterRequest>,
) -> Result<Json<UpdateCharacterResponse>, AppError> {
    let character = db::characters::read_character(&db_pool, &character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_not_deleted(&character)?;
    verify_character_user(&character, &user)?;

    if profanities_checker.find_profanity(&payload.name).is_some() {
        return Err(AppError::UserError(
            "this name contains inappropriate language, please choose a different name".into(),
        ));
    }

    let unlocked = db::user_unlocks::read_cosmetics(&db_pool, &user.user_id).await?;
    let validate = |id: &str, expected: fn(&CosmeticType) -> bool| {
        unlocked.contains(id) && master_store.cosmetics_store.get(id).is_some_and(expected)
    };
    if payload
        .cosmetics
        .title
        .as_deref()
        .is_some_and(|id| !validate(id, |c| matches!(c, CosmeticType::Title(_))))
        || payload
            .cosmetics
            .badge
            .as_deref()
            .is_some_and(|id| !validate(id, |c| matches!(c, CosmeticType::Badge(_))))
    {
        return Err(AppError::UserError("cosmetic is not unlocked".into()));
    }
    verify_portrait_unlocked(&db_pool, &master_store, &user.user_id, &payload.portrait).await?;

    match db::characters::update_character(
        &db_pool,
        &character_id,
        &payload.name,
        &payload.portrait,
        &payload.cosmetics,
    )
    .await?
    {
        Some(_) => Ok(Json(UpdateCharacterResponse {})),
        None => Err(AppError::UserError("name already taken".to_string())),
    }
}

async fn verify_portrait_unlocked(
    db_pool: &db::DbPool,
    master_store: &MasterStore,
    user_id: &UserId,
    portrait_name: &str,
) -> Result<(), AppError> {
    let portrait = master_store.cosmetics_store.iter().find(|(id, cosmetic)| {
        matches!(
            cosmetic,
            CosmeticType::Portrait(specs)
                if *id == portrait_name

        )
    });
    let Some((cosmetic_id, CosmeticType::Portrait(specs))) = portrait else {
        return Err(AppError::UserError("unknown portrait".into()));
    };
    if specs.locked
        && !db::user_unlocks::read_cosmetics(db_pool, user_id)
            .await?
            .contains(cosmetic_id)
    {
        return Err(AppError::UserError("portrait is not unlocked".into()));
    }
    Ok(())
}

async fn delete_character(
    State(db_pool): State<db::DbPool>,
    Path(character_id): Path<UserCharacterId>,
    Extension(user): Extension<User>,
) -> Result<Json<DeleteCharacterResponse>, AppError> {
    let character = db::characters::read_character(&db_pool, &character_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_character_user(&character, &user)?;
    verify_character_in_town(&character)?;

    db::characters::delete_character(&db_pool, &character_id).await?;
    Ok(Json(DeleteCharacterResponse {}))
}

impl From<db::characters::CharacterEntry> for UserCharacter {
    fn from(val: db::characters::CharacterEntry) -> Self {
        UserCharacter {
            user_id: val.user_id,
            realm: (&val.realm_id).into(),
            character_id: val.character_id,
            name: val.character_name,
            portrait: val.portrait,
            cosmetics: CharacterCosmetics {
                title: val.cosmetic_title,
                badge: val.cosmetic_badge,
            },
            is_ssf: val.is_ssf,
            resource_gems: val.resource_gems,
            resource_shards: val.resource_shards,
            resource_gold: val.resource_gold,
            resource_stamina: Duration::from_secs_f64(val.resource_stamina),
            played_time: Duration::from_secs_f64(val.played_time_seconds),
            max_area_level: val.max_area_level as AreaLevel,
            activity: if let (Some(area_id), Some(area_level)) = (val.area_id, val.area_level) {
                UserCharacterActivity::Grinding(area_id, area_level as AreaLevel)
            } else {
                UserCharacterActivity::Rusting
            },
        }
    }
}
