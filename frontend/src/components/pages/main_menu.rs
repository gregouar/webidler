use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::{components::Redirect, hooks::use_navigate};
use leptos_use::storage;
use rand::{Rng, distr::Alphanumeric};

use shared::{
    http::client::{ForgotPasswordRequest, SignInRequest, SignUpRequest},
    types::{Password, Username},
};

use crate::{
    assets::{img_asset, screenshot_asset},
    components::{
        backend_client::BackendClient,
        captcha::*,
        events::keyboard_event_key,
        settings::SettingsContext,
        shared::{leaderboard::LeaderboardPanel, news::NewsInset, player_count::PlayerCount},
        ui::{
            ALink,
            buttons::MenuButton,
            card::{Card, CardInset, CardTitle},
            input::{Input, ValidatedInput},
            list_row::MenuListRow,
            toast::*,
        },
    },
};

#[component]
pub fn MainMenuPage() -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let refresh_token =
        LocalResource::new(move || async move { backend.get_access_token().await.is_ok() });

    view! {
        <Transition fallback=move || {
            view! { <p class="text-zinc-400">"Loading..."</p> }
        }>
            {move || Suspend::new(async move {
                if refresh_token.await {
                    view! { <Redirect path="/user-dashboard" /> }.into_any()
                } else {
                    view! { <MainMenu /> }.into_any()
                }
            })}
        </Transition>
    }
}

#[component]
fn MainMenu() -> impl IntoView {
    let (get_username_storage, set_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("username");

    let username = RwSignal::new(get_username_storage.get_untracked());
    let password = RwSignal::new(None);

    let (get_guest_username_storage, _, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_username");

    let (get_guest_password_storage, _, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_password");

    let captcha_token = RwSignal::new(None);

    let connecting = RwSignal::new(false);
    let disable_connect = Signal::derive(move || {
        username.read().is_none()
            || password.read().is_none()
            || captcha_token.read().is_none()
            || connecting.get()
    });

    let do_signin = {
        let toaster = expect_context::<Toasts>();
        let backend = expect_context::<BackendClient>();
        let navigate = use_navigate();

        move |username, password| {
            connecting.set(true);
            let navigate = navigate.clone();
            spawn_local({
                async move {
                    match backend
                        .post_signin(&SignInRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            username,
                            password,
                        })
                        .await
                    {
                        Ok(_) => {
                            navigate("/user-dashboard", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Authentication error: {e}"),
                                ToastVariant::Error,
                            );
                            connecting.set(false);
                        }
                    }
                }
            });
        }
    };

    let signin = {
        let do_signin = do_signin.clone();
        move || {
            do_signin(username.get().unwrap(), password.get().unwrap());
            set_username_storage.set(username.get());
        }
    };

    let disable_guest = Signal::derive(move || captcha_token.read().is_none() || connecting.get());
    let show_guest_modal = RwSignal::new(false);

    let guest_signin = move || match (
        get_guest_username_storage.get(),
        get_guest_password_storage.get(),
    ) {
        (Some(guest_username), Some(guest_password)) => {
            do_signin(guest_username, guest_password);
        }
        _ => show_guest_modal.set(true),
    };

    let navigate_to_signup = {
        let navigate = use_navigate();
        move |_| {
            navigate("/signup", Default::default());
        }
    };
    let password_ref = NodeRef::<leptos::html::Input>::new();

    let show_forgot_password_modal = RwSignal::new(false);
    let open_leaderboard = RwSignal::new(false);
    let selected_screenshot = RwSignal::new(None);

    view! {
        <main class="w-full max-w-[112rem] mx-auto px-2 pb-4">
            <div class="w-full max-w-3xl mx-auto text-center">
                <PlayerCount />
                <LeaderboardPanel open=open_leaderboard />
                <Logo />
            </div>

            <div class="2xl:grid 2xl:grid-cols-[minmax(18rem,1fr)_minmax(0,48rem)_minmax(18rem,1fr)] 2xl:items-start 2xl:gap-6">
                <ScreenshotColumn
                    screenshots=[
                        ("battle.jpg", "Battle", "Fight waves of enemies across endless grinds."),
                        (
                            "skills.jpg",
                            "Skills",
                            "Shape your build with skills and permanent upgrades.",
                        ),
                        (
                            "passives.jpg",
                            "Passive progression",
                            "Master a two-layer passives tree for in-run and persistent progression.",
                        ),
                    ]
                    selected=selected_screenshot
                />

                <div class="w-full max-w-3xl mx-auto text-center">

                    <Card>
                        <CardTitle>"Kill. Loot. Trade. Build. Repeat."</CardTitle>
                        <CardInset class="mb-3 text-left">
                            <div class="space-y-1.5">
                                <p class="text-sm leading-relaxed text-zinc-300">
                                    "Grind to Rust is a free, open-source browser incremental idle game inspired by Path of Exile. Discover powerful loot, master a deep item and crafting system, trade with other players, and theorycraft unique builds through multiple layers of long-term progression."
                                </p>
                            </div>
                        </CardInset>

                        <div class="text-left">
                            <CardTitle>"Sign In"</CardTitle>
                        </div>
                        <CardInset class="text-left">
                            <div class="grid grid-cols-1 gap-3 sm:grid-cols-[minmax(0,1fr)_minmax(18.75rem,1fr)] sm:items-start">
                                <div class="min-w-0">
                                    <div class="w-full mx-auto text-left">
                                        // <label class="block mb-2 text-sm font-medium text-zinc-400">
                                        // "Sign In:"
                                        // </label>
                                        <Input
                                            id="username"
                                            input_type="text"
                                            placeholder="Enter your username"
                                            bind=username
                                            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                                if keyboard_event_key(&ev).as_deref() == Some("Enter")
                                                    && let Some(pw) = password_ref.get()
                                                {
                                                    pw.focus().unwrap();
                                                }
                                            }
                                        />
                                    </div>
                                    <div class="w-full mx-auto text-left mt-2">
                                        <Input
                                            node_ref=password_ref
                                            id="password"
                                            input_type="password"
                                            placeholder="Enter your password"
                                            bind=password
                                            on:keydown={
                                                let signin = signin.clone();
                                                move |ev| {
                                                    if keyboard_event_key(&ev).as_deref() == Some("Enter") {
                                                        signin();
                                                    }
                                                }
                                            }
                                        />
                                    </div>
                                    <div class="text-right mt-1">
                                        <button
                                            class="btn text-amber-300 text-sm underline hover:text-amber-200"
                                            on:click=move |_| show_forgot_password_modal.set(true)
                                        >
                                            "I forgot my password"
                                        </button>
                                    </div>
                                </div>

                                <div class="min-w-0">
                                    <div class="flex min-h-[4.1rem] items-start justify-center overflow-hidden">
                                        <Captcha token=captcha_token />
                                    </div>
                                    <div class="mx-auto mt-2 flex w-[18.75rem] max-w-full flex-col">
                                        <MenuButton
                                            on:click=move |_| signin()
                                            disabled=disable_connect
                                            class="w-full"
                                        >
                                            {move || {
                                                if connecting.get() { "Connecting..." } else { "Connect" }
                                            }}
                                        </MenuButton>
                                    </div>
                                </div>
                            </div>
                        </CardInset>

                        <MenuButton
                            on:click=move |_| guest_signin()
                            disabled=disable_guest
                            class="mt-4 mb-4"
                        >
                            "Play as Guest"
                        </MenuButton>
                        <MenuButton on:click=navigate_to_signup>"Create Account"</MenuButton>
                        <MenuButton on:click=move |_| {
                            open_leaderboard.set(true)
                        }>"Leaderboard"</MenuButton>

                        <div class="mt-4 text-left">
                            <CardTitle>"News"</CardTitle>
                            <NewsInset class="w-full gap-3 max-h-80" />
                        </div>
                    </Card>
                    <ForgotPasswordModal open=show_forgot_password_modal />
                    <GuestModal open=show_guest_modal captcha_token />

                    <MenuListRow class="mt-2 px-4 py-3 text-left">
                        <div class="space-y-3">
                            <div class="space-y-1.5">
                                <h2 class="text-xs font-semibold uppercase tracking-[0.18em] text-amber-300/90">
                                    "Disclaimer"
                                </h2>
                                <p class="text-sm leading-relaxed text-zinc-300">
                                    "This game uses AI-generated dummy assets, which are currently being replaced by artwork from GDCA. The game is developed by Gregouar. Icons are from "
                                    <a
                                        href="https://game-icons.net"
                                        class="text-amber-300 underline hover:text-amber-200"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        "game-icons.net"
                                    </a>"."
                                </p>
                            </div>

                            <div class="flex justify-center gap-6 border-t border-[#5f4c30]/55 pt-2 text-sm">
                                <ALink href="/terms">"Terms & Conditions"</ALink>
                                <ALink href="/privacy">"Privacy Notice"</ALink>
                            </div>
                        </div>
                    </MenuListRow>
                </div>

                <ScreenshotColumn
                    screenshots=[
                        (
                            "unique_item.jpg",
                            "Unique items",
                            "Discover build-defining unique items.",
                        ),
                        ("forge.jpg", "Forge", "Craft, trade, and perfect powerful items."),
                        ("edicts.jpg", "Edicts", "Customize every grinds with dangerous edicts."),
                    ]
                    selected=selected_screenshot
                />
            </div>

            <ScreenshotLightbox selected=selected_screenshot />
        </main>
    }
}

#[component]
fn ScreenshotColumn(
    screenshots: [(&'static str, &'static str, &'static str); 3],
    selected: RwSignal<Option<&'static str>>,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let stone_texture = img_asset("ui/dark_stone.webp");

    view! {
        <aside class="hidden 2xl:flex flex-col gap-6 mt-16" aria-label="Game screenshots">
            {screenshots
                .into_iter()
                .map(|(file, title, description)| {
                    let stone_texture = stone_texture.clone();
                    view! {
                        <article class="group mx-auto w-[90%] max-w-sm">
                            <div
                                class="relative"
                                style=move || {
                                    if settings.uses_heavy_effects() {
                                        "filter: drop-shadow(0 10px 25px rgba(0,0,0,0.45));"
                                    } else {
                                        ""
                                    }
                                }
                            >
                                <div
                                    class="absolute inset-0 bg-black clip-octagon"
                                    aria-hidden="true"
                                ></div>

                                <div
                                    class=move || {
                                        format!(
                                            "clip-octagon absolute inset-0 border {}",
                                            if settings.uses_heavy_effects() {
                                                "border-[#6c5734]/45 shadow-[inset_2px_2px_1px_rgba(255,255,255,0.06),inset_-2px_-2px_1px_rgba(0,0,0,0.15)]"
                                            } else if settings.uses_surface_effects() {
                                                "border-[#665131]/50"
                                            } else {
                                                "border-[#5c4a2e]/60"
                                            },
                                        )
                                    }
                                    style=move || {
                                        if settings.uses_textures() {
                                            format!(
                                                "background-image: url('{}'); background-blend-mode: normal; background-color: rgb(39, 39, 42);",
                                                stone_texture,
                                            )
                                        } else {
                                            "background-image: linear-gradient(180deg, rgba(74,69,76,0.98), rgba(30,29,34,1)); background-color: rgb(39, 39, 42);"
                                                .to_string()
                                        }
                                    }
                                >
                                    <Show when=move || settings.uses_surface_effects()>
                                        <div class="pointer-events-none clip-octagon [--cut:11px] absolute inset-[1px] border border-[#d4b57a]/8"></div>
                                    </Show>
                                </div>

                                <button
                                    type="button"
                                    class="btn clip-octagon relative z-10 block w-full overflow-hidden p-[3px] transition-transform duration-300 active:scale-[0.98]"
                                    aria-label=format!("View {title} screenshot at full size")
                                    on:click=move |_| selected.set(Some(file))
                                >
                                    <div class="clip-octagon [--cut:9px] overflow-hidden">
                                        <img
                                            src=screenshot_asset(file)
                                            alt=format!("{title} gameplay screenshot")
                                            class="aspect-video w-full object-cover opacity-85
                                            saturate-[0.7] sepia-[0.2] brightness-90
                                            transition-[filter,transform] duration-300
                                            group-hover:grayscale-0 group-hover:sepia-0 group-hover:saturate-100 group-hover:brightness-100
                                            group-hover:scale-[1.01]"
                                        />
                                    </div>
                                </button>
                            </div>
                            <p class="px-2 pt-1 text-center text-sm leading-relaxed text-zinc-300">
                                {description}
                            </p>
                        </article>
                    }
                })
                .collect_view()}
        </aside>
    }
}

#[component]
fn ScreenshotLightbox(selected: RwSignal<Option<&'static str>>) -> impl IntoView {
    let lightbox_ref = NodeRef::<Div>::new();
    Effect::new(move |_| {
        if selected.get().is_some()
            && let Some(lightbox) = lightbox_ref.get()
        {
            let _ = lightbox.focus();
        }
    });

    view! {
        <style>
            "@keyframes screenshot-lightbox-fade-in {
                from { opacity: 0; }
                to { opacity: 1; }
            }
            @keyframes screenshot-lightbox-scale-in {
                from { opacity: 0; transform: scale(0.97); }
                to { opacity: 1; transform: scale(1); }
            }"
        </style>
        <Show when=move || selected.get().is_some()>
            <div
                node_ref=lightbox_ref
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 p-4 backdrop-blur-sm"
                style="animation: screenshot-lightbox-fade-in 160ms ease-out both;"
                role="dialog"
                aria-modal="true"
                aria-label="Full-size game screenshot"
                tabindex="0"
                on:click=move |_| selected.set(None)
                on:keydown=move |ev| {
                    if keyboard_event_key(&ev).as_deref() == Some("Escape") {
                        selected.set(None);
                    }
                }
            >
                <div
                    class="relative flex max-h-full max-w-full items-center justify-center"
                    style="animation: screenshot-lightbox-scale-in 180ms ease-out both;"
                >
                    <img
                        src=move || { selected.get().map(screenshot_asset).unwrap_or_default() }
                        alt="Full-size game screenshot"
                        class="max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)] object-contain shadow-2xl"
                        on:click=|ev| ev.stop_propagation()
                    />
                    <button
                        type="button"
                        class="btn absolute right-2 top-2 grid h-10 w-10 place-items-center rounded-full border border-zinc-500 bg-black/75 text-2xl text-white hover:border-amber-300 hover:text-amber-200"
                        aria-label="Close full-size screenshot"
                        on:click=move |_| selected.set(None)
                    >
                        "×"
                    </button>
                </div>
            </div>
        </Show>
    }
}

const LOGO_COG_PATH: &str = "m438.582 296.069 53.302-14.694-.024-50.728-53.278-14.743c-4.76-21.901-13.445-42.325-25.185-60.608l27.323-48.157-35.904-35.855-48.078 27.3C338.4 86.818 317.975 78.16 296.053 73.347l-14.746-53.224H230.64l-14.77 53.224c-21.874 4.813-42.324 13.472-60.61 25.235l-48.13-27.298-35.88 35.883 27.3 48.077c-11.74 18.336-20.401 38.76-25.236 60.66l-53.198 14.744v50.727l53.198 14.694a186.28 186.28 0 0 0 25.235 60.658l-27.298 48.157 35.88 35.83 48.128-27.274a186.217 186.217 0 0 0 60.66 25.186l14.72 53.25 50.693-.024 14.72-53.225c21.923-4.813 42.348-13.47 60.686-25.212l48.127 27.327 35.805-35.883-27.273-48.155c11.714-18.31 20.4-38.708 25.185-60.635zM200.588 122.394h110.819l78.333 78.358v9.207h-55.162c-15.844-26.933-45.134-45.051-78.582-45.051-33.445 0-62.735 18.118-78.578 45.051H122.23v-9.207zm55.408 88.542c24.88 0 45.072 20.196 45.072 45.052 0 24.929-20.19 45.1-45.072 45.1-24.872 0-45.068-20.17-45.068-45.1 0-24.856 20.196-45.052 45.068-45.052zm55.411 179.05H200.588l-78.357-78.358v-9.612h55.106a91.66 91.66 0 0 0 14.253 18.417c17.206 17.206 40.082 26.681 64.407 26.681 24.333 0 47.208-9.476 64.415-26.683a91.636 91.636 0 0 0 14.25-18.415h55.079v9.612z";

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

#[component]
pub fn ForgotPasswordModal(open: RwSignal<bool>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();

    let email = RwSignal::new(None);
    let captcha_token = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let success = RwSignal::new(false);

    let node_ref = NodeRef::new();
    let _ = leptos_use::on_click_outside(node_ref, move |_| open.set(false));

    let on_submit = {
        move |_| {
            if processing.get() {
                return;
            }

            processing.set(true);
            spawn_local({
                async move {
                    match backend
                        .post_forgot_password(&ForgotPasswordRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            email: email.get_untracked().unwrap(),
                        })
                        .await
                    {
                        Ok(_) => {
                            success.set(true);
                            show_toast(
                                toaster,
                                "Password reset instructions sent!",
                                ToastVariant::Success,
                            );
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Incorrect email: {e}"),
                                ToastVariant::Error,
                            );
                        }
                    }
                    processing.set(false);
                }
            });
        }
    };

    let disable_submit = Signal::derive(move || {
        email.read().is_none() || captcha_token.read().is_none() || processing.get()
    });

    let on_close = move |_| {
        success.set(false);
        open.set(false);
    };

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
                <div
                    node_ref=node_ref
                    class="bg-zinc-900 ring-1 ring-zinc-700 rounded-lg p-6 w-full max-w-md shadow-xl text-gray-200 space-y-4"
                >
                    <h2 class="text-xl font-bold text-amber-300 font-display">"Forgot Password"</h2>

                    <Show when=move || !success.get()>
                        <p class="text-zinc-400 text-sm leading-relaxed">
                            "Enter the email associated with your account. We'll send you a link to reset your password."
                        </p>

                        <ValidatedInput
                            label="Email"
                            id="email"
                            input_type="text"
                            placeholder="Enter your email for password recovery"
                            bind=email
                        />
                        <Captcha token=captcha_token />

                        <div class="flex justify-between gap-2 pt-2">
                            <MenuButton on:click=on_close>"Cancel"</MenuButton>
                            <MenuButton on:click=on_submit disabled=disable_submit>
                                {move || {
                                    if processing.get() { "Sending..." } else { "Send Reset Link" }
                                }}
                            </MenuButton>
                        </div>
                    </Show>

                    <Show when=move || success.get()>
                        <div class="text-center space-y-3">
                            <p class="text-zinc-400 text-sm">
                                "Check your email and follow the reset link to set a new password. Please check your spams, I don't have money to pay for a proper mail service."
                            </p>
                            <MenuButton on:click=on_close>"Close"</MenuButton>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn GuestModal(open: RwSignal<bool>, captcha_token: RwSignal<Option<String>>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();

    let (_, set_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("username");

    let (_, set_guest_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_username");

    let (_, set_guest_password_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("guest_password");

    let username = RwSignal::new({
        let mut rng = rand::rng();
        Username::try_new(format!(
            "Guest_{}",
            rng.random_range(111_111_111..999_999_999)
        ))
        .ok()
    });
    let password = RwSignal::new({
        let rng = rand::rng();
        let pwd: String = rng
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        Password::try_new(pwd).ok()
    });

    let accepted_terms = RwSignal::new(false);

    let processing = RwSignal::new(false);

    let node_ref = NodeRef::new();
    let _ = leptos_use::on_click_outside(node_ref, move |_| open.set(false));

    let on_submit = {
        let navigate = use_navigate();
        move || {
            if processing.get() {
                return;
            }
            processing.set(true);

            let navigate = navigate.clone();
            spawn_local({
                async move {
                    match backend
                        .post_signup(&SignUpRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            username: username.get_untracked().unwrap(),
                            email: None,
                            password: password.get_untracked().unwrap(),
                            accepted_terms: accepted_terms.get_untracked(),
                        })
                        .await
                    {
                        Ok(_) => {
                            set_guest_username_storage.set(username.get_untracked());
                            set_guest_password_storage.set(password.get_untracked());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Failed to register Guest User: {e}"),
                                ToastVariant::Error,
                            );
                            open.set(false);
                            return;
                        }
                    }

                    match backend
                        .post_signin(&SignInRequest {
                            captcha_token: captcha_token.get_untracked().unwrap_or_default(),
                            username: username.get_untracked().unwrap(), // TODO: better error?
                            password: password.get_untracked().unwrap(),
                        })
                        .await
                    {
                        Ok(_) => {
                            navigate("/user-dashboard", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Authentication error: {e}"),
                                ToastVariant::Error,
                            );
                        }
                    }

                    set_username_storage.set(username.get_untracked());
                    processing.set(false);
                }
            });
        }
    };

    let disable_submit = Signal::derive(move || {
        username.read().is_none()
            || !accepted_terms.get()
            || captcha_token.read().is_none()
            || processing.get()
    });

    view! {
        <Show when=move || {
            open.get()
        }>
            {
                let on_submit = on_submit.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
                        <div
                            node_ref=node_ref
                            class="bg-zinc-900 ring-1 ring-zinc-700 rounded-lg p-6 w-full max-w-md shadow-xl text-gray-200 space-y-4"
                        >
                            <h2 class="text-xl font-bold text-amber-300 font-display">"Guest"</h2>

                            <p class="text-zinc-400 text-sm leading-relaxed">
                                "Playing as guest will create a new account with random username and password stored locally in your browser.
                                You will be able to change them later from the 'Account Settings' page."
                            </p>

                            <ValidatedInput
                                id="username"
                                input_type="text"
                                label="Username:"
                                placeholder="Enter your username"
                                bind=username
                            />

                            <div class="flex items-start mt-4">
                                <input
                                    id="terms"
                                    type="checkbox"
                                    class="mt-1 mr-2"
                                    prop:checked=accepted_terms
                                    on:input=move |ev| {
                                        accepted_terms.set(event_target_checked(&ev));
                                    }
                                />
                                <label for="terms" class="text-sm text-zinc-400">
                                    "I agree to the "
                                    <span class="text-amber-300 underline hover:text-amber-200">
                                        <ALink href="/terms">"Terms & Conditions"</ALink>
                                    </span>
                                    " and I have read the "
                                    <span class="text-amber-300 underline hover:text-amber-200">
                                        <ALink href="/privacy">"Privacy Notice"</ALink>
                                    </span>
                                    "."
                                </label>
                            </div>

                            <div class="flex justify-between gap-2 pt-2">
                                <MenuButton on:click=move |_| {
                                    open.set(false);
                                }>"Cancel"</MenuButton>
                                <MenuButton on:click=move |_| on_submit() disabled=disable_submit>
                                    {move || {
                                        if processing.get() { "Connecting..." } else { "Play" }
                                    }}
                                </MenuButton>
                            </div>
                        </div>
                    </div>
                }
            }
        </Show>
    }
}
