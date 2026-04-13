use leptos::{html::*, prelude::*};

use crate::{
    assets::img_asset,
    components::{
        settings::{GraphicsQuality, SettingsContext},
        ui::{buttons::CloseButton, separator::TitleSeparator},
    },
};

#[component]
pub fn Card(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = true)] gap: bool,
    #[prop(default = true)] pad: bool,
    #[prop(default = true)] enable_texture: bool,
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let stone_texture = img_asset("ui/dark_stone.webp");

    let enable_texture = move || enable_texture && settings.uses_textures();

    view! {
        <div class=format!("max-h-full flex flex-col relative {}", class.unwrap_or_default())>
            <Show when=move || settings.uses_textures()>
                <div
                    class="pointer-events-none absolute inset-0 drop-shadow-[0_10px_25px_rgba(0,0,0,0.45)]"
                    aria-hidden="true"
                >
                    <div class="absolute inset-0 bg-black clip-octagon"></div>
                </div>
            </Show>
            <div
                class=move || {
                    let quality = settings.graphics_quality();
                    format!(
                        "clip-octagon absolute inset-0 border {} {}",
                        match quality {
                            GraphicsQuality::High => {
                                "border-[#6c5734]/45 shadow-[inset_2px_2px_1px_rgba(255,255,255,0.06),inset_-2px_-2px_1px_rgba(0,0,0,0.15)]"
                            }
                            GraphicsQuality::Medium => "border-[#6c5734]/50",
                            GraphicsQuality::Low => "border-[#4f4532]",
                        },
                        if quality.uses_textures() { "" } else { "bg-zinc-800" },
                    )
                }
                style=move || {
                    match settings.graphics_quality() {
                        GraphicsQuality::High if enable_texture() => {
                            format!(
                                "
                                background-image:
                                    linear-gradient(180deg, rgba(214, 165, 102, 0.04), rgba(0,0,0,0)),
                                    url('{}');
                                background-blend-mode: screen, normal;
                                ",
                                stone_texture,
                            )
                        }
                        GraphicsQuality::Medium if enable_texture() => {
                            format!("background-image: url('{}');", stone_texture)
                        }
                        _ => String::new(),
                    }
                }
            >
                <Show when=move || settings.uses_surface_effects()>
                    <div class="pointer-events-none clip-octagon [--cut:11px] absolute inset-[1px] border border-white/6" />
                </Show>
            </div>

            <div class=format!(
                "relative z-10 flex h-full flex-col clip-octagon {} {}",
                if gap { "gap-1 xl:gap-2" } else { "" },
                if pad { "p-1 xl:p-3" } else { "m-[2px] overflow-hidden" },
            )>{children()}</div>
        </div>
    }
}

#[component]
pub fn MenuCard(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = true)] gap: bool,
    #[prop(default = true)] pad: bool,
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let stone_texture = img_asset("ui/dark_stone.webp");

    view! {
        <div
            class=move || {
                let quality = settings.graphics_quality();
                format!(
                    "max-h-full flex flex-col relative clip-octagon
                    {} {} {} {}",
                    match quality {
                        GraphicsQuality::High => {
                            "border border-[#6c5734]/45 shadow-[inset_2px_2px_1px_rgba(255,255,255,0.06),inset_-2px_-2px_1px_rgba(0,0,0,0.15)]"
                        }
                        GraphicsQuality::Medium => "border border-[#6c5734]/50",
                        GraphicsQuality::Low => "border border-[#4f4532] bg-zinc-800",
                    },
                    if gap { "gap-1 xl:gap-2" } else { "" },
                    if pad { "p-1 xl:p-3" } else { "p-[2px]" },
                    class.unwrap_or_default(),
                )
            }
            style=move || {
                match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        format!(
                            "
                            background-image:
                                linear-gradient(180deg, rgba(214, 165, 102, 0.04), rgba(0,0,0,0)),
                                url('{}');
                            background-blend-mode: screen, normal;
                            ",
                            stone_texture,
                        )
                    }
                    GraphicsQuality::Medium => {
                        format!("background-image: url('{}');", stone_texture)
                    }
                    GraphicsQuality::Low => String::new(),
                }
            }
        >

            {children()}
        </div>
    }
}

// #[component]
// pub fn Card(
//     #[prop(optional)] class: Option<&'static str>,
//     #[prop(default = true)] gap: bool,
//     #[prop(default = true)] pad: bool,
//     children: Children,
// ) -> impl IntoView {
//     view! {
//         <div class=format!(
//             "max-h-full flex flex-col relative
//             bg-zinc-800
//             rounded-[6px] xl:rounded-[8px]

//             ring-1 ring-zinc-700/50
//             shadow-[0_4px_6px_rgba(0,0,0,0.25),inset_1px_1px_1px_rgba(255,255,255,0.06),inset_-1px_-1px_1px_rgba(0,0,0,0.15)]
//             {} {} {}",
//             class.unwrap_or_default(),
//             if gap { "gap-1 xl:gap-2" } else { "" },
//             if pad { "p-1 xl:p-3" } else { "" },
//         )>{children()}</div>
//     }
// }

#[component]
pub fn CardTitle(children: Children) -> impl IntoView {
    let settings: SettingsContext = expect_context();

    view! {
        <span class=move || {
            format!(
                "text-amber-200 font-semibold text-base xl:text-xl font-display {}",
                if settings.uses_surface_effects() {
                    "text-shadow-lg/100 shadow-gray-950"
                } else {
                    ""
                },
            )
        }>{children()}</span>
    }
}

#[component]
pub fn CardHeader(
    title: &'static str,
    on_close: impl Fn() + Send + Sync + 'static,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="px-4 relative z-10 flex items-center justify-between">
            <CardTitle>{title}</CardTitle>
            {children.map(|children| children())}
            <CloseButton on:click=move |_| on_close() />
        </div>
    }
}

#[component]
pub fn CardInsetTitle(children: Children) -> impl IntoView {
    let settings: SettingsContext = expect_context();

    view! {
        <div class="w-full flex flex-col mt-1 mb-2 gap-1">
            <span class=move || {
                format!(
                    "text-amber-200 text-sm xl:text-base font-display font-semibold tracking-[0.08em] {}",
                    if settings.uses_surface_effects() {
                        "text-shadow-md/50 shadow-gray-950"
                    } else {
                        ""
                    },
                )
            }>{children()}</span>
            <TitleSeparator />
        </div>
    }
}

#[component]
pub fn CardInset(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = true)] gap: bool,
    #[prop(default = true)] pad: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            format!(
                "flex flex-col overflow-y-auto
                bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)] ring-1 ring-zinc-950 
                {} {} {}",
                class.unwrap_or_default(),
                if gap { "gap-1 xl:gap-2" } else { "" },
                if pad { "p-1 xl:p-3" } else { "" },
            )
        }>{children()}</div>
    }
}
