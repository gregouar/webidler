use leptos::prelude::*;

use crate::assets::img_asset;

#[component]
pub fn BaseHeaderMenu(children: Children) -> impl IntoView {
    view! {
        <div
            class="relative z-50 flex justify-between items-center p-1 xl:p-2 h-auto
            border-b border-[#6c5734]/45 bg-zinc-800
            shadow-[0_8px_18px_rgba(0,0,0,0.45),inset_0_-1px_0_rgba(0,0,0,0.18)]"
            style=format!(
                "
                background-image:
                    linear-gradient(180deg, rgba(214, 165, 102, 0.04), rgba(0,0,0,0)),
                    url('{}');
                background-blend-mode: screen, multiply;
                ",
                img_asset("ui/dark_stone.webp"),
            )
        >
            {children()}
        </div>
    }
}
