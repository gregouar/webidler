use leptos::prelude::*;

use super::buttons::CloseButton;

#[derive(Clone, Copy, Default)]
pub enum TutorialPopupPosition {
    Above,
    AboveLeft,
    BelowLeft,
    #[default]
    BelowRight,
}

impl TutorialPopupPosition {
    fn popup_class(self) -> &'static str {
        match self {
            Self::Above => "bottom-full left-1/2 mb-3 -translate-x-1/2",
            Self::AboveLeft => "bottom-full left-0 mb-3",
            Self::BelowLeft => "left-0 top-full mt-3",
            Self::BelowRight => "right-0 top-full mt-3",
        }
    }

    fn arrow_class(self) -> &'static str {
        match self {
            Self::Above => {
                "-bottom-2 left-1/2 -translate-x-1/2 border-x-8 border-t-8 border-x-transparent border-t-amber-300/80"
            }
            Self::AboveLeft => {
                "-bottom-2 left-4 border-x-8 border-t-8 border-x-transparent border-t-amber-300/80"
            }
            Self::BelowLeft => {
                "-top-2 left-4 border-x-8 border-b-8 border-x-transparent border-b-amber-300/80"
            }
            Self::BelowRight => {
                "-top-2 right-4 border-x-8 border-b-8 border-x-transparent border-b-amber-300/80"
            }
        }
    }
}

#[component]
pub fn TutorialPopup(
    #[prop(into)] show: Signal<bool>,
    #[prop(default = TutorialPopupPosition::BelowRight)] position: TutorialPopupPosition,
    #[prop(optional)] class: Option<&'static str>,
    message: &'static str,
    children: Children,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let force_closed = RwSignal::new(false);

    Effect::new(move || {
        if show.get() && !force_closed.get_untracked() {
            open.set(true);
        } else {
            open.set(false);
        }
    });

    view! {
        <div class=format!(
            "relative {}",
            class.unwrap_or_default(),
        )>
            // on:click=move |_| {
            // open.set(false);
            // }
            {children()}
            <div
                role="status"
                class:hidden=move || !open.get()
                class=format!(
                    "absolute z-50 w-72 max-w-[calc(100vw-1rem)] rounded border border-amber-300/80 bg-zinc-900 px-3 py-2 pr-9 text-left text-xs font-normal normal-case tracking-normal text-zinc-100 shadow-xl xl:text-sm {}",
                    position.popup_class(),
                )
            >
                <span class="font-bold text-amber-300">"Tip: "</span>
                {message}
                <div class="absolute right-1 top-1 opacity-80 hover:opacity-100">
                    <CloseButton
                        attr:aria-label="Close tutorial"
                        attr:title="Close tutorial"
                        on:click=move |_| {
                            open.set(false);
                            force_closed.set(true);
                        }
                    />
                </div>
                <span class=format!("absolute h-0 w-0 {}", position.arrow_class()) />
            </div>
        </div>
    }
}
