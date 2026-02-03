use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*};
use leptos_use::storage;

use shared::{
    data::{area::StartAreaConfig, user::UserCharacterId},
    messages::{
        client::ClientConnectMessage,
        server::{ErrorType, InitGameMessage, ServerMessage, SyncGameStateMessage},
    },
};

use crate::components::{
    auth::AuthContext,
    game::{
        GameContext,
        battle_scene::BattleScene,
        header_menu::HeaderMenu,
        panels::{EndQuestPanel, GameInventoryPanel, PassivesPanel, SkillsPanel, StatisticsPanel},
    },
    ui::{toast::*, tooltip::DynamicTooltip},
    websocket::WebsocketContext,
};

#[component]
pub fn GameInstance() -> impl IntoView {
    let game_context = GameContext::new();
    provide_context(game_context);

    let auth_context = expect_context::<AuthContext>();

    let (get_character_id_storage, _, _) =
        storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");

    let (get_area_config_storage, _, _) =
        storage::use_session_storage::<Option<StartAreaConfig>, JsonSerdeCodec>("area_config");

    Effect::new({
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if conn.connected.get() {
                conn.send(
                    &ClientConnectMessage {
                        jwt: auth_context.token(),
                        character_id: get_character_id_storage.get_untracked(),
                        area_config: get_area_config_storage.get_untracked(),
                    }
                    .into(),
                );
            }
        }
    });

    Effect::new({
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if let Some(message) = conn.message.get() {
                handle_message(&game_context, message);
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />
            <Show
                when=move || game_context.started.get()
                fallback=move || view! { <p>"Connecting..."</p> }
            >
                <HeaderMenu />
                <div class="relative flex-1">
                    <BattleScene />
                    <PassivesPanel open=game_context.open_passives />
                    <StatisticsPanel open=game_context.open_statistics />
                    <SkillsPanel open=game_context.open_skills />
                    <GameInventoryPanel open=game_context.open_inventory />
                    <EndQuestPanel />
                </div>
            </Show>
        </main>
    }
}

fn handle_message(game_context: &GameContext, message: ServerMessage) {
    match message {
        ServerMessage::Connect(_) => {}
        ServerMessage::InitGame(m) => {
            init_game(game_context, m);
        }
        ServerMessage::UpdateGame(m) => {
            sync_game(game_context, m);
        }
        ServerMessage::Error(error_message) => {
            let toaster = expect_context::<Toasts>();
            show_toast(
                toaster,
                error_message.message,
                match error_message.error_type {
                    ErrorType::Server => ToastVariant::Error,
                    ErrorType::Game => ToastVariant::Warning,
                },
            );
            if error_message.must_disconnect {
                let navigate = leptos_router::hooks::use_navigate();
                navigate("/", Default::default());
            }
        }
        ServerMessage::Disconnect(_) => {
            let navigate = leptos_router::hooks::use_navigate();
            // TODO: Bring to summary page on end_quest...
            navigate("/town", Default::default());
        }
    }
}

fn init_game(game_context: &GameContext, init_message: InitGameMessage) {
    let InitGameMessage {
        area_specs,
        area_state,
        passives_tree_specs,
        passives_tree_state,
        player_specs,
        player_state,
        last_skills_bought,
    } = init_message;

    game_context.started.set(true);
    game_context.area_specs.set(area_specs);
    game_context.area_state.set(area_state);
    game_context.passives_tree_specs.set(passives_tree_specs);
    game_context.passives_tree_state.set(passives_tree_state);
    game_context.player_specs.set(player_specs);
    game_context.player_state.set(player_state);
    game_context.last_skills_bought.set(last_skills_bought);
}

fn sync_game(game_context: &GameContext, sync_message: SyncGameStateMessage) {
    let SyncGameStateMessage {
        area_state,
        area_threat,
        passives_tree_state,
        player_specs,
        player_inventory,
        player_state,
        player_resources,
        player_stamina,
        monster_specs,
        monster_states,
        queued_loot,
        game_stats,
        quest_rewards,
    } = sync_message;

    game_context.area_state.sync(area_state);
    game_context.area_threat.set(area_threat);
    game_context.passives_tree_state.sync(passives_tree_state);
    game_context.player_specs.sync(player_specs);
    if let Some(player_inventory) = player_inventory {
        game_context.player_inventory.set(player_inventory);
    }
    game_context.player_resources.sync(player_resources);
    game_context.player_state.set(player_state);
    game_context.player_stamina.set(player_stamina);
    if let Some(monster_specs) = monster_specs {
        *game_context.monster_wave.write() += 1; // TODO: Overflow
        game_context.monster_specs.set(monster_specs);
    }
    game_context.monster_states.set(monster_states);
    game_context.queued_loot.sync(queued_loot);
    game_context.game_stats.set(game_stats);
    if let Some(quest_rewards) = quest_rewards {
        game_context.quest_rewards.set(quest_rewards);
    }
}
