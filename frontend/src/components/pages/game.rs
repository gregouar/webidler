use leptos::html::*;
use leptos::prelude::*;

use rand::Rng;

use shared::data::MonsterPrototype;
use shared::data::MonsterState;
use shared::data::PlayerPrototype;
use shared::data::PlayerState;
use shared::data::SkillPrototype;
use shared::messages::client::ClientConnectMessage;
use shared::messages::server::ServerMessage;

use crate::components::websocket::WebsocketContext;
use crate::components::{
    ui::buttons::MainMenuButton,
    ui::progress_bars::{CircularProgressBar, HorizontalProgressBar, VerticalProgressBar},
    websocket::Websocket,
};

#[derive(Clone)]
struct GameContext {
    started: RwSignal<bool>,

    player_prototype: RwSignal<PlayerPrototype>,
    player_state: RwSignal<PlayerState>,

    monster_wave: RwSignal<usize>, // Used to generate unique key in list
    monster_prototypes: RwSignal<Vec<MonsterPrototype>>,
    monster_states: RwSignal<Vec<MonsterState>>,
}

impl GameContext {
    fn new() -> Self {
        GameContext {
            started: RwSignal::new(false),
            player_prototype: RwSignal::new(PlayerPrototype::default()),
            player_state: RwSignal::new(PlayerState::default()),
            monster_wave: RwSignal::new(0),
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
        <main class="my-0 mx-auto text-center overflow-x-hidden">
            <Show
                when=move || game_context.started.get()
                fallback=move || view! { <p>"Connecting..."</p> }
            >
                <Menu  />
                <div class="grid grid-cols-3 justify-items-stretch flex items-start gap-4 m-4 ">
                    // <SideMenu class:col-span-2 />
                    <PlayerCard class:col-span-1 class:justify-self-end/>
                    <MonstersGrid class:col-span-2 class:justify-self-start/>
                </div>
            </Show>
        </main>
    }
}

#[component]
fn Menu() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/", Default::default());

    view! {
        <div class="flex flex-row items-center space-x-2 p-2 bg-zinc-800 shadow-md">
            <div>
                <p class="text-shadow-md shadow-gray-950 text-amber-200 text-2xl">
                    "Menu"
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
fn PlayerCard() -> impl IntoView {
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
        if max_health > 0.0 {
            (game_context.player_state.read().character_state.health / max_health * 100.0) as f32
        } else {
            0.0
        }
    });

    let mana_percent = Signal::derive(move || {
        let max_mana = game_context.player_prototype.read().max_mana;
        if max_mana > 0.0 {
            (game_context.player_state.read().mana / max_mana * 100.0) as f32
        } else {
            0.0
        }
    });

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
                <img src=player_portrait alt="player" class="border-8 border-double border-stone-500" />
            </div>
            <VerticalProgressBar class:w-3 class:md:w-6 bar_color="bg-gradient-to-b from-blue-500 to-blue-700" value=mana_percent />
        </div>

        <div class="grid grid-cols-4 gap-2">
            <For
                each=move || game_context.player_prototype.get().character_prototype.skill_prototypes.into_iter().enumerate()
                key=|(i,_)|  *i
                let((i,p))
            >
                <PlayerSkill prototype=p index=i />
            </For>
        </div>
    </div>
    }
}

#[component]
fn PlayerSkill(prototype: SkillPrototype, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let skill_cooldown = Signal::derive(move || {
        if prototype.cooldown > 0.0 {
            (game_context
                .player_state
                .read()
                .character_state
                .skill_states
                .get(index)
                .map(|x| x.elapsed_cooldown)
                .unwrap_or_default()
                * 100.0
                / prototype.cooldown) as f32
        } else {
            0.0
        }
    });

    // TODO: Skill component

    let just_triggered = Signal::derive(move || {
        game_context
            .player_state
            .read()
            .character_state
            .skill_states
            .get(index)
            .map(|x| x.just_triggered)
            .unwrap_or_default()
    });

    view! {
        <CircularProgressBar
            bar_width=4
            bar_color="text-amber-700"
            value=skill_cooldown
            reset=just_triggered
        >
            <img
                src={format!("./assets/{}",prototype.icon.clone())} alt=prototype.name
                class="invert drop-shadow-lg w-full h-full flex-no-shrink fill-current"
            />
        </CircularProgressBar>
    }
}

#[component]
fn MonstersGrid() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let all_monsters_dead = Signal::derive(move || {
        game_context
            .monster_states
            .read()
            .iter()
            .all(|x| !x.character_state.is_alive)
    });

    // TODO: double buffering to allow in and out at the same time
    view! {
        <div class="">
            <div class="w-full h-16 bg-[url(./assets/worlds/forest_header.webp)] bg-center bg-repeat-x ">
            </div>
            <div class="grid grid-cols-3 grid-rows-2 mt-2 mb-2 gap-2 grid-flow-col items-center h-full overflow-hidden">
                <style>"
                    @keyframes monster-fade-in {
                        0% { transform: translateX(100%); opacity: 0; }
                        65% { transform: translateX(0%); opacity: 1; }
                        80% { transform: translateX(5%); }
                        100% { transform: translateX(0%); }
                    }
                    
                    @keyframes monster-fade-out {
                        from { opacity: 1; transform: translateY(0%); }
                        to { opacity: 0; transform: translateY(100%); }
                    }
                "</style>
                <For
                    each=move || game_context.monster_prototypes.get().into_iter().enumerate()
                    // We need a unique key to replace old elements
                    key=move |(index,_)| (game_context.monster_wave.get(), *index)
                    children=move |(index, prototype)| {
                        let animation_delay = format!("animation-delay: {}s;", rand::rng().random_range(0.0..=0.2f32));
                        view! {
                            <div
                                style=move|| if all_monsters_dead.get()
                                    {format!("animation: monster-fade-out 1s ease-in; animation-fill-mode: both;")}
                                    else {format!("animation: monster-fade-in 1s ease-out; animation-fill-mode: both; {}",animation_delay)}
                            >
                                <MonsterCard prototype=prototype index=index />
                            </div>
                        }
                    }
                />
            </div>
            <div class="w-full h-16 bg-[url(./assets/worlds/forest_footer.webp)] bg-center bg-repeat-x ">
            </div>
        </div>
    }
}

#[component]
fn MonsterCard(prototype: MonsterPrototype, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let health_percent = Signal::derive(move || {
        let max_health = prototype.character_prototype.max_health;
        if max_health > 0.0 {
            (game_context
                .monster_states
                .read()
                .get(index)
                .map(|s| s.character_state.health)
                .unwrap_or_default()
                / prototype.character_prototype.max_health
                * 100.0) as f32
        } else {
            0.0
        }
    });

    let is_dead = move || {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or(false)
    };

    let is_dead_img_effect = move || {
        if is_dead() {
            "saturate-0 brightness-1"
        } else {
            ""
        }
    };

    view! {
        <div class="flex w-full bg-zinc-800 rounded-md gap-2 p-2">
            <div class="flex flex-col gap-2">
                <HorizontalProgressBar
                    class:h-2 class:sm:h-4 bar_color="bg-gradient-to-b from-red-500 to-red-700"
                    value=health_percent text=prototype.character_prototype.name.clone()
                />
                <div class="flex-1">
                    <img
                        src={format!("./assets/{}",prototype.character_prototype.portrait.clone())} alt=prototype.character_prototype.name
                        class=move || format!("h-full border-8 border-double border-stone-500 transition duration-1000 {}", is_dead_img_effect())
                    />
                </div>
            </div>
            <div class="flex flex-col justify-evenly w-full min-w-16">
                <For
                    each=move || prototype.character_prototype.skill_prototypes.clone().into_iter().enumerate()
                    key=|(i,_)|  *i
                    let((i,p))
                >
                    <MonsterSkill prototype=p index=i monster_index=index />
                </For>
            </div>
        </div>
    }
}

#[component]
fn MonsterSkill(prototype: SkillPrototype, index: usize, monster_index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let is_dead = move || {
        game_context
            .monster_states
            .read()
            .get(monster_index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or(false)
    };

    let skill_cooldown = Signal::derive(move || {
        if !is_dead() && prototype.cooldown > 0.0 {
            (game_context
                .monster_states
                .read()
                .get(monster_index)
                .map(|m| m.character_state.skill_states.get(index))
                .flatten()
                .map(|s| s.elapsed_cooldown)
                .unwrap_or(0.0)
                * 100.0
                / prototype.cooldown) as f32
        } else {
            0.0
        }
    });

    let just_triggered = Signal::derive(move || {
        if !is_dead() {
            game_context
                .monster_states
                .read()
                .get(monster_index)
                .map(|m| m.character_state.skill_states.get(index))
                .flatten()
                .map(|s| s.just_triggered)
                .unwrap_or_default()
        } else {
            false
        }
    });

    view! {
        <CircularProgressBar
            bar_width=4 bar_color="text-amber-700"
            value=skill_cooldown
            reset=just_triggered
        >
            <img
                src={format!("./assets/{}",prototype.icon.clone())} alt=prototype.name
                class="invert drop-shadow-lg w-full h-full flex-no-shrink fill-current"
            />
        </CircularProgressBar>
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
                *game_context.monster_wave.write() += 1; // TODO: Overflow
                game_context
                    .monster_prototypes
                    .set(monster_prototypes.clone());
            }
            game_context.monster_states.set(m.monster_states.clone());
        }
    }
}
