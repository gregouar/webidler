use leptos::{html::*, prelude::*};

use crate::components::ui::buttons::MenuButton;

#[component]
pub fn WikiButton() -> impl IntoView {
    view! {
        <a
            href="https://webidler.gitbook.io/webidler-docs/"
            target="_blank"
            rel="noopener noreferrer"
        >
            <MenuButton>"?"</MenuButton>
        </a>
    }
}
