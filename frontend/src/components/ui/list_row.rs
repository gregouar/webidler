use leptos::prelude::*;

#[component]
pub fn MenuListRow(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional, into)] state_class: Option<Signal<String>>,
    #[prop(optional, into)] selected: Option<Signal<bool>>,
    #[prop(optional, into)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let selected = Signal::derive(move || selected.map(|selected| selected.get()).unwrap_or(false));
    let is_clickable = on_click.is_some();

    view! {
        <div
            class=move || {
                format!(
                    "relative isolate overflow-hidden rounded-[8px] border
                    bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
                    shadow-[0_4px_12px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)]
                    transition-[border-color,background-color,box-shadow,transform] duration-150
                    {} {} {} {}",
                    if selected.get() {
                        "border-[#b28a4f] shadow-[0_5px_14px_rgba(0,0,0,0.28),inset_0_1px_0_rgba(244,225,181,0.07),inset_0_0_0_1px_rgba(214,177,102,0.16)]"
                    } else {
                        "border-[#3b3428]"
                    },
                    if !selected.get() && is_clickable {
                        "cursor-pointer hover:border-[#75603c] hover:bg-[linear-gradient(180deg,rgba(226,193,122,0.065),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(46,45,52,0.99),rgba(22,22,27,1))]"
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
            <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-white/5"></div>
            <div class="pointer-events-none absolute inset-x-3 top-0 h-px bg-gradient-to-r from-transparent via-[#edd39a]/40 to-transparent"></div>
            <div class="relative z-10">{children()}</div>
        </div>
    }
}
