use leptos::{html::*, prelude::*};

use crate::components::{accessibility::AccessibilityContext, backend_client::BackendClient};

#[component]
pub fn PlayerCount() -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let accessibility: AccessibilityContext = expect_context();

    let areas_and_players_data = RwSignal::new(None);
    let show_glimpse = RwSignal::new(false);

    LocalResource::new({
        move || async move {
            let areas = backend
                .get_areas()
                .await
                .map(|resp| resp.areas)
                .unwrap_or_default();

            if let Ok(resp) = backend.get_players_count().await {
                areas_and_players_data.set(Some((areas, resp)));
            }
        }
    });

    (!accessibility.is_on_mobile()).then(|| {
        view! {
            <div
                class="fixed bottom-2 right-2 bg-black/70 text-amber-300 px-3 py-1
                rounded-lg text-sm shadow-lg font-semibold backdrop-blur-sm
                border border-gray-700 z-50 cursor-pointer select-none"
                on:click=move |_| {
                    show_glimpse.update(|s| *s = !*s);
                }
            >
                {move || {
                    let value = areas_and_players_data
                        .read()
                        .as_ref()
                        .map(|d| d.1.value)
                        .unwrap_or_default();
                    format!("Players currently grinding: {}", value)
                }}

                {move || {
                    show_glimpse
                        .get()
                        .then(|| {
                            let glimpse = areas_and_players_data
                                .read()
                                .as_ref()
                                .map(|d| d.1.glimpse.clone())
                                .unwrap_or_default();
                            let areas = areas_and_players_data
                                .read()
                                .as_ref()
                                .map(|d| d.0.clone())
                                .unwrap_or_default();

                            view! {
                                {if glimpse.is_empty() {
                                    view! {
                                        <div class="absolute bottom-full right-0 mb-2 overflow-auto
                                        bg-zinc-900 text-white text-xs rounded-lg border border-zinc-700 shadow-lg p-2 space-y-1 z-50">
                                            <div class="text-gray-400 text-center">
                                                "No players grinding"
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div class="absolute w-96 bottom-full right-0 mb-2 overflow-auto
                                        bg-zinc-900 text-white text-xs rounded-lg border border-zinc-700 shadow-lg p-2 space-y-1 z-50
                                        grid grid-cols-2">
                                            {glimpse
                                                .into_iter()
                                                .map(|entry| {
                                                    view! {
                                                        <a href=format!("view-character/{}", &entry.character_name)>
                                                            <p class="">
                                                                {entry.character_name.clone()} <br /> "(" {entry.username}
                                                                ")"
                                                            </p>
                                                        </a>
                                                        <p class="text-amber-400">
                                                            {areas
                                                                .get(&entry.area_id)
                                                                .map(|area| area.name.clone())
                                                                .unwrap_or("Somewhere".into())}<br />{entry.area_level}
                                                        </p>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    }
                                        .into_any()
                                }}
                            }
                        })
                }}
            </div>
        }
    })
}

// #[component]
// pub fn PlayerCount() -> impl IntoView {
//     let players_count = LocalResource::new({
//         let backend = expect_context::<BackendClient>();
//         move || async move {
//             backend
//                 .get_players_count()
//                 .await
//                 .map(|r| r.value)
//                 .unwrap_or_default()
//         }
//     });

//     let accessibility: AccessibilityContext = expect_context();

//     (!accessibility.is_on_mobile()).then(|| view! {
//         <div class="fixed bottom-2 right-2 bg-black/70 text-amber-300 px-3 py-1
//         rounded-lg text-sm shadow-lg font-semibold backdrop-blur-sm
//         border border-gray-700 z-50">
//             "Players online: " {move || players_count.get().map(|x| x.take()).unwrap_or_default()}
//         </div>
//     })
// }
