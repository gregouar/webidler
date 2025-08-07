use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::data::user::{Character, User};

#[component]
pub fn UserDashboardPage(// user: RwSignal<User>,
    // characters: RwSignal<Vec<Character>>,
) -> impl IntoView {
    // Navigation
    let navigate = use_navigate();

    // Handlers
    let on_logout = move |_| {
        // Clear user session/token and navigate to login page
        // user.set(None);
        navigate("/", Default::default());
    };

    let on_request_deletion = move |_| {
        // Ideally opens confirmation modal
        // Or call backend to mark account for deletion
    };

    let on_play_character = move |character_id: String| {
        // Navigate to game with this character
        navigate(&format!("/game/{}", character_id), Default::default());
    };

    let user = RwSignal::new(User {
        user_id: "xxx".to_string(),
        username: "Username".to_string(),
        max_characters: 5,
    });
    let characters = RwSignal::new(vec![Character {
        character_id: "yyy".to_string(),
        name: "Name".to_string(),
        portrait: "adventurers/human_male_2.webp".to_string(),
        max_area_level: 3,
    }]);

    view! {
        <main class="max-w-4xl mx-auto p-6 space-y-8 text-white">
            <h1 class="text-3xl font-bold">"Welcome, " {move || user.read().username.clone()}</h1>

            <div class="flex justify-between items-center">
                <h2 class="text-xl font-semibold">"Your Characters"</h2>
                <div class="text-sm text-gray-400">
                    {move || {
                        format!(
                            "{} / {} characters",
                            characters.read().len(),
                            user.read().max_characters,
                        )
                    }}
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <For
                    each=move || characters.get()
                    key=|char| char.character_id.clone()
                    children=move |character| {
                        view! {
                            <div class="p-4 bg-gray-800 rounded-xl shadow-md space-y-2">
                                <img
                                    src=character.portrait
                                    alt="Avatar"
                                    class="w-full h-48 object-cover rounded"
                                />
                                <div class="text-lg font-semibold">{character.name.clone()}</div>
                                <div class="text-sm text-gray-400">
                                    {if character.status.in_quest {
                                        format!("On quest: {}", character.status.quest_name)
                                    } else {
                                        "Idle".into()
                                    }}
                                </div>
                                <div class="flex justify-between mt-2">
                                    <button
                                        class="bg-amber-500 hover:bg-amber-600 text-black px-4 py-1 rounded"
                                        on:click=move |_| {
                                            navigate_to_game_with(character.character_id.clone());
                                        }
                                    >
                                        "Play"
                                    </button>
                                    <button
                                        class="text-red-400 hover:text-red-600 text-sm"
                                        on:click=move |_| {}
                                    >
                                        // backend.delete_character(character.character_id.clone());
                                        // characters.refetch();
                                        "Delete"
                                    </button>
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            <div class="flex justify-end">
                <button
                    class="bg-green-600 hover:bg-green-700 text-white font-semibold px-4 py-2 rounded disabled:opacity-50"
                    on:click=move |_| {}
                    // spawn_local(create_character.clone());
                    disabled=Signal::derive(move || {
                        characters.read().len() >= user.read().max_characters
                    })
                >
                    "Create Character"
                </button>
            </div>

            <hr class="border-gray-700 my-6" />

            <div class="text-sm text-gray-400 space-x-4">
                <a href="/account" class="underline hover:text-gray-200">
                    "Account Settings"
                </a>
                <a href="/account/delete" class="underline text-red-400 hover:text-red-500">
                    "Request Account Deletion"
                </a>
            </div>
        </main>
    }
}
