use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::data::user::UserCharacter;

use crate::{
    assets::img_asset,
    components::{
        backend_client::BackendClient,
        ui::{
            buttons::{MenuButton, MenuButtonRed},
            menu_panel::MenuPanel,
        },
    },
};

#[component]
pub fn UserDashboardPage() -> impl IntoView {
    let (get_jwt_storage, set_jwt_storage, _) =
        storage::use_local_storage::<String, JsonSerdeCodec>("jwt");

    let user_and_characters = LocalResource::new({
        let backend = use_context::<BackendClient>().unwrap();
        move || async move {
            let user = backend
                .get_me(&get_jwt_storage.get())
                .await
                .map(|r| r.user)
                .ok(); // TODO: better error

            match user {
                Some(user) => {
                    let characters = backend
                        .get_characters(&get_jwt_storage.get(), &user.user_id)
                        .await
                        .map(|r| r.characters)
                        .unwrap_or_default();

                    Some((user, characters))
                }
                None => None,
            }
        }
    });

    let logout = {
        let navigate = use_navigate();
        let set_jwt_storage = set_jwt_storage.clone();
        move |_| {
            set_jwt_storage.set("".to_string());
            navigate("/", Default::default());
        }
    };

    let open_create_character = RwSignal::new(false);

    // let navigate_to_game = {
    //     let navigate = use_navigate();
    //     let delete_session_infos = delete_session_infos.clone();
    //     move |_| {
    //         // TODO: give token to backend alongside
    //         delete_session_infos();
    //         set_user_id_storage.set(username.get_untracked());
    //         navigate("game", Default::default());
    //     }
    // };

    // Redirect if not authenticated
    Effect::new({
        let navigate = use_navigate();
        move || {
            if user_and_characters
                .get()
                .map(|x| x.is_none())
                .unwrap_or_default()
            {
                navigate("/", Default::default());
            }
        }
    });

    view! {
        <main class="my-0 mx-auto max-w-3xl px-4 py-8 flex flex-col gap-6 text-white text-center">
            <CreateCharacterPanel open=open_create_character.clone() />
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    let logout = logout.clone();
                    Suspend::new(async move {
                        let (user, characters) = user_and_characters.await.unwrap_or_default();
                        let characters_len = characters.len();
                        view! {
                            <h1 class="text-4xl font-extrabold text-amber-200 text-shadow-lg shadow-gray-950 tracking-tight">
                                "Welcome, " {user.username}
                            </h1>

                            <div class="bg-zinc-800 rounded-xl border border-zinc-700 shadow-inner px-6 py-6 text-left space-y-6">

                                <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
                                    <h2 class="text-2xl font-bold text-white">"Your Characters"</h2>
                                    <span class="text-sm text-gray-400 font-medium">
                                        {format!(
                                            "{} / {} characters",
                                            characters_len,
                                            user.max_characters,
                                        )}
                                    </span>
                                </div>

                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                    <For
                                        each=move || characters.clone()
                                        key=|c| c.character_id.clone()
                                        children=move |character| {
                                            view! { <CharacterSlot character=character /> }
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
            </Suspense>
        </main>
    }
}

#[component]
fn CharacterSlot(character: UserCharacter) -> impl IntoView {
    view! {
        <div
            class="bg-neutral-900 rounded-xl border border-zinc-700 shadow-md min-h-[16rem]
            cursor-pointer
            hover:border-amber-400 hover:shadow-lg transition group
            overflow-hidden flex flex-col"
            on:click=move |_| {}
        >
            <div
                class="h-full w-full "
                style=format!("background-image: url('{}');", img_asset("ui/paper_background.webp"))
            >
                <img
                    src=img_asset(&character.portrait)
                    alt="Portrait"
                    class="object-cover h-full w-full"
                />
            </div>
            <div class="p-4 flex flex-col gap-1">
                <div class="text-lg font-semibold text-shadow-md shadow-gray-950 text-amber-200">
                    {character.name.clone()}
                </div>
                <div class="text-sm text-gray-400">
                    "Grinding: Inn Basement - level 134"
                // Or "Rusting in Town"
                </div>
                <div class="mt-3 flex gap-2">
                    <MenuButton class:flex-grow on:click=move |_| {}>
                        "Play"
                    </MenuButton>
                    <MenuButton on:click=move |_| {}>"❌"</MenuButton>
                // TODO: confirm + delete
                </div>
            </div>
        </div>
    }
}

#[component]
fn CreateCharacterSlot() -> impl IntoView {
    view! {
        <div class="bg-neutral-900 rounded-xl border border-zinc-700 shadow-md min-h-[16rem]
        flex flex-col items-center justify-center cursor-pointer
        hover:border-amber-400 hover:shadow-lg transition group">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-12 w-12 text-amber-300 group-hover:scale-110 group-active:scale-90 transition-transform duration-200"
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
pub fn CreateCharacterPanel(open: RwSignal<bool>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let portraits = [
        "human_male_1",
        "human_female_1p",
        "elf_male_1",
        "elf_female_1",
        "dwarf_male_1",
        "dwarf_female_1",
    ];
    let selected_portrait = RwSignal::new(portraits[0].to_string());

    view! {
        <MenuPanel open=open>
            <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
                <div class="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl p-6 w-full max-w-lg space-y-6">
                    <h2 class="text-2xl font-bold text-amber-300 text-center">
                        "Create Character"
                    </h2>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1">
                            "Character Name"
                        </label>
                        <input
                            type="text"
                            bind:value=name
                            placeholder="Enter a name"
                            class="w-full px-4 py-2 rounded-lg border border-gray-700 bg-gray-800 text-white placeholder-gray-500
                            focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
                        />
                    </div>

                    <div>
                        <span class="block text-sm font-medium text-gray-300 mb-2">
                            "Choose a Portrait"
                        </span>
                        <div class="grid grid-cols-3 gap-3">
                            <For
                                each=move || portraits.clone()
                                key=|src| src.to_string()
                                children=move |src| {
                                    let is_selected = Signal::derive(move || {
                                        selected_portrait.get() == src
                                    });
                                    view! {
                                        <div
                                            class="relative rounded-lg overflow-hidden border-2 cursor-pointer transition
                                            hover:scale-105"
                                            class:border-amber-400=move || is_selected.get()
                                            class:border-transparent=move || !is_selected.get()
                                            on:click=move |_| {
                                                selected_portrait.set(src.to_string());
                                            }
                                        >
                                            <img
                                                src=img_asset(&format!("adventurers/{}.webp", src))
                                                alt="Portrait"
                                                class="object-cover w-full h-24"
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

                    <div class="flex justify-end gap-3 pt-4 border-t border-zinc-700">
                        <MenuButtonRed on:click=move |_| {}>"Cancel"</MenuButtonRed>
                        <MenuButton on:click=move |_| {}>"Confirm"</MenuButton>
                    </div>
                </div>
            </div>
        </MenuPanel>
    }
}
