use leptos::{html::*, prelude::*};

use crate::components::{accessibility::AccessibilityContext, backend_client::BackendClient};

#[component]
pub fn PlayerCount() -> impl IntoView {
    let players_count = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move {
            backend
                .get_players_count()
                .await
                .map(|r| r.value)
                .unwrap_or_default()
        }
    });

    let accessibility: AccessibilityContext = expect_context();

    (!accessibility.is_on_mobile()).then(|| view! {
        <div class="fixed bottom-2 right-2 bg-black/70 text-amber-300 px-3 py-1 
        rounded-lg text-sm shadow-lg font-semibold backdrop-blur-sm 
        border border-gray-700 z-50">
            "Players online: " {move || players_count.get().map(|x| x.take()).unwrap_or_default()}
        </div>
    })
}
