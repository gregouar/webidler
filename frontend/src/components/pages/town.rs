use codee::string::JsonSerdeCodec;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::{storage, watch_debounced};

use shared::{
    data::user::{UserCharacterActivity, UserCharacterId},
    http::server::GetCharacterDetailsResponse,
};

use crate::components::{
    backend_client::{BackendClient, BackendError},
    data_context::DataContext,
    shared::{
        achievements::{AchievementsPanel, notify_newly_unlocked_achievements},
        pets::PetsPanel,
        player_count::PlayerCount,
        settings::SettingsModal,
    },
    town::{
        TownContext,
        header_menu::HeaderMenu,
        panels::{
            forge::ForgePanel,
            inventory::TownInventoryPanel,
            market::MarketPanel,
            passives::PassivesPanel,
            skill_masteries::{SkillMasteriesPanel, SkillMasteryDetailsModal},
            stash::StashPanel,
            temple::TemplePanel,
        },
        town_scene::TownScene,
    },
    ui::{loading_screen::LoadingScreen, toast::Toasts},
};

#[component]
pub fn TownPage() -> impl IntoView {
    let town_context = TownContext::default();
    provide_context(town_context);

    let data_context: DataContext = expect_context();
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();

    let (get_character_id_storage, _, _) =
        storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");
    let town_loaded = RwSignal::new(false);
    let static_data_loaded = RwSignal::new(false);
    let achievements_request_in_flight = RwSignal::new(false);
    let achievements_request_pending = RwSignal::new(false);

    let reconcile_achievements = Callback::new(move |()| {
        if achievements_request_in_flight.get_untracked() {
            achievements_request_pending.set(true);
            return;
        }

        achievements_request_in_flight.set(true);
        spawn_local(async move {
            loop {
                achievements_request_pending.set(false);
                let character_id = town_context.character.read_untracked().character_id;
                match backend.post_reconcile_achievements(&character_id).await {
                    Ok(response) => {
                        town_context.user_unlocks.set(response.user_unlocks);
                        notify_newly_unlocked_achievements(
                            data_context,
                            toaster,
                            response.newly_unlocked_achievements,
                        );
                    }
                    Err(error) => {
                        leptos::logging::error!("Failed to reconcile town achievements: {error}")
                    }
                }

                if !achievements_request_pending.get_untracked() {
                    break;
                }
            }
            achievements_request_in_flight.set(false);
        });
    });

    let _ = watch_debounced(
        move || {
            town_context.character.track();
            town_context.areas.track();
            town_context.inventory.track();
            town_context.passives_tree_ascension.track();
            town_context.player_skill_masteries.track();
            town_loaded.get() && static_data_loaded.get()
        },
        move |loaded, _, _| {
            if *loaded {
                reconcile_achievements.run(());
            }
        },
        1000.0,
    );

    let data_load = LocalResource::new({
        move || async move {
            if data_context.load_data(backend).await.is_err() {
                use_navigate()("/", Default::default());
            } else {
                static_data_loaded.set(true);
            }
        }
    });

    let initial_load = LocalResource::new({
        move || async move {
            match backend
                .get_character_details(&get_character_id_storage.get())
                .await
            {
                Ok(GetCharacterDetailsResponse {
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
                }) => {
                    if let UserCharacterActivity::Grinding(_, _) = character.activity {
                        use_navigate()("/game", Default::default())
                    }
                    town_context
                        .character_cosmetics
                        .set(character.cosmetics.clone());
                    town_context.player_pets.set(pets);
                    town_context.character.set(character);
                    town_context.areas.set(areas);
                    town_context.inventory.set(inventory);
                    town_context.passives_tree_ascension.set(ascension);
                    town_context.passives_tree_build.set(passives_build);
                    town_context.player_benedictions.set(benedictions);
                    town_context.player_skill_masteries.set(skill_masteries);
                    town_context
                        .skill_mastery_skill_specs
                        .set(skill_mastery_skill_specs);

                    town_context.user_unlocks.set(user_unlocks);
                    if let Some(character_stash) = character_stash {
                        town_context.character_stash.set(character_stash);
                    }
                    if let Some(user_stash) = user_stash {
                        town_context.user_stash.set(user_stash);
                    }
                    if let Some(market_stash) = market_stash {
                        town_context.market_stash.set(market_stash);
                    }
                    town_loaded.set(true);
                }
                Err(BackendError::Unauthorized(_) | BackendError::NotFound) => {
                    use_navigate()("/", Default::default())
                }
                _ => {} // TODO: Toast error ?
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <PlayerCount />

            <Transition fallback=move || {
                view! { <LoadingScreen detail="Loading your character and town data." /> }
            }>
                {move || Suspend::new(async move {
                    data_load.await;
                    initial_load.await;
                    view! {
                        <HeaderMenu />
                        <div class="relative flex-1">
                            <TownScene />
                            <TemplePanel open=town_context.open_temple />
                            <SkillMasteriesPanel open=town_context.open_skill_masteries />
                            <SkillMasteryDetailsModal />
                            <MarketPanel open=town_context.open_market />
                            <StashPanel open=town_context.open_stash />
                            <PassivesPanel open=town_context.open_ascend />
                            <ForgePanel open=town_context.open_forge />
                            <TownInventoryPanel open=town_context.open_inventory />
                            <SettingsModal open=town_context.open_settings />
                            <AchievementsPanel
                                open=town_context.open_achievements
                                user_unlocks=town_context.user_unlocks
                            />
                            <PetsPanel
                                open=town_context.open_pets
                                character_id=Signal::derive(move || {
                                    town_context.character.read().character_id
                                })
                                player_pets=town_context.player_pets
                                user_unlocks=town_context.user_unlocks
                            />
                        </div>
                    }
                })}
            </Transition>

        </main>
    }
}
