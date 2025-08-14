use std::{collections::HashMap, sync::Arc};

use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
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
        backend_client::BackendClient,
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
    let (get_jwt_storage, set_jwt_storage, _) =
        storage::use_local_storage::<String, JsonSerdeCodec>("jwt");

    let refresh_trigger = RwSignal::new(0u64);

    let async_data = LocalResource::new({
        let backend = use_context::<BackendClient>().unwrap();
        move || async move {
            let _ = refresh_trigger.read();

            let areas = backend
                .get_areas()
                .await
                .map(|r| r.areas)
                .unwrap_or_default();

            let user = backend
                .get_me(&get_jwt_storage.get())
                .await
                .map(|r| r.user)
                .ok();

            match user {
                Some(user) => {
                    let characters = backend
                        .get_user_characters(&get_jwt_storage.get(), &user.user_id)
                        .await
                        .map(|r| r.characters)
                        .unwrap_or_default();
                    Some((areas, user, characters))
                }
                None => None,
            }
        }
    });

    Effect::new({
        let navigate = use_navigate();
        move || {
            if async_data.get().map(|x| x.is_none()).unwrap_or_default() {
                navigate("/", Default::default());
            }
        }
    });

    let logout = {
        let navigate = use_navigate();
        move |_| {
            set_jwt_storage.set("".to_string());
            navigate("/", Default::default());
        }
    };

    let open_create_character = RwSignal::new(false);

    view! {
        <main class="my-0 mx-auto w-full max-w-6xl px-4 sm:px-8 text-center overflow-x-hidden flex flex-col  justify-around min-h-screen">
            <Transition fallback=move || {
                view! { <p class="text-gray-400">"Loading..."</p> }
            }>
                {move || {
                    let logout = logout.clone();
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

                            <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">
                                "Welcome, " {user.username}
                            </h1>

                            <div class="bg-zinc-800 rounded-xl ring-1 ring-zinc-950 shadow-inner px-6 py-6 sm:px-8 sm:py-8 text-left space-y-8">
                                <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
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

                                <div class="flex flex-nowrap gap-6 overflow-x-auto p-4 bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
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

                            <div class="flex flex-col items-center gap-2 text-sm text-gray-400">
                                <a
                                    href="/account"
                                    class="underline hover:text-amber-300 transition"
                                >
                                    "Account Settings"
                                </a>
                                <MenuButtonRed on:click=logout>"Logout"</MenuButtonRed>
                            </div>
                        }
                    })
                }}
            </Transition>
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
        let backend = use_context::<BackendClient>().unwrap();
        let (get_jwt_storage, _, _) = storage::use_local_storage::<String, JsonSerdeCodec>("jwt");
        let toaster = expect_context::<Toasts>();
        let character_id = character.character_id;

        move || {
            spawn_local(async move {
                match backend
                    .delete_character(&get_jwt_storage.get(), &character_id)
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
                            format!("Failed to delete character: {e:?}"),
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
        let character_activity = character.activity.clone();

        move |_| {
            set_character_id_storage.set(character.character_id);
            match character_activity {
                UserCharacterActivity::Rusting => navigate("/town", Default::default()),
                UserCharacterActivity::Grinding(_, _) => navigate("/game", Default::default()),
            }
        }
    };

    view! {
        <div
            class="bg-zinc-800 rounded-xl border border-zinc-700 shadow-md overflow-hidden
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
                            format!("Max Area Level: {}", character.max_area_level)
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
                    <MenuButton on:click=try_delete_character>"❌"</MenuButton>
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
        let (get_jwt_storage, _, _) = storage::use_local_storage::<String, JsonSerdeCodec>("jwt");
        let toaster = expect_context::<Toasts>();
        let backend = use_context::<BackendClient>().unwrap();

        move |_| {
            if disable_submit.get() {
                return;
            }

            processing.set(true);
            spawn_local({
                async move {
                    match backend
                        .post_create_character(
                            &get_jwt_storage.get(),
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
                                format!("Character creation error: {e:?}"),
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
            <div class="flex items-center justify-center p-4 min-h-screen">
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
                        <span class="block text-sm font-medium text-gray-300 mb-2">
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
                                                src=img_asset(&format!("adventurers/{}.webp", src))
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
