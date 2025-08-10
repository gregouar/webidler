use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::data::user::UserCharacter;

use crate::{
    assets::img_asset,
    components::{
        backend_client::BackendClient,
        ui::buttons::{MenuButton, MenuButtonRed},
    },
};

#[component]
pub fn UserDashboardPage() -> impl IntoView {
    let (get_jwt_storage, set_jwt_storage, _) =
        storage::use_local_storage::<String, JsonSerdeCodec>("jwt");

    let user = LocalResource::new({
        let backend = use_context::<BackendClient>().unwrap();
        move || async move {
            backend
                .get_me(&get_jwt_storage.get())
                .await
                .map(|r| r.user)
                .unwrap_or_default()
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

    // let characters = RwSignal::new(vec![UserCharacter {
    //     character_id: "yyy".to_string(),
    //     name: "Name".to_string(),
    //     portrait: "adventurers/human_male_2.webp".to_string(),
    //     max_area_level: 3,
    // }]);
    let characters: RwSignal<Vec<UserCharacter>> = RwSignal::new(vec![]);

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
            if user
                .get()
                .map(|user| user.username.is_empty())
                .unwrap_or_default()
            {
                navigate("/", Default::default());
            }
        }
    });

    view! {
        <main class="my-0 mx-auto max-w-3xl px-4 py-8 flex flex-col gap-6 text-white text-center">
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    let logout = logout.clone();
                    Suspend::new(async move {
                        let user = user.await;
                        view! {
                            <h1 class="text-4xl font-extrabold text-amber-200 text-shadow-lg shadow-gray-950 tracking-tight">
                                "Welcome, " {user.username}
                            </h1>

                            <div class="bg-zinc-800 rounded-xl border border-zinc-700 shadow-inner px-6 py-6 text-left space-y-6">

                                <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
                                    <h2 class="text-2xl font-bold text-white">"Your Characters"</h2>
                                    <span class="text-sm text-gray-400 font-medium">
                                        {move || {
                                            format!(
                                                "{} / {} characters",
                                                characters.read().len(),
                                                user.max_characters,
                                            )
                                        }}
                                    </span>
                                </div>

                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                    <For
                                        each=move || characters.get()
                                        key=|c| c.character_id.clone()
                                        children=move |character| {
                                            view! { <CharacterSlot character=character /> }
                                        }
                                    />

                                    {move || {
                                        if characters.read().len() < user.max_characters as usize {
                                            Some(view! { <CreateCharacterSlot /> })
                                        } else {
                                            None
                                        }
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
        <div
            on:click=move |_| {}
            class="bg-neutral-900 rounded-xl border border-zinc-700 shadow-md min-h-[16rem]
            flex flex-col items-center justify-center cursor-pointer
            hover:border-amber-400 hover:shadow-lg transition group"
        >
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
