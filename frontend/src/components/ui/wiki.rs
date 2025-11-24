use leptos::{html::*, prelude::*};

use crate::components::ui::buttons::MenuButton;

#[component]
pub fn WikiButton() -> impl IntoView {
    view! {
        <a href="https://webidler.gitbook.io/wiki/" target="_blank" rel="noopener noreferrer">
            <MenuButton>"?"</MenuButton>
        </a>
    }
}
