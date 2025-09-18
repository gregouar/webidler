use leptos::{html::*, prelude::*};

use shared::constants::WAVES_PER_AREA_LEVEL;
use shared::messages::client::{GoBackLevelMessage, SetAutoProgressMessage};

use crate::assets::img_asset;
use crate::components::ui::progress_bars::VerticalProgressBar;
use crate::components::ui::tooltip::{StaticTooltip, StaticTooltipPosition};
use crate::components::websocket::WebsocketContext;

use super::GameContext;
use super::loot_queue::LootQueue;
use super::monsters_grid::MonstersGrid;
use super::player_card::PlayerCard;

#[component]
pub fn BattleScene() -> impl IntoView {
    view! {
        <div class="absolute inset-0 p-1 xl:p-4">
            <div class="relative w-full max-h-full flex justify-between gap-1 xl:gap-4 ">
                <PlayerCard />
                <div class="w-2/3 aspect-[12/8] flex flex-col shadow-xl/30 rounded-md overflow-hidden">
                    <BattleSceneHeader />
                    <div class="flex w-full flex-1 min-h-0
                    bg-stone-800 overflow-hidden shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                        <MonstersGrid />
                        <ThreatMeter />
                    </div>
                    <LootQueue />
                    <BattleSceneFooter />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn BattleSceneHeader() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let auto_icon = move || {
        if game_context.area_state.read().auto_progress {
            "⏸"
        } else {
            "▶"
        }
    };

    let go_back = {
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            conn.send(&GoBackLevelMessage { amount: 1 }.into());
            game_context.area_state.update(|area_state| {
                area_state.going_back = area_state.going_back.saturating_add(1);
            });
        }
    };

    let toggle_auto_progress = {
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            let auto_progress = !game_context.area_state.read_untracked().auto_progress;
            game_context.area_state.write().auto_progress = auto_progress;
            conn.send(
                &SetAutoProgressMessage {
                    value: auto_progress,
                }
                .into(),
            );
        }
    };

    let header_background = move || {
        format!(
            "background-image: url('{}');",
            img_asset(&game_context.area_specs.read().header_background)
        )
    };

    view! {
        <div
            class="h-8 xl:h-16 relative overflow-hidden w-full
            bg-center bg-repeat-x flex items-center justify-between px-4"
            style=header_background
        >
            // <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>

            <div class="w-12 flex justify-start">
                <button
                    class="btn text-2xl xl:text-4xl text-amber-300 font-bold drop-shadow-[0_0_6px_rgba(0,0,10,0.8)]
                    hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)] 
                    active:scale-90 active:brightness-125 transition"
                    title="Go Back One Level"
                    on:click=go_back
                >
                    "←"
                </button>
            </div>

            <div class="flex-1 text-center relative">
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>
                <p class="relative z-10 text-shadow/30 text-amber-200 text-lg xl:text-2xl font-bold">
                    <span class="[font-variant:small-caps]">
                        {move || game_context.area_specs.read().name.clone()}
                    </span>
                    " — "
                    {move || {
                        game_context
                            .area_state
                            .with(|area_state| {
                                area_state
                                    .area_level
                                    .saturating_sub(area_state.going_back)
                                    .max(game_context.area_specs.read().starting_level)
                            })
                    }}
                </p>
            </div>

            <div class="w-12 flex justify-end">
                <button
                    class="btn text-xl xl:text-3xl text-amber-300 font-bold drop-shadow-[0_0_6px_rgba(0,0,10,0.8)]
                    hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)] 
                    active:scale-90 active:brightness-125 transition"
                    title="Toggle Auto Progress"
                    on:click=toggle_auto_progress
                >
                    {auto_icon}
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn BattleSceneFooter() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let footer_background = move || {
        format!(
            "background-image: url('{}');",
            img_asset(&game_context.area_specs.read().footer_background)
        )
    };

    let wave_info = move || {
        if game_context.area_state.read().is_boss {
            "Boss".to_string()
        } else {
            format!(
                "Wave: {}/{}",
                game_context.area_state.read().waves_done,
                WAVES_PER_AREA_LEVEL,
            )
        }
    };

    let threat_level = move || game_context.area_threat.read().threat_level;

    view! {
        // h-8 xl:h-16
        <div
            class="h-8 xl:h-16 overflow-hidden z-10 w-full
            bg-center bg-repeat-x flex items-center justify-between"
            style=footer_background
        >
            <div class="relative px-4 py-2">
                <div class="absolute inset-0 blur-lg
                bg-gradient-to-r from-transparent via-zinc-950 via-[percentage:10%_90%] to-transparent
                "></div>
                <p class="relative text-shadow-md/30 shadow-gray-950 text-amber-200 text-base xl:text-2xl font-bold">
                    {wave_info}
                </p>
            </div>

            <div class="relative px-1 py-2">
                <div class="absolute inset-0 blur-lg
                bg-gradient-to-r from-transparent via-zinc-950 via-[percentage:10%_90%] to-transparent
                "></div>
                <div class="relative text-shadow-md/30 shadow-gray-950 text-amber-200 text-base xl:text-2xl font-bold
                flex items-center gap-1">
                    <span>{threat_level}</span>
                    <span class="text-yellow-500">
                        <ThreatIcon />
                    </span>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ThreatMeter() -> impl IntoView {
    let game_context: GameContext = expect_context();

    // TODO: predictive
    let value = Signal::derive(move || game_context.area_threat.read().elapsed_cooldown * 100.0);

    let threat_increase = Signal::derive(move || game_context.area_threat.read().just_increased);

    let time_remaining = Signal::derive(move || {
        (game_context.area_threat.read().cooldown > 0.0).then(|| {
            (1.0 - game_context.area_threat.read().elapsed_cooldown)
                * (game_context.area_threat.read().cooldown
                    / (game_context.player_specs.read().threat_gain * 0.01))
        })
    });

    view! {
        <StaticTooltip
            position=StaticTooltipPosition::Left
            tooltip=move || {
                time_remaining
                    .get()
                    .map(|time_remaining| {
                        format!("Time remaining before next Threat Level: {:.0}s", time_remaining)
                    })
                    .unwrap_or("No Threat".to_string())
            }
        >
            <div class="h-full py-1 pr-2 xl:pr-3 z-2">
                <VerticalProgressBar
                    class:z-2
                    class:w-4
                    class:xl:w-8
                    value
                    reset=threat_increase
                    bar_color="bg-gradient-to-l from-yellow-500 to-yellow-700"
                />
            </div>
        </StaticTooltip>
    }
}

#[component]
pub fn ThreatIcon() -> impl IntoView {
    view! {
        <StaticTooltip
            position=StaticTooltipPosition::Left
            tooltip=|| "Each Threat Level increases Enemies Power by 50%."
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-[1.3em] xl:h-[2em] aspect-square"
                fill="currentColor"
                viewBox="0 0 862.000000 1280.000000"
                stroke="currentColor"
                stroke-width="1"
            >
                <metadata>"Created by potrace 1.15, written by Peter Selinger 2001-2017"</metadata>
                <g transform="translate(0.000000,1280.000000) scale(0.100000,-0.100000)">
                    <path d="M4270 12770 c-118 -16 -314 -35 -435 -44 -965 -71 -1260 -141 -1790
                    -425 -381 -203 -634 -415 -1033 -865 -224 -251 -302 -349 -411 -511 -333 -497
                    -507 -1003 -583 -1695 -19 -174 -16 -706 6 -915 93 -921 234 -1332 563 -1638
                    l51 -48 17 58 c27 92 52 229 64 347 15 152 13 552 -4 921 -17 360 -19 632 -5
                    762 20 186 97 377 327 813 82 157 156 303 163 325 91 287 224 538 359 678 80
                    83 231 190 247 174 10 -9 -50 -149 -130 -302 -118 -227 -126 -248 -227 -617
                    -100 -367 -110 -392 -252 -617 -49 -79 -105 -178 -125 -219 -119 -258 -152
                    -667 -132 -1627 14 -688 9 -723 -102 -858 -48 -56 -131 -99 -288 -147 -195
                    -59 -282 -121 -339 -240 -25 -53 -26 -61 -26 -241 l0 -187 44 -83 c92 -178
                    207 -301 390 -421 184 -120 377 -206 896 -399 345 -128 426 -169 548 -276 86
                    -76 143 -149 250 -322 126 -203 243 -329 407 -438 152 -101 247 -145 445 -202
                    410 -119 1132 -165 1803 -115 l93 7 -6 -39 c-3 -21 -12 -98 -20 -171 -23 -220
                    7 -405 79 -489 33 -38 36 -39 98 -39 53 0 72 5 106 27 35 24 44 37 61 93 38
                    122 55 235 55 374 l1 133 -39 34 c-49 43 -99 61 -211 77 l-90 13 55 12 c30 6
                    125 24 211 41 190 37 219 46 386 128 215 106 292 177 518 473 154 203 228 285
                    333 370 109 88 172 118 458 220 511 182 742 297 936 467 113 99 259 279 302
                    373 57 125 77 319 42 420 -24 68 -99 146 -182 188 -32 16 -124 53 -204 82
                    -181 65 -250 101 -315 165 -67 65 -91 129 -105 277 -22 230 33 751 131 1237
                    l11 54 64 -64 64 -63 0 -265 c0 -222 5 -312 31 -570 17 -168 32 -308 34 -310
                    10 -10 158 37 228 71 175 88 298 256 366 498 45 164 57 259 100 790 52 635 56
                    707 55 1075 0 306 -3 386 -22 535 -68 555 -217 962 -519 1417 -239 360 -489
                    628 -848 909 -519 406 -1090 675 -1639 774 -91 17 -154 20 -377 20 -244 0
                    -285 3 -474 30 -114 17 -217 29 -229 29 -11 -1 -118 -14 -236 -29z m-1403
                    -4351 c230 -31 450 -127 600 -263 122 -110 220 -283 254 -451 44 -213 4 -497
                    -97 -683 -221 -409 -580 -718 -1024 -882 -210 -78 -378 -98 -532 -65 -222 46
                    -376 209 -441 462 -20 81 -22 110 -21 363 0 299 13 437 60 629 105 443 362
                    718 805 866 92 30 117 34 211 35 59 0 142 -5 185 -11z m3233 -15 c478 -89 816
                    -552 870 -1195 6 -68 22 -190 35 -270 29 -173 30 -192 9 -273 -32 -124 -106
                    -245 -216 -353 -91 -90 -158 -137 -283 -199 -144 -70 -266 -106 -365 -105
                    -526 4 -1237 790 -1305 1442 -16 149 64 354 201 516 163 193 392 329 729 429
                    120 37 167 38 325 8z m-1758 -1813 c65 -25 120 -121 139 -241 14 -94 34 -176
                    115 -469 108 -387 152 -590 173 -781 20 -190 21 -210 11 -210 -6 0 -147 25
                    -313 55 l-302 55 -251 -84 -252 -84 -7 72 c-23 248 86 750 255 1176 83 211
                    219 437 294 492 41 29 92 37 138 19z" />
                    <path d="M1364 4030 c-49 -32 -69 -89 -68 -195 1 -91 55 -1232 74 -1555 24
                    -430 90 -591 368 -903 209 -234 256 -294 390 -497 74 -112 162 -241 196 -285
                    137 -180 301 -324 459 -404 143 -71 452 -143 747 -173 159 -16 666 -16 820 1
                    67 7 231 12 391 11 290 -1 407 10 539 49 275 80 490 228 753 516 346 380 660
                    853 782 1180 l37 100 29 405 c15 223 33 434 40 470 6 36 36 162 66 280 69 274
                    80 335 93 500 12 158 13 410 2 410 -4 0 -23 -5 -42 -10 -19 -6 -111 -26 -204
                    -46 -254 -52 -358 -98 -473 -207 -108 -101 -194 -250 -252 -431 -44 -135 -62
                    -219 -116 -521 -55 -311 -83 -433 -132 -580 -91 -277 -225 -484 -388 -600
                    -183 -130 -467 -223 -785 -256 -147 -15 -704 -6 -890 15 -332 38 -563 96 -779
                    197 -283 133 -470 320 -610 609 -85 175 -100 266 -121 715 -6 121 -17 252 -24
                    291 -34 168 -95 318 -185 453 -63 95 -186 238 -220 255 -15 8 -76 49 -136 91
                    -151 106 -211 135 -277 135 -36 0 -63 -7 -84 -20z" />
                    <path d="M2977 3400 c-34 -10 -100 -59 -127 -93 -60 -75 -80 -178 -56 -281 48
                    -204 201 -354 339 -331 52 9 119 70 152 140 22 47 25 67 25 157 0 158 -40 358
                    -80 398 -17 17 -33 20 -122 19 -57 0 -115 -4 -131 -9z" />
                    <path d="M4010 3335 c-19 -9 -43 -24 -53 -33 -34 -30 -76 -128 -94 -219 l-18
                    -88 -6 65 c-8 80 -36 145 -85 200 -56 62 -104 83 -179 78 -69 -4 -123 -35
                    -149 -87 -26 -50 -46 -192 -53 -371 l-6 -177 54 -19 c74 -26 235 -26 347 0
                    l82 19 1 76 c1 67 2 72 10 38 20 -84 83 -169 137 -185 15 -4 65 -7 112 -7 99
                    0 149 21 199 82 55 67 64 114 58 307 -4 138 -9 184 -26 236 -32 100 -23 95
                    -168 98 -98 1 -136 -2 -163 -13z" />
                    <path d="M4495 3337 c-7 -22 -54 -369 -66 -486 -6 -63 -8 -134 -4 -158 6 -39
                    11 -46 42 -60 87 -36 276 -25 382 23 l54 24 -6 213 c-11 362 -32 401 -232 432
                    -164 25 -166 25 -170 12z" />
                    <path d="M2843 1979 c7 -152 34 -273 73 -332 35 -54 112 -87 212 -94 82 -6
                    151 7 177 33 46 47 -21 292 -108 391 -47 54 -131 90 -262 113 -33 6 -69 14
                    -79 17 -18 5 -19 0 -13 -128z" />
                    <path d="M3500 2067 c-3 -3 -45 -10 -93 -15 -49 -5 -91 -12 -94 -16 -3 -3 11
                    -90 31 -193 61 -315 135 -424 270 -399 107 20 207 232 192 410 -11 145 -74
                    205 -225 214 -42 2 -78 2 -81 -1z" />
                    <path d="M4847 2026 c-48 -18 -83 -49 -110 -101 -19 -34 -22 -56 -21 -140 1
                    -83 6 -116 32 -195 45 -138 91 -200 148 -200 60 0 217 152 267 258 52 111 39
                    209 -40 293 -73 78 -194 115 -276 85z" />
                    <path d="M3984 1939 c-91 -87 -132 -208 -113 -333 25 -153 126 -235 279 -224
                    85 6 121 28 170 101 62 93 83 158 84 262 1 81 -2 94 -26 131 -15 23 -40 51
                    -55 62 -43 31 -129 52 -210 52 l-74 0 -55 -51z" />
                </g>
            </svg>
        </StaticTooltip>
    }
}
