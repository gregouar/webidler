use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct TooltipFramePalette {
    pub border_class: &'static str,
    pub inner_border_class: &'static str,
    pub shine_color: &'static str,
}

#[component]
pub fn TooltipFrame(
    palette: TooltipFramePalette,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "relative isolate text-center  backdrop-blur-sm {}",
            class.unwrap_or("max-w-xs"),
        )>
            <div
                class="pointer-events-none absolute inset-0 rounded-[3px]"
                aria-hidden="true"
                style="filter: drop-shadow(0 0px 10px rgba(0,0,0,0.45));"
            >
                <div class="absolute inset-0 rounded-[3px] bg-black/78"></div>
            </div>

            <div class=format!(
                "relative overflow-hidden rounded-[3px] border {} shadow-[inset_0_1px_0_rgba(255,255,255,0.05),inset_0_-1px_0_rgba(0,0,0,0.34)]",
                palette.border_class,
            )>
                <div class=format!(
                    "pointer-events-none absolute inset-[1px] rounded-[2px] border {}",
                    palette.inner_border_class,
                )></div>
                <span
                    class="pointer-events-none absolute inset-x-[14px] top-[2px] h-px rounded-full"
                    style=format!(
                        "background: linear-gradient(90deg, transparent, {}, transparent);",
                        palette.shine_color,
                    )
                ></span>
                <div class="pointer-events-none absolute inset-x-0 top-0 h-[24%] bg-[linear-gradient(180deg,rgba(255,255,255,0.024),transparent)]"></div>
                <div class="pointer-events-none absolute inset-x-0 bottom-0 h-[22%] bg-[linear-gradient(0deg,rgba(0,0,0,0.16),transparent)]"></div>
                <div class="relative space-y-2 p-2 xl:p-4">{children()}</div>
            </div>
        </div>
    }
}
