use leptos::html::*;
use leptos::prelude::*;
use shared::data::MonsterPrototype;
use shared::data::MonsterState;
use shared::data::PlayerPrototype;
use shared::data::PlayerState;
use shared::messages::client::ClientConnectMessage;
use shared::messages::server::ServerMessage;

use crate::components::websocket::WebsocketContext;
use crate::components::{
    icons::{
        attack_icon::AttackIcon, bite_icon::BiteIcon, claw_icon::ClawIcon,
        fireball_icon::FireballIcon,
    },
    ui::buttons::MainMenuButton,
    ui::progress_bars::{CircularProgressBar, HorizontalProgressBar, VerticalProgressBar},
    websocket::Websocket,
};

#[derive(Clone)]
struct GameContext {
    started: RwSignal<bool>,

    player_prototype: RwSignal<PlayerPrototype>,
    player_state: RwSignal<PlayerState>,

    monster_prototypes: RwSignal<Vec<MonsterPrototype>>,
    monster_states: RwSignal<Vec<MonsterState>>,
}

impl GameContext {
    pub fn new() -> Self {
        GameContext {
            started: RwSignal::new(false),
            player_prototype: RwSignal::new(PlayerPrototype::default()),
            player_state: RwSignal::new(PlayerState::default()),
            monster_prototypes: RwSignal::new(Vec::new()),
            monster_states: RwSignal::new(Vec::new()),
        }
    }
}

#[component]
pub fn Game() -> impl IntoView {
    view! {
        <Websocket>
            <GameInstance/>
        </Websocket>
    }
}

#[component]
fn GameInstance() -> impl IntoView {
    let game_context = GameContext::new();
    provide_context(game_context.clone());

    Effect::new({
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if conn.connected.get() {
                conn.send(
                    &ClientConnectMessage {
                        bearer: String::from("Le Pou"),
                    }
                    .into(),
                );
            }
        }
    });

    Effect::new({
        let game_context = game_context.clone();
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if let Some(m) = conn.message.get() {
                handle_message(&game_context, &m);
            }
        }
    });

    view! {
        <main class="my-0 mx-auto text-center">
            <Show
                when=move || game_context.started.get()
                fallback=move || view! { <p>"Connecting..."</p> }
            >
            <div class="grid grid-cols-8 justify-items-stretch flex items-start gap-4 m-4 ">
                <SideMenu class:col-span-2 />
                <AdventurerPanel class:col-span-3 class:justify-self-end/>
                <MonstersPanel class:col-span-3 class:justify-self-start/>
            </div>
            </Show>
        </main>
    }
}

#[component]
fn SideMenu() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/", Default::default());

    view! {
        <div class="flex flex-col space-y-2 p-2 bg-zinc-800 rounded-md">
            <div>
                <p class="text-shadow-md shadow-gray-950 text-amber-200 text-2xl">
                    Menu
                </p>
            </div>
            <MainMenuButton>
                "Inventory"
            </MainMenuButton>
            <MainMenuButton>
                "Passive Skills"
            </MainMenuButton>
            <MainMenuButton>
                "Statistics"
            </MainMenuButton>
            <MainMenuButton on:click=abandon_quest>
                "Abandon Quest"
            </MainMenuButton>
        </div>
    }
}

#[component]
fn AdventurerPanel() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let player_name = move || {
        game_context
            .player_prototype
            .read()
            .character_prototype
            .name
            .clone()
    };

    // TODO: get_asset?
    let player_portrait = move || {
        format!(
            "./assets/{}",
            game_context
                .player_prototype
                .read()
                .character_prototype
                .portrait,
        )
    };

    let health_percent = Signal::derive(move || {
        let max_health = game_context
            .player_prototype
            .read()
            .character_prototype
            .max_health;
        if max_health > 0 {
            (game_context.player_state.read().character_state.health * 100 / max_health) as f32
        } else {
            0.0
        }
    });

    let mana_percent = Signal::derive(move || {
        let max_mana = game_context.player_prototype.read().max_mana;
        if max_mana > 0 {
            (game_context.player_state.read().mana * 100 / max_mana) as f32
        } else {
            0.0
        }
    });

    let (action_bar, set_action_bar) = signal(69.0);
    view! {
        <div class="flex flex-col gap-2 p-2 bg-zinc-800 rounded-md h-full">
            <div>
                <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">
                    {player_name}
                </p>
            </div>

            <div class="flex gap-2">
                <VerticalProgressBar class:w-3 class:md:w-6 bar_color="bg-gradient-to-b from-red-500 to-red-700" value=health_percent />
                <div class="flex-1">
                    <img src=player_portrait alt="adventurer" class="border-8 border-double border-stone-500" />
                </div>
                <VerticalProgressBar class:w-3 class:md:w-6 bar_color="bg-gradient-to-b from-blue-500 to-blue-700" value=mana_percent />
            </div>

            <div class="grid grid-cols-4 gap-2">
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon  class:drop-shadow-lg  class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar   bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <FireballIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <FireballIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
            </div>
        </div>
    }
}

#[component]
fn MonstersPanel() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    view! {
        <div class="grid grid-cols-2 grid-rows-3 gap-2 h-full">
            <For
                // a function that returns the items we're iterating over; a signal is fine
                each=move || game_context.monster_prototypes.get() // TODO: Read?
                // a unique key for each item
                key=|p| p.character_prototype.identifier
                // renders each item to a view
                children=move |p: MonsterPrototype| {
                    // TODO: find proper way to correlate proto and state... maybe should
                    // be other way around?
                    // TODO: Get rid of the .get() and use .read()
                    view! {
                        // <button>"Value: " {move || counter.count.get()}</button>
                        <MonsterPanel prototype=p />
                    }
                }
            />
            // <MonsterPanel />
            // <MonsterPanel />
            // <MonsterPanel />
            // <MonsterPanel />
            // <MonsterPanel />
            // <MonsterPanel />
        </div>
    }
}

#[component]
fn MonsterPanel(prototype: MonsterPrototype) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    // let monster_state = move || {
    //     // TODO: Get rid of get and use read
    //     game_context
    //         .monster_states
    //         .get()
    //         .get(prototype.character_prototype.identifier as usize - 1)
    // };

    let health_percent = Signal::derive(move || {
        let max_health = prototype.character_prototype.max_health;
        if max_health > 0 {
            (game_context
                .monster_states
                .read()
                .get(prototype.character_prototype.identifier as usize - 1)
                .map(|s| s.character_state.health)
                .unwrap_or(0)
                * 100
                / prototype.character_prototype.max_health) as f32
        } else {
            0.0
        }
    });

    let (action_bar, set_action_bar) = signal(42.0);
    view! {
        <div class="flex w-full bg-zinc-800 rounded-md gap-2 p-2">
            <div class="flex flex-col gap-2">
                <HorizontalProgressBar
                    class:h-2 class:sm:h-4 bar_color="bg-gradient-to-b from-red-500 to-red-700"
                    value=health_percent text=prototype.character_prototype.name.clone()
                />
                <div class="flex-1">
                    <img src={format!("./assets/{}",prototype.character_prototype.portrait)} alt=prototype.character_prototype.name  class="h-full border-8 border-double border-stone-500"/>
                </div>
            </div>
            <div class="flex flex-col justify-evenly w-full min-w-16">
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <BiteIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <ClawIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
            </div>
        </div>
    }
}

fn handle_message(game_context: &GameContext, message: &ServerMessage) {
    match message {
        ServerMessage::Connect(_) => {}
        ServerMessage::InitGame(m) => {
            game_context.started.set(true);
            game_context
                .player_prototype
                .set(m.player_prototype.clone());
            game_context.player_state.set(m.player_state.clone());
        }
        ServerMessage::UpdateGame(m) => {
            game_context.player_state.set(m.player_state.clone());
            if let Some(monster_prototypes) = m.monster_prototypes.as_ref() {
                game_context
                    .monster_prototypes
                    .set(monster_prototypes.clone());
            }
            game_context.monster_states.set(m.monster_states.clone());
        }
    }
}
