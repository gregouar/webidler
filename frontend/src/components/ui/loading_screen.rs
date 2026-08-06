use leptos::prelude::*;

use crate::components::{
    pages::{LogoCog, LogoWord},
    ui::card::{Card, CardTitle},
};

#[component]
pub fn LoadingScreen(
    #[prop(default = "Loading...")] title: &'static str,
    detail: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex w-full items-center justify-center p-4 min-h-screen">
            <div class="flex w-full max-w-sm flex-col items-center">
                <LoadingLogo />

                <Card class="z-10 w-full">
                    <div
                        class="flex flex-col items-center px-4 py-6 text-center"
                        role="status"
                        aria-live="polite"
                        aria-label=title
                    >
                        <CardTitle>{title}</CardTitle>
                        <p class="mt-2 text-sm text-zinc-400">{detail}</p>
                    </div>
                </Card>
            </div>
        </div>
    }
}

#[component]
fn LoadingLogo() -> impl IntoView {
    view! {
        <div
            class="relative isolate -mb-5 flex h-40 w-64 items-center justify-center leading-none select-none"
            aria-hidden="true"
        >
            <div class="pointer-events-none absolute left-1/2 top-1/2 h-36 w-36 -translate-x-1/2 -translate-y-1/2 opacity-45 drop-shadow-[0_4px_12px_rgba(0,0,0,0.8)]">
                <div class="loading-gear progress-bar-animation h-full w-full">
                    <LogoCog />
                </div>
            </div>

            <div class="relative z-10 flex flex-col items-center">
                <LogoWord
                    text="GrinD"
                    class="text-[2.8rem] tracking-[0.06em]"
                    texture_size="96px 96px"
                    base_gradient="linear-gradient(180deg, rgba(255,251,236,0.99), rgba(245,224,168,0.99) 16%, rgba(217,159,72,0.98) 43%, rgba(134,78,34,0.99) 76%, rgba(58,30,12,0.99) 100%)"
                    highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.6), rgba(255,248,227,0.28) 17%, rgba(255,210,124,0.12) 40%, rgba(0,0,0,0.2) 100%)"
                    shadow="[text-shadow:0_1px_0_rgba(255,247,222,0.38),0_2px_0_rgba(116,80,38,0.88),0_5px_10px_rgba(0,0,0,0.78)]"
                />
                <LogoWord
                    text="to"
                    class="-mt-2 -mb-3 text-[0.95rem] tracking-[0.08em]"
                    texture_size="72px 72px"
                    base_gradient="linear-gradient(180deg, rgba(251,243,224,0.99), rgba(222,181,103,0.96) 38%, rgba(108,69,33,0.99) 100%)"
                    highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.42), rgba(255,246,221,0.15) 20%, rgba(0,0,0,0.18) 100%)"
                    shadow="[text-shadow:0_1px_0_rgba(255,244,214,0.25),0_2px_0_rgba(87,59,27,0.8),0_4px_8px_rgba(0,0,0,0.7)]"
                />
                <LogoWord
                    text="RusT"
                    class="text-[3rem] tracking-[0.05em]"
                    texture_size="104px 104px"
                    base_gradient="linear-gradient(180deg, rgba(255,246,198,0.99), rgba(240,190,100,0.98) 18%, rgba(206,112,48,0.97) 46%, rgba(110,49,18,0.99) 78%, rgba(43,17,8,0.99) 100%)"
                    highlight_gradient="linear-gradient(180deg, rgba(255,255,255,0.5), rgba(255,244,208,0.18) 18%, rgba(255,189,102,0.08) 42%, rgba(0,0,0,0.22) 100%)"
                    shadow="[text-shadow:0_1px_0_rgba(255,240,202,0.32),0_2px_0_rgba(106,59,25,0.88),0_5px_10px_rgba(0,0,0,0.8)]"
                />
            </div>
        </div>
    }
}
