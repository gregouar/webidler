use frontend::{assets::img_asset, components::ui::card::Card};
use leptos::prelude::*;

use crate::header::HeaderMenu;

const LOGO_COG_PATH: &str = "m438.582 296.069 53.302-14.694-.024-50.728-53.278-14.743c-4.76-21.901-13.445-42.325-25.185-60.608l27.323-48.157-35.904-35.855-48.078 27.3C338.4 86.818 317.975 78.16 296.053 73.347l-14.746-53.224H230.64l-14.77 53.224c-21.874 4.813-42.324 13.472-60.61 25.235l-48.13-27.298-35.88 35.883 27.3 48.077c-11.74 18.336-20.401 38.76-25.236 60.66l-53.198 14.744v50.727l53.198 14.694a186.28 186.28 0 0 0 25.235 60.658l-27.298 48.157 35.88 35.83 48.128-27.274a186.217 186.217 0 0 0 60.66 25.186l14.72 53.25 50.693-.024 14.72-53.225c21.923-4.813 42.348-13.47 60.686-25.212l48.127 27.327 35.805-35.883-27.273-48.155c11.714-18.31 20.4-38.708 25.185-60.635zM200.588 122.394h110.819l78.333 78.358v9.207h-55.162c-15.844-26.933-45.134-45.051-78.582-45.051-33.445 0-62.735 18.118-78.578 45.051H122.23v-9.207zm55.408 88.542c24.88 0 45.072 20.196 45.072 45.052 0 24.929-20.19 45.1-45.072 45.1-24.872 0-45.068-20.17-45.068-45.1 0-24.856 20.196-45.052 45.068-45.052zm55.411 179.05H200.588l-78.357-78.358v-9.612h55.106a91.66 91.66 0 0 0 14.253 18.417c17.206 17.206 40.082 26.681 64.407 26.681 24.333 0 47.208-9.476 64.415-26.683a91.636 91.636 0 0 0 14.25-18.415h55.079v9.612z";

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <HeaderMenu />
            <Logo />
            <Card>"Hello There"</Card>
        </main>
    }
}

#[component]
pub fn LogoCog() -> impl IntoView {
    view! {
        <svg
            class="h-full w-full overflow-visible"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="14 6 488 488"
            aria-hidden="true"
        >
            <defs>
                <clipPath id="logo-cog-clip">
                    <path d=LOGO_COG_PATH></path>
                </clipPath>
                <linearGradient
                    id="logo-cog-fill"
                    x1="256"
                    y1="36"
                    x2="256"
                    y2="476"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop offset="0%" stop-color="#644e40"></stop>
                    <stop offset="30%" stop-color="#422d25"></stop>
                    <stop offset="72%" stop-color="#2b211c"></stop>
                    <stop offset="100%" stop-color="#242424"></stop>
                </linearGradient>
                <linearGradient
                    id="logo-cog-rim"
                    x1="160"
                    y1="68"
                    x2="356"
                    y2="436"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop offset="0%" stop-color="#7f6649"></stop>
                    <stop offset="45%" stop-color="#2b1d15"></stop>
                    <stop offset="100%" stop-color="#050404"></stop>
                </linearGradient>
                <filter id="logo-cog-shadow" x="-35%" y="-35%" width="170%" height="170%">
                    <feDropShadow
                        dx="0"
                        dy="16"
                        stdDeviation="16"
                        flood-color="#000000"
                        flood-opacity="0.9"
                    ></feDropShadow>
                // <feDropShadow
                // dx="0"
                // dy="0"
                // stdDeviation="4"
                // flood-color="#cabe88"
                // flood-opacity="0.08"
                // ></feDropShadow>
                </filter>
            </defs>

            <g transform="translate(-1,0)" style="isolation: isolate;">
                <path
                    d=LOGO_COG_PATH
                    fill="#090605"
                    fill-opacity="0.72"
                    transform="translate(0,12)"
                ></path>
                <path
                    d=LOGO_COG_PATH
                    fill="url(#logo-cog-fill)"
                    stroke="#e0bb86"
                    stroke-opacity="0.08"
                    stroke-width="3.5"
                    filter="url(#logo-cog-shadow)"
                ></path>
                <image
                    href=img_asset("ui/dark_metal.webp")
                    x="20"
                    y="20"
                    width="472"
                    height="472"
                    preserveAspectRatio="xMidYMid slice"
                    clip-path="url(#logo-cog-clip)"
                    opacity="0.38"
                    style="mix-blend-mode: soft-light;"
                ></image>
                <path
                    d=LOGO_COG_PATH
                    fill="none"
                    stroke="url(#logo-cog-rim)"
                    stroke-opacity="0.24"
                    stroke-width="1.5"
                ></path>
            </g>
        </svg>
    }
}

#[component]
pub fn Logo() -> impl IntoView {
    view! {
        <div class="relative isolate flex flex-col items-center leading-none select-none py-4 xl:py-6">
            // <div class="pointer-events-none absolute inset-x-10 top-1/2 h-24 xl:h-32 -translate-y-1/2 rounded-full bg-[radial-gradient(circle,rgba(0,0,0,0.44),transparent_72%)] blur-xl"></div>
            <div class="pointer-events-none absolute left-1/2 top-1/2 h-[12rem] w-[12rem] xl:h-[17rem] xl:w-[17rem] -translate-x-1/2 -translate-y-1/2 opacity-[0.42]
            drop-shadow-[0_4px_18px_rgba(0,0,0,0.8)]">
                <LogoCog />
            </div>

            <div class="relative z-10 flex flex-col items-center">
                <LogoWord
                    text="GrinD"
                    class="text-[3.8rem] xl:text-[6rem] tracking-[0.06em]"
                    texture_size="110px 110px"
                    base_gradient="linear-gradient(180deg, rgba(255,251,236,0.99), rgba(245,224,168,0.99) 16%, rgba(217,159,72,0.98) 43%, rgba(134,78,34,0.99) 76%, rgba(58,30,12,0.99) 100%)"
                    highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.6), rgba(255,248,227,0.28) 17%, rgba(255,210,124,0.12) 40%, rgba(0,0,0,0.2) 100%)"
                    shadow="[text-shadow:0_1px_0_rgba(255,247,222,0.38),0_2px_0_rgba(116,80,38,0.88),0_8px_16px_rgba(0,0,0,0.78)]"
                />

                <LogoWord
                    text="to"
                    class="text-[1.15rem] xl:text-[1.9rem] -mt-2 xl:-mt-3 -mb-3 xl:-mb-6 tracking-[0.08em]"
                    texture_size="90px 90px"
                    base_gradient="linear-gradient(180deg, rgba(251,243,224,0.99), rgba(222,181,103,0.96) 38%, rgba(108,69,33,0.99) 100%)"
                    highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.42), rgba(255,246,221,0.15) 20%, rgba(0,0,0,0.18) 100%)"
                    shadow="[text-shadow:0_1px_0_rgba(255,244,214,0.25),0_2px_0_rgba(87,59,27,0.8),0_5px_12px_rgba(0,0,0,0.7)]"
                />

                <LogoWord
                    text="RusT"
                    class="text-[4.05rem] xl:text-[6.45rem] tracking-[0.05em]"
                    texture_size="120px 120px"
                    base_gradient="linear-gradient(180deg, rgba(255,246,198,0.99), rgba(240,190,100,0.98) 18%, rgba(206,112,48,0.97) 46%, rgba(110,49,18,0.99) 78%, rgba(43,17,8,0.99) 100%)"
                    highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.5), rgba(255,244,208,0.18) 18%, rgba(255,189,102,0.08) 42%, rgba(0,0,0,0.22) 100%)"
                    shadow="[text-shadow:0_1px_0_rgba(255,240,202,0.32),0_2px_0_rgba(106,59,25,0.88),0_9px_18px_rgba(0,0,0,0.8)]"
                />
            </div>
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
            "relative inline-grid place-items-center font-black [font-variant:small-caps] drop-shadow-[0_5px_3px_rgba(0,0,0,0.58)]
            {}",
            class,
        )>
            <span
                class=format!(
                    "col-start-1 row-start-1 inline-block whitespace-pre text-transparent bg-clip-text {}",
                    shadow,
                )
                style=format!(
                    "background-image: {}; -webkit-text-fill-color: transparent;",
                    base_gradient,
                )
            >
                {text}
            </span>
            <span
                class="col-start-1 row-start-1 inline-block whitespace-pre text-transparent bg-clip-text opacity-[0.8] mix-blend-hard-light"
                style=format!(
                    "background-image: url('{}'); background-size: {}; background-position: center; background-repeat: repeat; -webkit-text-fill-color: transparent;",
                    img_asset("ui/metal_rust.webp"),
                    texture_size,
                )
            >
                {text}
            </span>
            <span
                class=format!(
                    "col-start-1 row-start-1 inline-block whitespace-pre text-transparent bg-clip-text opacity-[0.45] {}",
                    shadow,
                )
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
