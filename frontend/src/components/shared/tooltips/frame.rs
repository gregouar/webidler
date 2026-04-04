use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct TooltipFramePalette {
    pub border_class: &'static str,
    pub inner_border_class: &'static str,
    pub shadow_color: &'static str, // TODO: Remove
    pub wash_color: &'static str,
    pub core_color: &'static str,
    pub shine_color: &'static str,
}

#[component]
pub fn TooltipFrame(
    palette: TooltipFramePalette,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("relative isolate text-center {}", class.unwrap_or("max-w-xs"))>
            <div
                class="pointer-events-none absolute inset-0 shadow-[0_0_10px_rgba(0,0,0,0.45)]"
                aria-hidden="true"
            >
                // style=format!(
                // "filter: drop-shadow(0 10px 20px {}) drop-shadow(0 3px 5px rgba(0,0,0,0.45));",
                // palette.shadow_color,
                // )
                <div
                    class="absolute inset-0 bg-black/90"
                    style="clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 10px, 100% calc(100% - 10px), calc(100% - 10px) 100%, 10px 100%, 0 calc(100% - 10px), 0 10px);"
                ></div>
            </div>

            <div
                class=format!(
                    "relative overflow-hidden border {} shadow-[inset_0_1px_0_rgba(240,215,159,0.16),inset_0_-1px_0_rgba(0,0,0,0.5)]",
                    palette.border_class,
                )
                style=format!(
                    "clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 10px, 100% calc(100% - 10px), calc(100% - 10px) 100%, 10px 100%, 0 calc(100% - 10px), 0 10px);
                    background-image:
                        linear-gradient(180deg, rgba(214,177,102,0.05), rgba(0,0,0,0.2)),
                        radial-gradient(circle at 50% 20%, {}, transparent 64%),
                        linear-gradient(180deg, {}, transparent 34%),
                        linear-gradient(135deg, rgba(31,29,36,0.985), rgba(9,9,12,1));
                    background-blend-mode: screen, soft-light, screen, normal;",
                    palette.core_color,
                    palette.wash_color,
                )
            >
                <div
                    class=format!(
                        "pointer-events-none absolute inset-[1px] border {}",
                        palette.inner_border_class,
                    )
                    style="clip-path: polygon(9px 0, calc(100% - 9px) 0, 100% 9px, 100% calc(100% - 9px), calc(100% - 9px) 100%, 9px 100%, 0 calc(100% - 9px), 0 9px);"
                ></div>
                <span
                    class="pointer-events-none absolute inset-x-[6px] top-[2px] h-[2px]"
                    style=format!(
                        "background: linear-gradient(90deg, transparent, {}, transparent);",
                        palette.shine_color,
                    )
                ></span>
                <span
                    class="pointer-events-none absolute inset-y-[5px] left-[1px] w-[2px]"
                    style=format!(
                        "background: linear-gradient(180deg, transparent, {}, transparent);",
                        palette.shine_color,
                    )
                ></span>
                <span
                    class="pointer-events-none absolute inset-y-[5px] right-[1px] w-[2px]"
                    style=format!(
                        "background: linear-gradient(180deg, transparent, {}, transparent);",
                        palette.shine_color,
                    )
                ></span>
                <div class="relative space-y-2 p-2 xl:p-4">{children()}</div>
            </div>
        </div>
    }
}
