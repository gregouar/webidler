use std::{collections::HashMap, sync::Arc};

use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local, web_sys};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::{
    data::{
        area::AreaSpecs,
        user::{UserCharacter, UserCharacterActivity, UserCharacterId, UserId},
    },
    http::client::CreateCharacterRequest,
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

    let open_create_character = RwSignal::new(false);

    view! {
        <main class="my-0 mx-auto w-full min-h-screen text-center overflow-x-hidden flex flex-col justify-center">
            <DiscordInviteBanner />
            <PlayerCount />

            <div class="h-full max-w-6xl w-full mx-auto px-4 xl:px-8 text-center overflow-x-hidden flex flex-col justify-around">

                <Transition fallback=move || {
                    view! { <p class="text-gray-400">"Loading..."</p> }
                }>
                    {move || {
                        let sign_out = sign_out.clone();
                        Suspend::new(async move {
                            let (areas, user, characters) = async_data.await.unwrap_or_default();
                            let areas = Arc::new(areas);
                            let characters_len = characters.len();

                            view! {
                                <CreateCharacterPanel
                                    open=open_create_character
                                    user_id=user.user_id
                                    refresh_trigger=refresh_trigger
                                />

                                <h1 class="text-shadow-lg shadow-gray-950 text-amber-200 text-4xl  md:text-5xl xl:text-6xl font-extrabold leading-none tracking-tight">
                                    "Welcome, " {user.username}
                                </h1>

                                <div class="bg-zinc-800 rounded-xl ring-1 ring-zinc-950 shadow-inner p-2 xl:p-4 text-left space-y-2 xl:space-y-4">
                                    <div class="flex flex-row justify-between items-center gap-1 xl:gap-2">
                                        // <h2 class="text-2xl font-bold text-white">"Your Characters"</h2>
                                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                                            "Your Characters"
                                        </span>

                                        <span class="text-sm text-gray-400 font-medium">
                                            {format!(
                                                "{} / {} characters",
                                                characters_len,
                                                user.max_characters,
                                            )}
                                        </span>
                                    </div>

                                    <div class="flex flex-nowrap gap-6 overflow-x-auto p-4 bg-neutral-900 ring-1 ring-neutral-950 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
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
                                    <div class="flex items-center justify-between gap-2 text-gray-400">
                                        <a href="leaderboard">
                                            <MenuButton>"Leaderboard"</MenuButton>
                                        </a>
                                        <a href="account">
                                            <MenuButton>"Account Settings"</MenuButton>
                                        </a>
                                        <MenuButtonRed on:click=move |_| sign_out()>
                                            "Sign Out"
                                        </MenuButtonRed>
                                    </div>
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
            class="bg-neutral-800 rounded-xl border border-neutral-700 shadow-md overflow-hidden
            flex flex-col 
            min-h-[20rem] w-40 sm:w-48 md:w-56 flex-shrink-0
            transition group cursor-pointer
            hover:border-amber-400 hover:shadow-lg active:scale-95 active:border-amber-500"
            on:click=play_character.clone()
        >
            <div
                class="aspect-[3/4] w-full relative"
                style=format!("background-image: url('{}');", img_asset("ui/paper_background.webp"))
            >
                <img
                    draggable="false"
                    src=img_asset(&character.portrait)
                    alt="Portrait"
                    class="object-cover w-full h-full"
                />
            </div>

            <div class="p-4 flex flex-col flex-grow justify-between">
                <div class="space-y-1">
                    <div class="text-lg font-semibold text-shadow-md shadow-gray-950 text-amber-200 truncate">
                        {character.name.clone()}
                    </div>

                    <div class="text-sm text-gray-400 truncate">
                        {if character.max_area_level > 0 {
                            format!("Max Item Level: {}", character.max_area_level)
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
                                    <br />
                                    "Level reached: "
                                    {area_level}
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </div>

                <div class="mt-4 flex gap-2">
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
        <div class="bg-zinc-800 rounded-xl border border-zinc-700 shadow-md min-h-[20rem]
        w-40 sm:w-48 md:w-56 flex-shrink-0
        flex flex-col items-center justify-center cursor-pointer
        hover:border-amber-400 hover:shadow-lg active:scale-95 active:border-amber-500 transition group">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-12 w-12 text-amber-300 group-hover:scale-110 transition-transform"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width="2"
            >
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
            </svg>
            <span class="mt-2 text-lg font-semibold text-amber-300 group-hover:text-amber-200 transition-colors">
                "Create Character"
            </span>
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

// #[component]
// fn DiscordInviteBanner() -> impl IntoView {
//     let backend = expect_context::<BackendClient>();
//     let auth = expect_context::<AuthContext>();
//     let toaster = expect_context::<Toasts>();

//     let invite_url = RwSignal::new(None::<String>);
//     let loading = RwSignal::new(false);

//     let fetch_invite = move |_| {
//         loading.set(true);

//         spawn_local(async move {
//             // match backend.get_discord_invite(&auth.token()).await {
//             //     Ok(resp) => {
//             //         invite_url.set(Some(resp.invite));
//             //     }
//             //     Err(err) => {
//             //         show_toast(
//             //             toaster,
//             //             format!("Failed to load Discord invite: {err}"),
//             //             ToastVariant::Error,
//             //         );
//             //     }
//             // }

//             loading.set(false);
//         });
//     };

//     view! {
//         <div class="w-full bg-amber-900/20 border border-amber-700 rounded-xl p-4 my-6 text-left">
//             <h2 class="text-amber-300 font-semibold text-lg">
//                 "Help shape the future of the game!"
//             </h2>
//             <p class="text-gray-300 text-sm mt-1">
//                 "Join our community on Discord to give feedback, suggest new features, and access early information."
//             </p>

//             {move || match invite_url.get() {
//                 Some(url) => {
//                     view! {
//                         <a
//                             href=url
//                             target="_blank"
//                             class="inline-block mt-3 px-4 py-2 bg-amber-500 text-black font-bold rounded-lg hover:bg-amber-400 transition"
//                         >
//                             "Join Discord"
//                         </a>
//                     }
//                         .into_any()
//                 }
//                 None => {

//                     view! {
//                         <button
//                             class="inline-block mt-3 px-4 py-2 bg-amber-500 text-black font-bold rounded-lg hover:bg-amber-400 transition disabled:opacity-50"
//                             on:click=fetch_invite
//                             disabled=loading
//                         >
//                             {move || if loading.get() { "Loading..." } else { "Get Invite" }}
//                         </button>
//                     }
//                         .into_any()
//                 }
//             }}
//         </div>
//     }
// }
