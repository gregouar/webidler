use leptos::{html::*, prelude::*, web_sys};

#[component]
pub fn Pannable(children: Children) -> impl IntoView {
    let offset = RwSignal::new((0.0, 0.0));
    let dragging = RwSignal::new(None::<(f64, f64)>);
    let zoom = RwSignal::new(0.75f64);

    let svg_ref = NodeRef::new();

    let screen_to_svg = move |ev: &web_sys::MouseEvent| -> (f64, f64) {
        let svg: web_sys::SvgElement = svg_ref.get().expect("SVG node should exist");

        let rect = svg.get_bounding_client_rect();
        let x = (ev.client_x() as f64 - rect.left()) * 1000.0 / rect.width() - 500.0;
        let y = (ev.client_y() as f64 - rect.top()) * 1000.0 / rect.height() - 500.0;
        (x, y)
    };

    let on_mouse_down = {
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            dragging.set(Some(screen_to_svg(&ev)));
        }
    };

    let handle = window_event_listener(leptos::ev::mouseup, move |_| dragging.set(None));
    on_cleanup(move || handle.remove());

    let handle = window_event_listener(leptos::ev::mousemove, {
        move |ev: web_sys::MouseEvent| {
            if let Some((last_x, last_y)) = dragging.get() {
                ev.stop_propagation();
                let (cur_x, cur_y) = screen_to_svg(&ev);
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

            let (x, y) = screen_to_svg(&ev);
            let (ox, oy) = offset.get();
            offset.set((
                x - (x - ox) * (new_zoom / old_zoom),
                y - (y - oy) * (new_zoom / old_zoom),
            ));

            zoom.set(new_zoom);
        }
    };

    view! {
        <div
            on:wheel=on_wheel
            on:mousedown=on_mouse_down
            class="flex items-center justify-center w-full h-full overflow-hidden bg-neutral-900"
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
                </defs>
                <g
                    transform=move || {
                        let (x, y) = offset.get();
                        format!("translate({x},{y}) scale({})", zoom.get())
                    }
                    filter="drop-shadow(0 2px 4px black)"
                >
                    {children()}
                </g>
            </svg>
        </div>
    }
}
