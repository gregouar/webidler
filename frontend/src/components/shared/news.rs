use leptos::prelude::*;

use shared::http::server::NewsEntry;

use crate::components::{
    backend_client::BackendClient,
    ui::{card::CardInset, number::format_datetime},
};

#[component]
pub fn NewsInset(#[prop(default = "w-full gap-3")] class: &'static str) -> impl IntoView {
    let news_data = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move { backend.get_news().await.unwrap_or_default().entries }
    });

    view! {
        <CardInset class=class class:text-left>
            <Transition fallback=move || {
                view! { <p class="text-zinc-400">"Loading..."</p> }
            }>
                {move || {
                    Suspend::new(async move {
                        let news = news_data.await;
                        view! {
                            <For
                                each=move || news.clone()
                                key=|entry| entry.timestamp
                                children=move |news| {
                                    view! { <NewsCard news /> }
                                }
                            />
                        }
                    })
                }}
            </Transition>
        </CardInset>
    }
}

#[component]
fn NewsCard(news: NewsEntry) -> impl IntoView {
    let mut lines = news.content.lines();
    let title = lines.next().unwrap_or("").trim().to_string();
    let body = lines.collect::<Vec<_>>().join("\n");

    view! {
        <div class="rounded-[10px] border border-[#5f5137]/60
        select-text
        bg-[linear-gradient(180deg,rgba(214,177,102,0.035),rgba(0,0,0,0.08)),linear-gradient(135deg,rgba(39,38,44,0.96),rgba(18,18,22,1))]
        shadow-[0_6px_14px_rgba(0,0,0,0.18),inset_0_1px_0_rgba(255,255,255,0.035)]
        p-4 flex flex-col gap-3">
            <div class="relative z-10 flex items-start justify-between gap-3">
                <span class="text-amber-300 font-semibold text-base font-display text-shadow-lg/100 shadow-gray-950 leading-tight">
                    {title}
                </span>

                <span class="shrink-0 text-xs text-gray-500 uppercase tracking-[0.08em]">
                    {format_datetime(news.timestamp)}
                </span>
            </div>

            <p class="relative z-10 text-zinc-300 text-sm whitespace-pre-line leading-relaxed">
                {body}
            </p>
        </div>
    }
}
