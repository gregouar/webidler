use leptos::{html::*, prelude::*, web_sys};

#[component]
pub fn Pannable(children: Children) -> impl IntoView {
    let offset = RwSignal::new((0.0, 0.0));
    let dragging = RwSignal::new(None::<(f64, f64)>);
    let zoom = RwSignal::new(1.0f64);

    let on_mouse_down = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        dragging.set(Some((ev.client_x() as f64, ev.client_y() as f64)));
    };

    let handle = window_event_listener(leptos::ev::mouseup, {
        move |_| {
            dragging.set(None);
        }
    });
    on_cleanup(move || handle.remove());

    let handle = window_event_listener(leptos::ev::mousemove, {
        move |ev: web_sys::MouseEvent| {
            if let Some((last_x, last_y)) = dragging.get() {
                let dx = ev.client_x() as f64 - last_x;
                let dy = ev.client_y() as f64 - last_y;
                offset.update(|(x, y)| {
                    *x += dx;
                    *y += dy;
                });
                dragging.set(Some((ev.client_x() as f64, ev.client_y() as f64)));
            }
        }
    });
    on_cleanup(move || handle.remove());

    let on_wheel = {
        move |ev: web_sys::WheelEvent| {
            ev.prevent_default();
            let zoom_factor = if ev.delta_y() < 0.0 { 1.1 } else { 0.9 };
            let old_zoom = zoom.get();
            let new_zoom = (old_zoom * zoom_factor).clamp(0.5, 3.0);

            let (old_x, old_y) = offset.get();
            offset.set((old_x * (new_zoom / old_zoom), old_y * (new_zoom / old_zoom)));

            zoom.set(new_zoom);
        }
    };

    view! {
        <div
            on:wheel=on_wheel
            on:mousedown=on_mouse_down
            class="w-full aspect-[3/1] sm:aspect-[2/1] md:aspect-[5/2] overflow-hidden bg-neutral-900"
        >
            <svg
                width="100%"
                height="100%"
                viewBox="-500 -500 1000 1000"
                preserveAspectRatio="xMidYMid meet"
            >
                // TODO: Find way to make this generic
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
                        format!("translate({x},{y}),scale({})", zoom.get())
                    }
                    filter="drop-shadow(0 2px 4px black)"
                >
                    {children()}
                </g>
            </svg>
        </div>
    }
}
