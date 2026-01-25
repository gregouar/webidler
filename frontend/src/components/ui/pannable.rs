use leptos::{html::*, prelude::*, web_sys};

#[component]
pub fn Pannable(children: Children) -> impl IntoView {
    let offset = RwSignal::new((0.0, 0.0));
    let dragging = RwSignal::new(None::<(f64, f64)>);
    let zoom = RwSignal::new(0.5f64);

    let svg_ref = NodeRef::new();

    let screen_to_svg = move |x: f64, y: f64| -> (f64, f64) {
        let svg: web_sys::SvgElement = svg_ref.get().expect("SVG node should exist");

        let rect = svg.get_bounding_client_rect();
        let x = (x - rect.left()) * 1000.0 / rect.width() - 500.0;
        let y = (y - rect.top()) * 1000.0 / rect.height() - 500.0;
        (x, y)
    };

    // --- Mouse handling ---
    let on_mouse_down = {
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            dragging.set(Some(screen_to_svg(
                ev.client_x() as f64,
                ev.client_y() as f64,
            )));
        }
    };

    let handle = window_event_listener(leptos::ev::mouseup, move |_| dragging.set(None));
    on_cleanup(move || handle.remove());

    let handle = window_event_listener(leptos::ev::mousemove, {
        move |ev: web_sys::MouseEvent| {
            if let Some((last_x, last_y)) = dragging.get() {
                ev.stop_propagation();
                let (cur_x, cur_y) = screen_to_svg(ev.client_x() as f64, ev.client_y() as f64);
                let dx = cur_x - last_x;
                let dy = cur_y - last_y;
                offset.update(|(x, y)| {
                    *x += dx;
                    *y += dy;
                });
                dragging.set(Some((cur_x, cur_y)));
            }
        }
    });
    on_cleanup(move || handle.remove());

    let on_wheel = {
        move |ev: web_sys::WheelEvent| {
            ev.prevent_default();
            let zoom_factor = if ev.delta_y() < 0.0 { 1.1 } else { 0.9 };
            let old_zoom = zoom.get();
            let new_zoom = (old_zoom * zoom_factor).clamp(0.25, 2.0);

            let (x, y) = screen_to_svg(ev.client_x() as f64, ev.client_y() as f64);
            let (ox, oy) = offset.get();
            offset.set((
                x - (x - ox) * (new_zoom / old_zoom),
                y - (y - oy) * (new_zoom / old_zoom),
            ));

            zoom.set(new_zoom);
        }
    };

    // --- Touch handling ---
    let last_pinch_distance = RwSignal::new(None::<f64>);

    let on_touch_start = {
        move |ev: web_sys::TouchEvent| {
            if ev.touches().length() == 1 {
                let touch = ev.touches().item(0).unwrap();
                let (x, y) = screen_to_svg(touch.client_x() as f64, touch.client_y() as f64);
                dragging.set(Some((x, y)));
            } else if ev.touches().length() == 2 {
                // pinch start
                let t1 = ev.touches().item(0).unwrap();
                let t2 = ev.touches().item(1).unwrap();
                let dx = (t1.client_x() as f64 - t2.client_x() as f64).abs();
                let dy = (t1.client_y() as f64 - t2.client_y() as f64).abs();
                last_pinch_distance.set(Some((dx.powi(2) + dy.powi(2)).sqrt()));
            }
        }
    };

    let on_touch_move = {
        move |ev: web_sys::TouchEvent| {
            ev.prevent_default();
            if ev.touches().length() == 1 {
                if let Some((last_x, last_y)) = dragging.get() {
                    let touch = ev.touches().item(0).unwrap();
                    let (cur_x, cur_y) =
                        screen_to_svg(touch.client_x() as f64, touch.client_y() as f64);
                    offset.update(|(x, y)| {
                        *x += cur_x - last_x;
                        *y += cur_y - last_y;
                    });
                    dragging.set(Some((cur_x, cur_y)));
                }
            } else if ev.touches().length() == 2 {
                // pinch zoom
                let t1 = ev.touches().item(0).unwrap();
                let t2 = ev.touches().item(1).unwrap();
                let dx = (t1.client_x() as f64 - t2.client_x() as f64).abs();
                let dy = (t1.client_y() as f64 - t2.client_y() as f64).abs();
                let dist = (dx.powi(2) + dy.powi(2)).sqrt();

                if let Some(last) = last_pinch_distance.get() {
                    let factor = dist / last;
                    let old_zoom = zoom.get();
                    let new_zoom = (old_zoom * factor).clamp(0.25, 2.0);

                    let (x, y) = screen_to_svg(
                        (t1.client_x() + t2.client_x()) as f64 * 0.5,
                        (t1.client_y() + t2.client_y()) as f64 * 0.5,
                    );
                    let (ox, oy) = offset.get();
                    offset.set((
                        x - (x - ox) * (new_zoom / old_zoom),
                        y - (y - oy) * (new_zoom / old_zoom),
                    ));

                    zoom.set(new_zoom);
                }
                last_pinch_distance.set(Some(dist));
            }
        }
    };

    let on_touch_end = move |_ev: web_sys::TouchEvent| {
        dragging.set(None);
        last_pinch_distance.set(None);
    };

    let grid_size = Memo::new(move |_| {
        let z = zoom.get();
        if z < 0.5 {
            100
        } else if z < 1.0 {
            50
        } else {
            25
        }
    });

    view! {
        <div
            on:wheel=on_wheel
            on:mousedown=on_mouse_down
            on:touchstart=on_touch_start
            on:touchmove=on_touch_move
            on:touchend=on_touch_end
            class="flex items-center justify-center w-full h-full touch-none overflow-hidden"
        >
            <svg
                node_ref=svg_ref
                width="100%"
                height="100%"
                viewBox="-500 -500 1000 1000"
                preserveAspectRatio="xMidYMid meet"
            >
                <defs>
                    <radialGradient id="node-inner-gradient" cx="50%" cy="50%" r="50%">
                        <stop offset="20%" stop-color="black" stop-opacity=0 />
                        <stop offset="70%" stop-color="black" stop-opacity=0.5 />
                        <stop offset="100%" stop-color="black" stop-opacity=0.8 />
                    </radialGradient>

                    <pattern
                        id="grid"
                        width=move || grid_size.get()
                        height=move || grid_size.get()
                        patternUnits="userSpaceOnUse"
                    >
                        <path
                            d=move || {
                                let s = grid_size.get();
                                format!("M {s} 0 L 0 0 0 {s}")
                            }
                            fill="none"
                            stroke="#555"
                            stroke-width="1"
                        />
                    </pattern>
                </defs>
                <g
                    transform=move || {
                        let (x, y) = offset.get();
                        format!("translate({x},{y}) scale({})", zoom.get())
                    }
                    class="xl:drop-shadow-[0_2px_4px_black] will-change-transform"
                >
                    {children()}
                </g>
            </svg>
        </div>
    }
}
