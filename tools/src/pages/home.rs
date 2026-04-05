use frontend::assets::img_asset;
use leptos::prelude::*;

use crate::header::HeaderMenu;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <HeaderMenu />
            <Logo />
            "Hello There"
        </main>
    }
}

#[component]
pub fn Logo() -> impl IntoView {
    view! {
        <div class="relative flex flex-col items-center leading-none select-none py-2">
            <div class="pointer-events-none absolute inset-x-10 top-1/2 h-24 xl:h-32 -translate-y-1/2 rounded-full bg-[radial-gradient(circle,rgba(0,0,0,0.34),transparent_72%)] blur-xl"></div>

            <LogoWord
                text="GrinD"
                class="text-[3.8rem] xl:text-[6rem] tracking-[0.04em]"
                texture_size="110px 110px"
                base_gradient="linear-gradient(180deg, rgba(255,240,204,0.98), rgba(214,165,82,0.96) 34%, rgba(106,64,22,0.98) 82%)"
                highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.28), rgba(255,255,255,0.06) 18%, rgba(0,0,0,0.18) 100%)"
                shadow="[text-shadow:0_1px_0_rgba(255,231,183,0.22),0_2px_0_rgba(92,67,28,0.75),0_5px_8px_rgba(0,0,0,0.62)]"
            />

            <LogoWord
                text="to"
                class="text-[1.2rem] xl:text-[2rem] -mt-1 xl:-mt-2 -mb-3 xl:-mb-6 tracking-[0.18em]"
                texture_size="90px 90px"
                base_gradient="linear-gradient(180deg, rgba(248,234,194,0.98), rgba(205,156,76,0.95) 45%, rgba(104,73,28,0.98))"
                highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.24), rgba(255,255,255,0.05) 18%, rgba(0,0,0,0.16) 100%)"
                shadow="[text-shadow:0_1px_0_rgba(255,239,201,0.18),0_2px_0_rgba(88,65,30,0.68),0_4px_6px_rgba(0,0,0,0.58)]"
            />

            <LogoWord
                text="RusT"
                class="text-[4rem] xl:text-[6.4rem] tracking-[0.04em]"
                texture_size="120px 120px"
                base_gradient="linear-gradient(180deg, rgba(255,224,150,0.98), rgba(206,114,42,0.95) 38%, rgba(86,39,10,0.98) 84%)"
                highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.24), rgba(255,255,255,0.05) 18%, rgba(0,0,0,0.18) 100%)"
                shadow="[text-shadow:0_1px_0_rgba(255,235,188,0.24),0_2px_0_rgba(94,57,22,0.78),0_6px_10px_rgba(0,0,0,0.7)]"
            />
        </div>
    }
}

#[component]
fn LogoWord(
    text: &'static str,
    class: &'static str,
    texture_size: &'static str,
    base_gradient: &'static str,
    highlight_gradient: &'static str,
    shadow: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!(
            "relative inline-grid place-items-center font-extrabold [font-variant:small-caps] {} drop-shadow-[0_5px_3px_rgba(0,0,0,0.58)] {}",
            class,
            shadow,
        )>
            <span
                class="col-start-1 row-start-1 inline-block whitespace-pre text-transparent bg-clip-text"
                style=format!(
                    "background-image: {}; -webkit-text-fill-color: transparent;",
                    base_gradient,
                )
            >
                {text}
            </span>
            <span
                class="col-start-1 row-start-1 inline-block whitespace-pre text-transparent bg-clip-text mix-blend-hard-light"
                style=format!(
                    "background-image: url('{}'); background-size: {}; background-position: center; background-repeat: repeat; filter: contrast(1.55) brightness(1.12); -webkit-text-fill-color: transparent;",
                    img_asset("ui/metal_rust.webp"),
                    texture_size,
                )
            >
                {text}
            </span>
            <span
                class="col-start-1 row-start-1 inline-block whitespace-pre text-transparent bg-clip-text opacity-[0.45]"
                style=format!(
                    "background-image: {}; -webkit-text-fill-color: transparent;",
                    highlight_gradient,
                )
            >
                {text}
            </span>
        </span>
    }
}
