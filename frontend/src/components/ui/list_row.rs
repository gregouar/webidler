use leptos::prelude::*;

use crate::components::settings::{GraphicsQuality, SettingsContext};

#[component]
pub fn MenuListRow(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional, into)] state_class: Option<Signal<String>>,
    #[prop(optional, into)] selected: Option<Signal<bool>>,
    #[prop(optional, into)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let selected = Signal::derive(move || selected.map(|selected| selected.get()).unwrap_or(false));
    let is_clickable = on_click.is_some();

    view! {
        <div
            class=move || {
                let quality = settings.graphics_quality();
                format!(
                    "relative isolate overflow-hidden rounded-[8px] border
                    {}
                    {}
                    transition-[border-color,background-color,box-shadow,transform] duration-150
                    {} {} {} {}",
                    match quality {
                        GraphicsQuality::High => {
                            "bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]"
                        }
                        GraphicsQuality::Medium => {
                            "bg-[linear-gradient(180deg,rgba(194,158,89,0.045),rgba(0,0,0,0.02)_32%,rgba(0,0,0,0.12)_100%),linear-gradient(135deg,rgba(38,37,43,0.98),rgba(18,18,22,1))]"
                        }
                        GraphicsQuality::Low => {
                            "bg-[linear-gradient(180deg,rgba(171,138,80,0.04),rgba(0,0,0,0.04)_34%,rgba(0,0,0,0.12)_100%),linear-gradient(135deg,rgba(37,36,41,0.98),rgba(19,18,22,1))]"
                        }
                    },
                    if settings.uses_heavy_effects() {
                        "shadow-[0_4px_12px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)]"
                    } else {
                        "shadow-md"
                    },
                    if selected.get() {
                        match quality {
                            GraphicsQuality::High => {
                                "border-[#b28a4f] shadow-[0_5px_14px_rgba(0,0,0,0.28),inset_0_1px_0_rgba(244,225,181,0.07),inset_0_0_0_1px_rgba(214,177,102,0.16)]"
                            }
                            GraphicsQuality::Medium => "border-[#9d7b45]",
                            GraphicsQuality::Low => "border-[#8a6d40]",
                        }
                    } else {
                        match quality {
                            GraphicsQuality::High => "border-[#3b3428]",
                            GraphicsQuality::Medium => "border-[#4a3e2b]",
                            GraphicsQuality::Low => "border-[#554631]",
                        }
                    },
                    if !selected.get() && is_clickable {
                        match quality {
                            GraphicsQuality::High => {
                                "cursor-pointer hover:border-[#75603c] hover:bg-[linear-gradient(180deg,rgba(226,193,122,0.065),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(46,45,52,0.99),rgba(22,22,27,1))]"
                            }
                            GraphicsQuality::Medium => {
                                "cursor-pointer hover:border-[#705a37] hover:bg-[linear-gradient(180deg,rgba(204,170,97,0.06),rgba(0,0,0,0.02)_32%,rgba(0,0,0,0.12)_100%),linear-gradient(135deg,rgba(43,42,48,0.99),rgba(21,21,26,1))]"
                            }
                            GraphicsQuality::Low => {
                                "cursor-pointer hover:border-[#6a5535] hover:bg-[linear-gradient(180deg,rgba(184,149,88,0.055),rgba(0,0,0,0.04)_34%,rgba(0,0,0,0.12)_100%),linear-gradient(135deg,rgba(41,40,46,0.99),rgba(20,20,24,1))]"
                            }
                        }
                    } else {
                        ""
                    },
                    class.unwrap_or_default(),
                    state_class.as_ref().map(|state_class| state_class.get()).unwrap_or_default(),
                )
            }
            on:click=move |ev| {
                let _ = ev;
                if let Some(on_click) = on_click.clone() {
                    on_click.run(());
                }
            }
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-[#d4b57a]/8"></div>
            </Show>
            <Show when=move || settings.uses_heavy_effects()>
                <div class="pointer-events-none absolute inset-x-3 top-0 h-px bg-gradient-to-r from-transparent via-[#edd39a]/40 to-transparent"></div>
            </Show>
            <div class="relative z-10">{children()}</div>
        </div>
    }
}
