use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;

use frontend::components::ui::buttons::MenuButton;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let navigate_to_passives = {
        let navigate = use_navigate();
        move |_| {
            navigate("/passives", Default::default());
        }
    };

    view! {
        <div class="relative z-50 flex justify-between items-center p-1 xl:p-2
        bg-zinc-800 border-b-1 border-zinc-900/50 shadow-md/30 h-auto">
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <MenuButton on:click=navigate_to_passives>"Passives"</MenuButton>
            </div>
        </div>
    }
}
