use leptos::{html::*, prelude::*};

use crate::header::HeaderMenu;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        // <MenuButton on:click=navigate_to_leaderboard>"Leaderboard"</MenuButton>
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <HeaderMenu />
            "Hello There"
        </main>
    }
}
