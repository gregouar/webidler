use anyhow::{anyhow, Result};

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use rand::TryRngCore;
use tokio::{task::yield_now, time::timeout};

use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use std::{ops::ControlFlow, vec};

use shared::{
    data::{
        character::CharacterSize,
        item::ItemRarity,
        item_affix::EffectsMap,
        player::{CharacterSpecs, PlayerInventory, PlayerResources, PlayerSpecs, PlayerState},
        skill::SkillSpecs,
    },
    messages::{
        client::{ClientConnectMessage, ClientMessage},
        server::ConnectMessage,
        SessionKey, UserId,
    },
};

use crate::game::{
    data::{master_store::MasterStore, DataInit},
    game_instance_data::GameInstanceData,
    session::{Session, SessionsStore},
    systems::{loot_generator, player_controller},
    GameInstance,
};
use crate::websocket::WebSocketConnection;

const CLIENT_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(sessions_store): State<SessionsStore>,
    State(master_store): State<MasterStore>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, sessions_store, master_store))
}

async fn handle_socket(
    socket: WebSocket,
    who: SocketAddr,
    sessions_store: SessionsStore,
    master_store: MasterStore,
) {
    let mut conn = WebSocketConnection::establish(socket, who, CLIENT_INACTIVITY_TIMEOUT);

    tracing::debug!("waiting for client to connect...");
    let (user_id, session) = match timeout(
        Duration::from_secs(30),
        wait_for_connect(&master_store, &sessions_store, &mut conn),
    )
    .await
    {
        Err(e) => {
            tracing::error!("connection timeout: {}", e);
            return;
        }
        Ok(Err(e)) => {
            tracing::error!("unable to connect: {}", e);
            return;
        }
        Ok(Ok(p)) => p,
    };
    tracing::debug!("client connected");

    tracing::debug!("starting the game...");

    let game = GameInstance::new(&mut conn, session.data, master_store);
    match game.run().await {
        Ok(game_data) => {
            sessions_store.sessions.lock().unwrap().insert(
                user_id,
                Session {
                    session_key: session.session_key,
                    last_active: Instant::now(),
                    data: game_data,
                },
            );
        }
        Err(e) => tracing::error!("error running game: {e}"),
    }

    // returning from the handler closes the websocket connection
    tracing::info!("websocket context {who} destroyed");
}

async fn wait_for_connect(
    master_store: &MasterStore,
    sessions_store: &SessionsStore,
    conn: &mut WebSocketConnection,
) -> Result<(UserId, Session)> {
    loop {
        match conn.poll_receive() {
            ControlFlow::Continue(Some(ClientMessage::Connect(msg))) => {
                return handle_connect(sessions_store, master_store, conn, msg).await;
            }
            ControlFlow::Break(_) => {
                return Err(anyhow::format_err!("disconnected"));
            }
            _ => {}
        }
        yield_now().await;
    }
}

async fn handle_connect(
    sessions_store: &SessionsStore,
    master_store: &MasterStore,
    conn: &mut WebSocketConnection,
    msg: ClientConnectMessage,
) -> Result<(UserId, Session)> {
    // TODO: verify if user exist, is already playing, get basic data etc
    tracing::info!("Connect: {:?}", msg);

    let session = match msg.session_key {
        Some(session_key) => {
            handle_resume_session(&msg.user_id, sessions_store, session_key).await?
        }
        None => handle_new_session(&msg.user_id, master_store).await?,
    };

    conn.send(
        &ConnectMessage {
            greeting: msg.user_id.clone(),
            session_key: session.session_key.clone(),
        }
        .into(),
    )
    .await?;

    Ok((msg.user_id, session))
}

async fn handle_resume_session(
    user_id: &str,
    sessions_store: &SessionsStore,
    session_key: SessionKey,
) -> Result<Session> {
    if let Some(session) = sessions_store.sessions.lock().unwrap().get(user_id) {
        if session_key == session.session_key {
            return Ok(session.clone());
        }
    }
    Err(anyhow!("couldn't load player session"))
}

async fn handle_new_session(user_id: &str, master_store: &MasterStore) -> Result<Session> {
    let mut rng = rand::rng();
    let mut session_key: SessionKey = [0u8; 32];
    rng.try_fill_bytes(&mut session_key)?;

    let passives_tree_specs = master_store.passives_store.get("default").unwrap().clone();

    let mut player_specs = PlayerSpecs {
        character_specs: CharacterSpecs {
            name: user_id.to_string(), // TODO: LOL
            portrait: String::from("adventurers/human_male_2.webp"),
            size: CharacterSize::Small,
            position_x: 0,
            position_y: 0,
            max_life: 100.0,
            life_regen: 1.0,
            armor: 0.0,
            fire_armor: 0.0,
            poison_armor: 0.0,
        },
        skills_specs: vec![
            SkillSpecs::init(master_store.skills_store.get("magic_missile").unwrap()),
            SkillSpecs::init(master_store.skills_store.get("fireball").unwrap()),
            SkillSpecs::init(master_store.skills_store.get("heal").unwrap()),
        ],
        level: 1,
        experience_needed: 20.0,
        max_mana: 100.0,
        mana_regen: 1.0,
        movement_cooldown: 2.0,
        gold_find: 1.0,
        effects: EffectsMap::default(),
        auto_skills: vec![false, false, false],
    };

    let mut player_inventory = PlayerInventory::default();
    player_inventory.max_bag_size = 40;

    let player_resources = PlayerResources::default();

    tracing::debug!("loading the game...");
    let world_blueprint = match master_store
        .world_blueprints_store
        .get("forest.json")
        .cloned() // TODO: Avoid clone?
    {
        Some(world_blueprint) => world_blueprint,
        None => {
            return Err(anyhow!("couldn't load world: forest.json"));
        }
    };

    let mut player_state = PlayerState::init(&player_specs); // How to avoid this?

    if let Some(base_weapon) = master_store.items_store.get("shortsword").cloned() {
        let _ = player_controller::equip_item(
            &mut player_specs,
            &mut player_inventory,
            &mut player_state,
            loot_generator::roll_item(
                base_weapon,
                ItemRarity::Normal,
                1,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
            ),
        );
    }

    Ok(Session {
        session_key,
        last_active: Instant::now(),
        data: Box::new(GameInstanceData::init(
            world_blueprint,
            passives_tree_specs,
            player_resources,
            player_specs,
            player_inventory,
        )),
    })
}
