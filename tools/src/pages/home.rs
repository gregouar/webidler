// use frontend::components::ui::progress_bars::CircularProgressBar;
use leptos::prelude::*;
use leptos_use::use_interval_fn;
use std::sync::Arc;

use crate::header::HeaderMenu;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        // <MenuButton on:click=navigate_to_leaderboard>"Leaderboard"</MenuButton>
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <HeaderMenu />
            "Hello There"
            <Card class="w-3xl">
                <div class="w-full grid grid-cols-5 gap-2">
                    {(0..20)
                        .map(|_| {
                            let trigger_reset_progress = RwSignal::new(false);
                            let reset_progress = Signal::derive(move || {
                                trigger_reset_progress.get()
                            });
                            let progress_value = predictive_cooldown(
                                Signal::derive(move || 1.0),
                                reset_progress,
                                Signal::derive(move || false),
                                0.0,
                            );
                            Effect::new(move || {
                                if progress_value.get() >= 1.0 {
                                    trigger_reset_progress.set(true)
                                } else {
                                    trigger_reset_progress.set(false)
                                }
                            });

                            view! {
                                <div class="flex flex-col gap-1">
                                    <SegmentedCircularProgressBar
                                        bar_color="oklch(55.5% 0.163 48.998)"
                                        value=progress_value
                                        reset=reset_progress
                                        bar_width=4
                                    >
                                        <img
                                            draggable="false"
                                            src="assets/images/skills/attack.svg"
                                            alt="attack"
                                            class="w-full h-full flex-no-shrink fill-current
                                            xl:drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert"
                                        />
                                    </SegmentedCircularProgressBar>

                                    <div class="flex justify-around">
                                        <Toggle toggle_callback=|_| {}>
                                            <span class="inline xl:hidden">"A"</span>
                                            <span class="hidden xl:inline font-variant:small-caps">
                                                "Auto"
                                            </span>
                                        </Toggle>
                                        <FancyButton>
                                            <span class="text-base xl:text-2xl">"+"</span>
                                        </FancyButton>
                                    </div>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </Card>
        </main>
    }
}

pub fn predictive_cooldown(
    remaining_time: Signal<f64>,
    reset: Signal<bool>,
    disabled: Signal<bool>,
    starting_value: f64,
) -> RwSignal<f64> {
    let progress_value = RwSignal::new(starting_value);
    let rate = RwSignal::new(0.0);

    Effect::new(move || {
        let remaining_time = remaining_time.get();
        if remaining_time > 0.0 {
            let remaining: f64 = (1.0f64 - progress_value.get_untracked()).clamp(0.0, 1.0);
            rate.set(remaining / remaining_time);
        }
    });

    Effect::new(move || {
        if reset.get() {
            progress_value.set(0.0);
        }
    });

    use_interval_fn(
        move || {
            let rate = rate.get_untracked();
            if !disabled.get_untracked() && rate > 0.0 {
                progress_value.update(|progress_value| {
                    if *progress_value < 1.2 {
                        *progress_value += rate * 0.1;
                    }
                    if remaining_time.get_untracked() == 0.0 && rate == 0.0 {
                        *progress_value = 1.0;
                    }
                });
            }
        },
        100,
    );

    progress_value
}

#[component]
pub fn Card(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = true)] gap: bool,
    #[prop(default = true)] pad: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "max-h-full flex flex-col relative
            bg-zinc-800 
            rounded-[6px] xl:rounded-[8px]
                 
            ring-1 ring-zinc-900/80
            shadow-[0_4px_6px_rgba(0,0,0,0.25),inset_1px_1px_1px_rgba(255,255,255,0.06),inset_-1px_-1px_1px_rgba(0,0,0,0.15)]
            {} {} {}",
            class.unwrap_or_default(),
            if gap { "gap-1 xl:gap-2" } else { "" },
            if pad { "p-1 xl:p-3" } else { "" },
        )>{children()}</div>
    }
}

#[component]
pub fn SegmentedCircularProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] value: Signal<f64>,
    bar_color: &'static str,
    bar_width: u8,
    // Instant reset
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(into,default = Signal::derive(|| false))] disabled: Signal<bool>,
    // Inside the circular bar
    children: Children,
) -> impl IntoView {
    const SEGMENT_COUNT: usize = 15;
    const BRASS_DARK: &str = "#473117";
    const BRASS_MID: &str = "#92703a";
    const BRASS_BRIGHT: &str = "#d6ab60";
    const SOCKET_BLACK: &str = "#040404";
    const SOCKET_GLOW: &str = "rgba(130, 130, 150, 0.14)";
    const SEGMENT_BASE: &str = "#2a160c";
    const SEGMENT_STROKE: &str = "rgba(96, 58, 24, 0.95)";
    const SEGMENT_HIGHLIGHT: &str = "rgba(255, 241, 214, 0.14)";
    let _ = bar_width;

    let reset_overlay_style = RwSignal::new("opacity: 0; pointer-events: none;".to_string());
    let reset_icon_animation = RwSignal::new("");
    let live_segment_progress = RwSignal::new(0.0);
    let reset_segment_progress = RwSignal::new(0.0);
    let reset_armed = RwSignal::new(false);
    let geometries = Arc::new(build_segment_geometries(SEGMENT_COUNT));
    let base_geometries = Arc::clone(&geometries);
    let ease_out = |t: f64| 1.0 - (1.0 - t).powf(2.0);
    let lit_opacity = move |progress: f64, index: usize| {
        let raw_fill = (progress - index as f64).clamp(0.0, 1.0);
        0.06 + 0.94 * ease_out(raw_fill)
    };
    let highlight_opacity = move |progress: f64, index: usize| {
        let raw_fill = (progress - index as f64).clamp(0.0, 1.0);
        0.02 + 0.52 * ease_out(raw_fill)
    };

    Effect::new(move |_| {
        live_segment_progress.set(value.get().clamp(0.0, 1.0) * SEGMENT_COUNT as f64);
    });

    Effect::new(move |_| {
        if reset.get() && !reset_armed.get() {
            reset_armed.set(true);

            if !disabled.get_untracked() {
                reset_segment_progress.set(live_segment_progress.get_untracked());
                reset_overlay_style
                    .set("opacity: 1; pointer-events: none; animation: circular-progress-bar-fade-out 0.5s linear; animation-fill-mode: both;".to_string());
                reset_icon_animation.set(
                    "animation: circular-progress-bar-glow 0.5s ease; animation-fill-mode: both;",
                );

                // Trick to reset animation by removing it when ended
                set_timeout(
                    move || {
                        reset_overlay_style.set("opacity: 0; pointer-events: none;".to_string());
                        reset_icon_animation.set("");
                    },
                    std::time::Duration::from_millis(500),
                );
            }
        } else if !reset.get() {
            reset_armed.set(false);
        }
    });

    let lit_ring = move |progress_signal: RwSignal<f64>, container_style: RwSignal<String>, animated: bool| {
        let geometries = Arc::clone(&geometries);
        view! {
            <div
                class="absolute inset-[5%] rounded-full will-change-opacity"
                style=move || container_style.get()
            >
                {geometries
                    .iter()
                    .enumerate()
                    .map(|(index, geometry)| {
                        let segment_path = geometry.segment_path.clone();
                        let highlight_path = geometry.highlight_path.clone();
                        view! {
                            <svg
                                class="absolute inset-0 h-full w-full overflow-visible"
                                viewBox="0 0 100 100"
                                aria-hidden="true"
                            >
                                <path
                                    d=segment_path
                                    fill=bar_color
                                    stroke="rgba(255, 214, 158, 0.55)"
                                    stroke-width="0.9"
                                    stroke-linejoin="round"
                                    class=if animated { "transition-opacity duration-150 ease-linear" } else { "" }
                                    style:opacity=move || format!("{:.3}", lit_opacity(progress_signal.get(), index))
                                />
                                <path
                                    d=highlight_path
                                    fill="rgba(255, 248, 230, 0.92)"
                                    class=if animated { "transition-opacity duration-150 ease-linear" } else { "" }
                                    style:opacity=move || format!("{:.3}", highlight_opacity(progress_signal.get(), index))
                                />
                            </svg>
                        }
                    })
                    .collect_view()}
            </div>
        }
    };
    let live_ring_style = RwSignal::new("opacity: 1;".to_string());

    view! {
        <div class="circular-progress-bar">
            <div
                class="relative w-full h-full aspect-square rounded-full overflow-hidden
                bg-black"
                style="contain: strict;"
            >
                <svg class="absolute inset-0 h-full w-full" viewBox="0 0 100 100" aria-hidden="true">
                    <circle cx="50" cy="50" r="48" fill="none" stroke=BRASS_DARK stroke-width="3.2" />
                    <circle cx="50" cy="50" r="46.4" fill="none" stroke=BRASS_MID stroke-width="1.6" />
                    <circle cx="50" cy="50" r="45.2" fill="none" stroke=BRASS_BRIGHT stroke-width="0.8" opacity="0.9" />
                    {base_geometries
                        .iter()
                        .map(|geometry| {
                            view! {
                                <path
                                    d=geometry.segment_path.clone()
                                    fill=SEGMENT_BASE
                                    stroke=SEGMENT_STROKE
                                    stroke-width="0.85"
                                    stroke-linejoin="round"
                                />
                                <path
                                    d=geometry.highlight_path.clone()
                                    fill=SEGMENT_HIGHLIGHT
                                    opacity="0.7"
                                />
                            }
                        })
                        .collect_view()}
                </svg>

                {lit_ring(live_segment_progress, live_ring_style, true)}
                {lit_ring(reset_segment_progress, reset_overlay_style, false)}

                <div
                    class="absolute rounded-full border border-black/80"
                    style=format!(
                        "inset: 18%;
                         background: radial-gradient(circle at 50% 46%, {SOCKET_GLOW} 0%, rgba(14,14,18,0.12) 28%, {SOCKET_BLACK} 76%);
                         box-shadow: inset 0 1px 2px rgba(255,255,255,0.03), inset 0 -8px 10px rgba(0,0,0,0.82);"
                    )
                ></div>

                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2
                    will-change-[filter,transform] transition-[filter,transform] duration-500"
                    style=reset_icon_animation
                    class:brightness-50=move || disabled.get()
                >
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[derive(Clone)]
struct SegmentGeometry {
    segment_path: String,
    highlight_path: String,
}

fn build_segment_geometries(count: usize) -> Vec<SegmentGeometry> {
    let outer_radius = 43.5;
    let inner_radius = 31.5;
    let highlight_outer = 42.0;
    let highlight_inner = 37.8;
    let sweep = 360.0 / count as f64;
    let gap = sweep * 0.24;

    (0..count)
        .map(|index| {
            let start = -90.0 + index as f64 * sweep + gap * 0.5;
            let end = -90.0 + (index + 1) as f64 * sweep - gap * 0.5;
            let segment_path = trapezoid_ring_path(inner_radius, outer_radius, start, end);

            let mid = (start + end) * 0.5;
            let highlight_path = trapezoid_ring_path(
                highlight_inner,
                highlight_outer,
                mid - (end - start) * 0.22,
                mid - (end - start) * 0.02,
            );

            SegmentGeometry {
                segment_path,
                highlight_path,
            }
        })
        .collect()
}

fn trapezoid_ring_path(inner_radius: f64, outer_radius: f64, start_deg: f64, end_deg: f64) -> String {
    let (outer_start_x, outer_start_y) = polar_to_cartesian(50.0, 50.0, outer_radius, start_deg);
    let (outer_end_x, outer_end_y) = polar_to_cartesian(50.0, 50.0, outer_radius, end_deg);
    let (inner_end_x, inner_end_y) = polar_to_cartesian(50.0, 50.0, inner_radius, end_deg);
    let (inner_start_x, inner_start_y) = polar_to_cartesian(50.0, 50.0, inner_radius, start_deg);

    format!(
        "M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} Z",
        outer_start_x,
        outer_start_y,
        outer_end_x,
        outer_end_y,
        inner_end_x,
        inner_end_y,
        inner_start_x,
        inner_start_y
    )
}

fn polar_to_cartesian(cx: f64, cy: f64, radius: f64, degrees: f64) -> (f64, f64) {
    let radians = degrees.to_radians();
    (cx + radius * radians.cos(), cy + radius * radians.sin())
}

#[component]
pub fn FancyButton(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="btn
            tracking-wide
            text-white font-extrabold text-shadow shadow-neutral-950
            px-2 xl:px-3 rounded shadow-md
            text-sm xl:text-base 
            border border-neutral-950
            bg-gradient-to-t from-zinc-900 to-zinc-800 
            overflow-hidden
            hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
            active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
            active:translate-y-[1px]
            disabled:from-zinc-700 disabled:to-zinc-600
            disabled:text-zinc-400
            disabled:opacity-60 disabled:shadow-none
            "
            disabled=disabled
        >
            // disabled=disabled
            {children()}
        </button>
    }
}

#[component]
pub fn Toggle(
    #[prop(default = false)] initial: bool,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    mut toggle_callback: impl FnMut(bool) + 'static,
    children: Children,
) -> impl IntoView {
    let checked: RwSignal<bool> = RwSignal::new(initial);
    let switch_value = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        let new_value = !checked.get();
        checked.set(new_value);
        toggle_callback(new_value);
    };

    let toggle_class = move || {
        if checked.get() {
            "shadow-md text-white"
            // "ring-2 ring-amber-600/20 shadow-md text-white "
        } else {
            "opacity-60 shadow-none text-zinc-400"
        }
    };

    view! {
        <button
            on:click=switch_value
            class=move || {
                format!(
                    "btn
                    tracking-wide
                    px-2 xl:px-3
                    text-sm xl:text-base 
                    font-extrabold text-shadow shadow-neutral-950
                    border border-neutral-950 rounded 
                    bg-gradient-to-t from-zinc-900 to-zinc-800 
                    hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700
                    active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950
                    active:translate-y-[1px]
                    disabled:from-zinc-700 disabled:to-zinc-600
                    disabled:text-zinc-400
                    disabled:opacity-60 disabled:shadow-none
                    transition-all duration-200
                    relative
                    group
                    {}
                    ",
                    toggle_class(),
                )
            }
            disabled=disabled
        >
            {children()}
        </button>
    }
}
