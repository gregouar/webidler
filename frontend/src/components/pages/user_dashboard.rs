use std::{collections::HashMap, sync::Arc};

use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local, web_sys};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::{
    data::{
        area::AreaSpecs,
        user::{User, UserCharacter, UserCharacterActivity, UserCharacterId, UserId},
    },
    http::{client::CreateCharacterRequest, server::NewsEntry},
    types::AssetName,
};

use crate::{
    assets::img_asset,
    components::{
        accessibility::AccessibilityContext,
        auth::AuthContext,
        backend_client::BackendClient,
        shared::player_count::PlayerCount,
        ui::{
            buttons::{MenuButton, MenuButtonRed},
            confirm::ConfirmContext,
            input::ValidatedInput,
            menu_panel::MenuPanel,
            number::format_datetime,
            toast::*,
        },
    },
};

#[component]
pub fn UserDashboardPage() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u64);

    let async_data = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        let auth_context = expect_context::<AuthContext>();
        move || async move {
            let _ = refresh_trigger.read();

            let areas = backend
                .get_areas()
                .await
                .map(|r| r.areas)
                .unwrap_or_default();

            let user = backend
                .get_me(&auth_context.token())
                .await
                .map(|r| r.user_details.user)
                .ok();

            match user {
                Some(user) => {
                    let characters = backend
                        .get_user_characters(&auth_context.token(), &user.user_id)
                        .await
                        .map(|r| r.characters)
                        .unwrap_or_default();
                    Some((areas, user, characters))
                }
                None => None,
            }
        }
    });

    let exit_fullscreen = move || {
        if let Some(doc) = web_sys::window().and_then(|w| w.document())
            && doc.fullscreen_element().is_some()
        {
            doc.exit_fullscreen();
        }
    };

    let sign_out = {
        let navigate = use_navigate();
        let auth_context = expect_context::<AuthContext>();
        move || {
            exit_fullscreen();
            auth_context.sign_out();
            navigate("/", Default::default());
        }
    };

    Effect::new({
        let sign_out = sign_out.clone();
        move || {
            if async_data.get().map(|x| x.is_none()).unwrap_or_default() {
                sign_out()
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full max-h-screen text-center overflow-x-hidden flex flex-col">
            <DiscordInviteBanner />
            <PlayerCount />

            <div class="relative flex-1 max-w-6xl w-full mx-auto p-2 xl:p-4 gap-2 xl:gap-4 flex flex-col ">
                <Transition fallback=move || {
                    view! { <p class="text-gray-400">"Loading..."</p> }
                }>
                    {move || {
                        let sign_out = sign_out.clone();
                        Suspend::new(async move {
                            let (areas, user, characters) = async_data.await.unwrap_or_default();
                            let areas = Arc::new(areas);
                            view! {
                                <h1 class="mb-2 text-shadow-lg shadow-gray-950 text-amber-200 text-2xl xl:text-4xl font-extrabold leading-none tracking-tight">
                                    "Welcome, " {user.username.clone()}"!"
                                </h1>

                                <div class="w-full grid grid-cols-1 lg:grid-cols-2 gap-2 xl:gap-4">
                                    <NewsPanel />
                                    <CharactersSelection
                                        areas=areas.clone()
                                        characters
                                        user
                                        refresh_trigger
                                    />
                                </div>

                                <div class="w-full bg-zinc-800 rounded-xl ring-1 ring-zinc-950 shadow-xl
                                flex items-center justify-between gap-2 text-gray-400 p-2 xl:p-4">
                                    <a href="leaderboard">
                                        <MenuButton>"Leaderboard"</MenuButton>
                                    </a>
                                    <a
                                        href="https://webidler.gitbook.io/wiki/"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <MenuButton>"Wiki"</MenuButton>
                                    </a>
                                    <a href="account">
                                        <MenuButton>"Account Settings"</MenuButton>
                                    </a>
                                    <MenuButtonRed on:click=move |_| sign_out()>
                                        "Sign Out"
                                    </MenuButtonRed>
                                </div>
                            }
                        })
                    }}
                </Transition>
            </div>
        </main>
    }
}

#[component]
fn CharactersSelection(
    areas: Arc<HashMap<String, AreaSpecs>>,
    characters: Vec<UserCharacter>,
    user: User,
    refresh_trigger: RwSignal<u64>,
) -> impl IntoView {
    let open_create_character = RwSignal::new(false);

    let characters_len = characters.len();

    view! {
        <CreateCharacterPanel
            open=open_create_character
            user_id=user.user_id
            refresh_trigger=refresh_trigger
        />

        <div class="flex flex-col h-full min-h-0 bg-zinc-800 rounded-xl ring-1 ring-zinc-950 shadow-xl p-4 text-left space-y-4">
            <div class="flex flex-row justify-between items-center">
                <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                    "Your Characters"
                </span>
                <span class="text-sm text-gray-400 font-medium">
                    {format!("{characters_len} / {}", user.max_characters)}
                </span>
            </div>

            <div class="w-full aspect-square flex flex-col p-2 gap-3 overflow-y-auto
            bg-neutral-900 ring-1 ring-neutral-950 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                <For
                    each=move || characters.clone()
                    key=|c| c.character_id
                    children=move |character| {
                        view! {
                            <CharacterSlot
                                character=character
                                areas=areas.clone()
                                refresh_trigger=refresh_trigger
                            />
                        }
                    }
                />

                {if characters_len < user.max_characters as usize {
                    Some(
                        view! {
                            <CreateCharacterSlot on:click=move |_| {
                                open_create_character.set(true)
                            } />
                        },
                    )
                } else {
                    None
                }}
            </div>

        </div>
    }
}

#[component]
fn CharacterSlot(
    character: UserCharacter,
    refresh_trigger: RwSignal<u64>,
    areas: Arc<HashMap<String, AreaSpecs>>,
) -> impl IntoView {
    let delete_character = Arc::new({
        let backend = expect_context::<BackendClient>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();
        let character_id = character.character_id;

        move || {
            spawn_local(async move {
                match backend
                    .delete_character(&auth_context.token(), &character_id)
                    .await
                {
                    Ok(_) => {
                        refresh_trigger.update(|n| *n += 1);
                        show_toast(
                            toaster,
                            "Character deleted".to_string(),
                            ToastVariant::Success,
                        );
                    }
                    Err(e) => {
                        show_toast(
                            toaster,
                            format!("Failed to delete character: {e}"),
                            ToastVariant::Error,
                        );
                    }
                }
            });
        }
    });

    let try_delete_character = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                "Deleting your character is irreversible and you will loose all items and progress, are you sure?".to_string(),
                delete_character.clone(),
            );
        }
    };

    let play_character = {
        let navigate = use_navigate();
        let (_, set_character_id_storage, _) =
            storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");
        let (_, set_area_id_storage, _) =
            storage::use_session_storage::<Option<String>, JsonSerdeCodec>("area_id");
        let character_activity = character.activity.clone();
        let accessibility: AccessibilityContext = expect_context();

        move |_| {
            accessibility.go_fullscreen();
            set_character_id_storage.set(character.character_id);
            match &character_activity {
                UserCharacterActivity::Rusting => navigate("/town", Default::default()),
                UserCharacterActivity::Grinding(area_id, _) => {
                    set_area_id_storage.set(Some(area_id.clone()));
                    navigate("/game", Default::default())
                }
            }
        }
    };

    view! {
        <div
            class="bg-neutral-800 rounded-xl border border-neutral-700 shadow-md
            flex flex-row items-stretch gap-4 p-3
            cursor-pointer hover:border-amber-400 hover:shadow-lg
            transition active:scale-95 active:border-amber-500"
            on:click=play_character.clone()
        >

            <div
                class="w-28 h-36 rounded-lg overflow-hidden flex-shrink-0"
                style=format!("background-image: url('{}');", img_asset("ui/paper_background.webp"))
            >
                <img
                    draggable="false"
                    src=img_asset(&character.portrait)
                    alt="Portrait"
                    class="object-cover w-full h-full"
                />
            </div>

            <div class="flex flex-col justify-between flex-grow overflow-hidden">
                <div class="space-y-1 overflow-hidden">
                    <div class="text-lg font-semibold text-shadow-md shadow-gray-950 text-amber-300 truncate">
                        {character.name.clone()}
                    </div>

                    <div class="text-sm text-gray-400 truncate">
                        {if character.max_area_level > 0 {
                            format!("Item Power Level: {}", character.max_area_level)
                        } else {
                            "Newbie".to_string()
                        }}
                    </div>

                    <div class="text-sm text-gray-400 truncate">
                        {match character.activity {
                            UserCharacterActivity::Rusting => view! { "Rusting in Town" }.into_any(),
                            UserCharacterActivity::Grinding(area_id, area_level) => {
                                view! {
                                    "Grinding: "
                                    {areas
                                        .get(&area_id)
                                        .map(|area_specs| area_specs.name.clone())
                                        .unwrap_or(area_id)}
                                    " ("
                                    {area_level}
                                    ")"
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </div>

                <div class="mt-2 flex gap-2">
                    <MenuButton class:flex-grow on:click=play_character.clone()>
                        "Play"
                    </MenuButton>
                    <MenuButton on:click=move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        try_delete_character(ev);
                    }>"❌"</MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
fn CreateCharacterSlot() -> impl IntoView {
    view! {
        <div class="bg-zinc-800 rounded-xl border border-zinc-700 shadow-md
        flex flex-row items-center gap-4 p-4 cursor-pointer
        hover:border-amber-400 hover:shadow-lg transition active:scale-95">

            <div class="h-12 w-12 flex items-center justify-center">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-12 w-12 text-amber-300"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="2"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
                </svg>
            </div>

            <span class="text-lg font-semibold text-amber-300">"Create Character"</span>
        </div>
    }
}

#[component]
pub fn CreateCharacterPanel(
    open: RwSignal<bool>,
    user_id: UserId,
    refresh_trigger: RwSignal<u64>,
) -> impl IntoView {
    let name = RwSignal::new(None);
    let selected_portrait = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let disable_submit =
        Signal::derive(move || name.read().is_none() || selected_portrait.read().is_none());

    let on_submit = {
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();
        let backend = expect_context::<BackendClient>();

        move |_| {
            if disable_submit.get() {
                return;
            }

            processing.set(true);
            spawn_local({
                async move {
                    match backend
                        .post_create_character(
                            &auth_context.token(),
                            &user_id,
                            &CreateCharacterRequest {
                                name: name.get().unwrap(),
                                portrait: selected_portrait.get().unwrap(),
                            },
                        )
                        .await
                    {
                        Ok(_) => {
                            open.set(false);
                            processing.set(false);
                            refresh_trigger.update(|n| *n += 1);
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Character creation error: {e}"),
                                ToastVariant::Error,
                            );
                            processing.set(false);
                        }
                    }
                }
            });
        }
    };

    let portraits = [
        "human_male_1",
        "human_male_2",
        "human_male_3",
        "human_female_1",
        "human_female_2",
        "human_female_3",
    ];

    view! {
        <MenuPanel open=open>
            <div class="flex items-center justify-center p-4 max-h-full">
                <div class="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl p-6 sm:p-8 space-y-8 w-full max-w-lg mx-auto">
                    <h2 class="text-2xl font-bold text-amber-300 text-center">
                        "Create Character"
                    </h2>

                    <ValidatedInput
                        id="name"
                        label="Character Name"
                        input_type="text"
                        bind=name
                        placeholder="Enter a name"
                    />

                    <div>
                        <span class="block text-sm font-medium text-gray-400 mb-2">
                            "Choose a Portrait"
                        </span>
                        <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                            <For
                                each=move || portraits
                                key=|src| src.to_string()
                                children=move |src| {
                                    let is_selected = Signal::derive(move || {
                                        selected_portrait
                                            .get()
                                            .map(|portrait| portrait.into_inner() == src)
                                            .unwrap_or_default()
                                    });
                                    view! {
                                        <div
                                            class="relative rounded-lg overflow-hidden border-2 cursor-pointer transition
                                            hover:scale-105"
                                            style=format!(
                                                "background-image: url('{}');",
                                                img_asset("ui/paper_background.webp"),
                                            )
                                            class:border-amber-400=move || is_selected.get()
                                            class:border-transparent=move || !is_selected.get()
                                            on:click=move |_| {
                                                selected_portrait.set(AssetName::try_new(src).ok());
                                            }
                                        >
                                            <img
                                                draggable="false"
                                                src=img_asset(&format!("adventurers/{src}.webp"))
                                                alt="Portrait"
                                                class="object-cover w-full h-28 sm:h-32"
                                            />
                                            {move || {
                                                is_selected
                                                    .get()
                                                    .then(|| {
                                                        view! {
                                                            <div class="absolute inset-0 bg-amber-400/20"></div>
                                                        }
                                                    })
                                            }}
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>

                    <div class="flex justify-around gap-3 pt-4 border-t border-zinc-700">
                        <MenuButtonRed on:click=move |_| {
                            open.set(false)
                        }>"Cancel"</MenuButtonRed>
                        <MenuButton on:click=on_submit disabled=disable_submit>
                            "Confirm"
                        </MenuButton>
                    </div>
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
fn DiscordInviteBanner() -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let auth = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let invite_url = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);

    let fetch_invite = move |_| {
        loading.set(true);

        spawn_local(async move {
            match backend.get_discord_invite(&auth.token()).await {
                Ok(resp) => invite_url.set(Some(format!("https://discord.gg/{}", resp.code))),
                Err(e) => {
                    show_toast(
                        toaster,
                        format!("Failed to get Discord invite: {e}"),
                        ToastVariant::Error,
                    );
                }
            }

            loading.set(false);
        });
    };

    view! {
        <div class="
        sticky left-0 top-0 z-20
        w-full px-4 py-2
        bg-slate-800/90 backdrop-blur
        border-b border-slate-700
        flex items-center justify-between
        text-sm
        ">
            <span class="text-slate-300">
                "Help shape the future of the game and join our community on Discord to give feedback, suggest new features, and access early information."
            </span>

            {move || match invite_url.get() {
                Some(url) => {
                    view! {
                        <a
                            href=url
                            target="_blank"
                            class="px-3 py-1 rounded bg-amber-500 text-black font-semibold hover:bg-amber-400 transition"
                        >
                            "Join"
                        </a>
                    }
                        .into_any()
                }
                None => {

                    view! {
                        <button
                            on:click=fetch_invite
                            disabled=loading
                            class="px-3 py-1 rounded bg-slate-700 text-slate-200 border border-slate-600 hover:bg-slate-600 transition disabled:opacity-50"
                        >
                            {move || if loading.get() { "..." } else { "Get Link" }}
                        </button>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn NewsPanel() -> impl IntoView {
    let news_data = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move { backend.get_news().await.unwrap_or_default().entries }
    });

    view! {
        <div class="flex flex-col bg-zinc-800 rounded-xl ring-1 ring-zinc-950 shadow-xl p-4 text-left space-y-4">
            <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                "News"
            </span>

            <div class="w-full aspect-square flex flex-col p-2 gap-3 overflow-y-auto
            bg-neutral-900 ring-1 ring-neutral-950 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                <Transition fallback=move || {
                    view! { <p class="text-gray-400">"Loading..."</p> }
                }>
                    {move || {
                        Suspend::new(async move {
                            let news = news_data.await;
                            view! {
                                <For
                                    each=move || news.clone()
                                    key=|c| c.timestamp
                                    children=move |news| {
                                        view! { <NewsCard news /> }
                                    }
                                />
                            }
                        })
                    }}
                </Transition>
            </div>
        </div>
    }
}

#[component]
fn NewsCard(news: NewsEntry) -> impl IntoView {
    let mut lines = news.content.lines();

    let title = lines.next().unwrap_or("").trim().to_string();

    let body = lines.collect::<Vec<_>>().join("\n");

    view! {
        <div class="bg-neutral-800 rounded-xl border border-neutral-700 shadow-lg
        p-4 flex flex-col gap-3">

            <div class="flex items-center justify-between">
                <span class="text-amber-300 font-semibold text-lg">{title}</span>

                <span class="text-xs text-gray-400">{format_datetime(news.timestamp)}</span>
            </div>

            <p class="text-gray-300 text-sm text-justify whitespace-pre-line leading-relaxed">
                {body}
            </p>
        </div>
    }
}
