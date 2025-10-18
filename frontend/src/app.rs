use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_toaster::*;
use url::Url;

use crate::components::{
    accessibility::provide_accessibility_context,
    auth::provide_auth_context,
    backend_client::BackendClient,
    events::provide_events_context,
    pages,
    ui::confirm::{provide_confirm_context, ConfirmationModal},
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
        .unwrap_or_else(|| "/".to_string());

    provide_context(BackendClient::new(
        option_env!("BACKEND_HTTP_URL").unwrap_or("http://localhost:4200"),
        option_env!("BACKEND_WS_URL").unwrap_or("ws://localhost:4200"),
    ));

    provide_auth_context();
    provide_accessibility_context();
    provide_events_context();

    let confirm_state = provide_confirm_context();

    view! {
        <Toaster position=ToasterPosition::BottomCenter></Toaster>
        <ConfirmationModal state=confirm_state />
        <Router base=base_uri>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::MainMenuPage />
                <Route path=path!("/terms") view=pages::terms::TermsPage />
                <Route path=path!("/privacy") view=pages::privacy::PrivacyPage />
                <Route path=path!("/game") view=pages::GamePage />
                // <Route path=path!("/game/:characterid") view=pages::GamePage />
                <Route path=path!("/leaderboard") view=pages::LeaderboardPage />
                <Route path=path!("/signup") view=pages::SignUpPage />
                <Route path=path!("/user-dashboard") view=pages::UserDashboardPage />
                <Route path=path!("/account") view=pages::AccountSettingsPage />
                <Route path=path!("/reset-password") view=pages::ResetPasswordPage />
                <Route path=path!("/town") view=pages::TownPage />
            </Routes>
        </Router>
    }
}
