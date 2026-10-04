use leptos::prelude::*;
use leptos_use::on_click_outside;

#[component]
pub fn ContextMenu(on_close: Callback<()>, children: Children) -> impl IntoView {
    let node_ref = NodeRef::new();

    let _ = on_click_outside(node_ref, move |_| {
        on_close.run(());
    });

    view! {
        <div
            node_ref=node_ref
            class="
            absolute inset-0 z-30 flex flex-col justify-center 
            w-full
            p-1
            text-center
            overflow-clip
            "
            style="
            animation: fade-in 0.2s ease-out forwards;
            linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.16)),
            linear-gradient(135deg, rgba(42,40,46,0.96), rgba(17,16,20,0.98));
            border: 1px solid rgba(108,83,41,0.72);
            box-shadow:
            0 10px 22px rgba(0,0,0,0.52),
            inset 0 1px 0 rgba(240,215,159,0.16),
            inset 0 -1px 0 rgba(0,0,0,0.45);
            "
        >
            <div class="pointer-events-none absolute inset-[1px] border border-white/5"></div>
            <div class="relative z-10 flex flex-col">{children()}</div>
        </div>
    }
}

#[derive(Clone, Copy)]
pub enum ActionMenuTone {
    Success,
    Skill,
    Warning,
    Neutral,
}

#[derive(Clone, Copy)]
struct ActionMenuRowTone {
    text: &'static str,
    hover_text: &'static str,
    wash: &'static str,
}

fn action_menu_row_tone(tone: ActionMenuTone) -> ActionMenuRowTone {
    match tone {
        ActionMenuTone::Success => ActionMenuRowTone {
            text: "text-amber-300",
            hover_text: "hover:text-[#f2e5bc]",
            wash: "rgba(247, 190, 77, 0.3)",
        },
        ActionMenuTone::Skill => ActionMenuRowTone {
            text: "text-violet-300",
            hover_text: "hover:text-fuchsia-100",
            wash: "rgba(112,80,138,0.36)",
        },
        ActionMenuTone::Warning => ActionMenuRowTone {
            text: "text-[#f86c47]",
            hover_text: "hover:text-[#ffd4c8]",
            wash: "rgba(192,92,61,0.30)",
        },
        ActionMenuTone::Neutral => ActionMenuRowTone {
            text: "text-zinc-300",
            hover_text: "hover:text-zinc-100",
            wash: "rgba(255,255,255,0.10)",
        },
    }
}

#[component]
pub fn ActionMenuRow(
    #[prop(optional)] label: Option<&'static str>,
    #[prop(optional, into)] label_signal: Option<Signal<String>>,
    tone: ActionMenuTone,
    #[prop(into)] on_click: Callback<()>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let tone = action_menu_row_tone(tone);
    view! {
        <button
            class=format!(
                "btn relative w-full overflow-clip px-2 xl:px-2.5 py-1.5 xl:py-2
                text-sm xl:text-base font-semibold tracking-[0.04em]
                transition-colors duration-150 {} {}
                bg-zinc-900/90
                text-center active:brightness-90 disabled:opacity-50 disabled:cursor-not-allowed",
                tone.text,
                tone.hover_text,
            )
            on:click=move |_| on_click.run(())
            disabled=move || disabled.get()
        >
            <div
                class="pointer-events-none absolute inset-0 hover:bg-white/[0.02]"
                style=format!(
                    "background:
                    linear-gradient(90deg, transparent, {}, transparent);
                 border-top: 1px solid rgba(255,255,255,0.04);",
                    tone.wash,
                )
            />
            <span class="pointer-events-none absolute inset-x-3 top-0 h-px bg-gradient-to-r from-transparent via-white/8 to-transparent"></span>
            <span class="drop-shadow-[0_2px_2px_rgba(0,0,0,0.95)]">
                {move || {
                    label_signal
                        .as_ref()
                        .map(|label_signal| label_signal.get())
                        .or_else(|| label.map(str::to_string))
                        .unwrap_or_default()
                }}
            </span>
        </button>
    }
}
