use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*};
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_use::storage;

use shared::data::user::{Character, User};

use crate::{
    assets::img_asset,
    components::ui::buttons::{MenuButton, MenuButtonRed},
};

#[component]
pub fn UserDashboardPage() -> impl IntoView {
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

    // TODO: Split in components
    view! {
        <main class="my-0 mx-auto max-w-3xl px-4 py-8 flex flex-col gap-6 text-white text-center">

            <h1 class="text-4xl font-extrabold text-amber-200 text-shadow-lg shadow-gray-950 tracking-tight">
                "Welcome, " {move || user.read().username.clone()}
            </h1>

            <div class="bg-zinc-800 rounded-xl border border-zinc-700 shadow-inner px-6 py-6 text-left space-y-6">

                <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
                    <h2 class="text-2xl font-bold text-white">"Your Characters"</h2>
                    <span class="text-sm text-gray-400 font-medium">
                        {move || {
                            format!(
                                "{} / {} characters",
                                characters.read().len(),
                                user.read().max_characters,
                            )
                        }}
                    </span>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <For
                        each=move || characters.get()
                        key=|char| char.character_id.clone()
                        children=move |character| {
                            view! {
                                <div class="bg-neutral-900 rounded-xl border border-zinc-700 shadow-md overflow-hidden flex flex-col">
                                    <div
                                        class="h-full w-full"
                                        style=format!(
                                            "background-image: url('{}');",
                                            img_asset("ui/paper_background.webp"),
                                        )
                                    >
                                        <img
                                            src=img_asset(&character.portrait)
                                            alt="Portrait"
                                            class="object-cover h-full w-full"
                                        />
                                    </div>
                                    <div class="p-4 flex flex-col gap-1">
                                        <div class="text-lg font-semibold text-white">
                                            {character.name.clone()}
                                        </div>
                                        <div class="text-sm text-gray-400">
                                            "Grinding <<Inn Basement - level 134>>"
                                        // Or "Rusting in Town"
                                        </div>
                                        <div class="mt-3 flex justify-between gap-2">
                                            <MenuButton on:click=move |_| {}>"Play"</MenuButton>
                                            <MenuButtonRed on:click=move |_| {}>"Delete"</MenuButtonRed>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />

                    {move || {
                        if characters.read().len() < user.read().max_characters as usize {
                            Some(
                                view! {
                                    <div
                                        on:click=move |_| {}
                                        class="bg-neutral-900 rounded-xl border border-zinc-700 shadow-md min-h-[16rem]
                                        flex flex-col items-center justify-center cursor-pointer
                                        hover:border-amber-400 hover:shadow-lg transition group"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="h-12 w-12 text-amber-300 group-hover:scale-110 transition-transform duration-200"
                                            fill="none"
                                            viewBox="0 0 24 24"
                                            stroke="currentColor"
                                            stroke-width="2"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                d="M12 4v16m8-8H4"
                                            />
                                        </svg>
                                        <span class="mt-2 text-lg font-semibold text-amber-300 group-hover:text-amber-200 transition-colors">
                                            "Create Character"
                                        </span>
                                    </div>
                                },
                            )
                        } else {
                            None
                        }
                    }}
                </div>

            </div>

            <div class="flex flex-col items-center gap-2 text-sm text-gray-400">
                <a href="/account" class="underline hover:text-amber-300 transition">
                    "Account Settings"
                </a>

                <MenuButtonRed on:click=move |_| {}>"Logout"</MenuButtonRed>
            </div>

        </main>
    }
}
