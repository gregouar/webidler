use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use url::Url;

use crate::components::{
    accessibility::provide_accessibility_context,
    auth::provide_auth_context,
    backend_client::BackendClient,
    chat::chat_context::ChatProvider,
    data_context::provide_data_context,
    events::provide_events_context,
    pages,
    settings::provide_settings_context,
    ui::{
        confirm::{ConfirmationModal, provide_confirm_context},
        toast::{Toaster, ToasterPosition, provide_toasts},
        tooltip::DynamicTooltip,
    },
};
// TODO: localization https://crates.io/crates/fluent-templates

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let base_uri = document()
        .base_uri()
        .ok()
        .flatten()
        .and_then(|base| Url::parse(&base).ok())
        .map(|url| url.path().to_string())
        .unwrap_or_default();

    let base_uri = router_base_from_base_uri(base_uri);

    console_log(&base_uri);

    provide_context(BackendClient::new(
        option_env!("BACKEND_HTTP_URL").unwrap_or("http://localhost:4200"),
        option_env!("BACKEND_WS_URL").unwrap_or("ws://localhost:4200"),
    ));

    provide_auth_context();
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
        <ChatProvider url=option_env!("BACKEND_CHAT_WS_URL")
            .unwrap_or("ws://localhost:4242/chatws")
            .into()>
            <Router base=base_uri>
                <Routes fallback=|| "Page not found.">
                    <Route path=path!("/") view=pages::MainMenuPage />
                    <Route path=path!("/terms") view=pages::terms::TermsPage />
                    <Route path=path!("/privacy") view=pages::privacy::PrivacyPage />
                    <Route path=path!("/game") view=pages::GamePage />
                    <Route path=path!("/signup") view=pages::SignUpPage />
                    <Route path=path!("/user-dashboard") view=pages::UserDashboardPage />
                    <Route path=path!("/reset-password") view=pages::ResetPasswordPage />
                    <Route path=path!("/town") view=pages::TownPage />
                    <Route
                        path=path!("/view-character/:character_name")
                        view=pages::ViewCharacterPage
                    />
                </Routes>
            </Router>
        </ChatProvider>
    }
}

fn router_base_from_base_uri(mut base_uri: String) -> String {
    if base_uri.starts_with("/html/") && !base_uri.ends_with("index.html") {
        if !base_uri.ends_with('/') {
            base_uri.push('/');
        }
        base_uri.push_str("index.html");
    }

    if base_uri == "/" {
        return String::new();
    }

    if !base_uri.starts_with('/') {
        base_uri.insert(0, '/');
    }

    base_uri.trim_end_matches('/').to_string()
}
