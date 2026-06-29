use frontend::components::{
    accessibility::provide_accessibility_context,
    backend_client::BackendClient,
    data_context::provide_data_context,
    events::provide_events_context,
    settings::provide_settings_context,
    ui::{
        confirm::{ConfirmationModal, provide_confirm_context},
        toast::{Toaster, ToasterPosition, provide_toasts},
        tooltip::DynamicTooltip,
    },
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::pages;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    provide_context(BackendClient::new(
        option_env!("BACKEND_HTTP_URL").unwrap_or("http://localhost:4200"),
        option_env!("BACKEND_WS_URL").unwrap_or("ws://localhost:4200"),
    ));

    provide_accessibility_context();
    provide_settings_context();
    provide_events_context();
    provide_data_context();
    provide_toasts();

    let confirm_state = provide_confirm_context();

    view! {
        <Toaster position=ToasterPosition::BottomCenter></Toaster>
        <ConfirmationModal state=confirm_state />
        <DynamicTooltip />
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::HomePage />
                <Route path=path!("/passives") view=pages::PassivesPage />
                <Route path=path!("/skills") view=pages::SkillsPage />
                <Route path=path!("/ui_tests") view=pages::UiTestsPage />
                <Route path=path!("/items") view=pages::ItemsPage />
                <Route path=path!("/logo") view=pages::LogoPage />
            </Routes>
        </Router>
    }
}
