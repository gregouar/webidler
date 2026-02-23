use leptos::{html::*, prelude::*};
use leptos_router::hooks::use_navigate;

#[component]
pub fn ALink(
    #[prop(optional, into)] href: Option<String>,
    #[prop(optional, default = true)] underline: bool,
    children: Children,
) -> impl IntoView {
    let navigate = {
        let navigate = use_navigate();
        move |_| {
            if let Some(href) = &href {
                navigate(href, Default::default());
            }
        }
    };

    view! {
        <button
            class="btn text-amber-300 hover:text-amber-200"
            class:underline=underline
            on:click=navigate
        >
            {children()}
        </button>
    }
}
