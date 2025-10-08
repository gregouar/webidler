use leptos::{html::*, prelude::*, task::spawn_local, Params};
use leptos_router::{
    hooks::{use_navigate, use_query},
    params::Params,
};

use shared::{data::user::UserId, http::client::ResetPasswordRequest};

use crate::components::{
    backend_client::BackendClient,
    captcha::Captcha,
    ui::{
        buttons::MenuButton,
        input::{Input, ValidatedInput},
        toast::*,
    },
};

#[derive(Params, PartialEq, Clone, Default, Debug)]
struct ResetPasswordParams {
    user_id: Option<UserId>,
    token: Option<String>,
}

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let navigate_to_menu = {
        let navigate = use_navigate();
        move |_| {
            navigate("/", Default::default());
        }
    };

    let query = use_query::<ResetPasswordParams>();

    let password = RwSignal::new(None);
    let confirm_password = RwSignal::new(None);
    let captcha_token = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let passwords_mismatch = Signal::derive(move || password.get() != confirm_password.get());
    let disable_submit = Signal::derive(move || {
        !query
            .read()
            .as_ref()
            .ok()
            .map(|query| query.user_id.is_some())
            .unwrap_or_default()
            || password.read().is_none()
            || passwords_mismatch.get()
            || captcha_token.read().is_none()
            || processing.get()
    });

    let on_submit = {
        let toaster = expect_context::<Toasts>();
        let backend = expect_context::<BackendClient>();
        let navigate = use_navigate();
        move |_| {
            if disable_submit.get() {
                return;
            }

            processing.set(true);
            let navigate = navigate.clone();
            let query = query.get().unwrap_or_default();
            spawn_local({
                async move {
                    match backend
                        .post_reset_password(&ResetPasswordRequest {
                            captcha_token: captcha_token.get().unwrap_or_default(),
                            user_id: query.user_id.unwrap_or_default(),
                            password_token: query.token.unwrap_or_default(),
                            password: password.get().unwrap(),
                        })
                        .await
                    {
                        Ok(_) => {
                            show_toast(
                                toaster,
                                format!("Reset password success!"),
                                ToastVariant::Success,
                            );
                            navigate("/", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Reset password error: {e}"),
                                ToastVariant::Error,
                            );
                            processing.set(false);
                        }
                    }
                }
            });
        }
    };
    view! {
        <main class="my-0 mx-auto max-w-2xl text-center flex flex-col justify-center p-6 gap-6">
            <h1 class="text-amber-200 text-4xl font-extrabold">"Reset Password"</h1>

            <p class="text-left">
                "You are about the reset your password. If you didn't request a password change, please go back."
            </p>

            <div class="space-y-4 text-left text-white">

                <ValidatedInput
                    label="Password"
                    id="password"
                    input_type="password"
                    placeholder="Enter your password"
                    bind=password
                />

                <Input
                    id="confirm-password"
                    input_type="password"
                    placeholder="Confirm your password"
                    bind=confirm_password
                    invalid=passwords_mismatch
                />

                <div class="w-full flex justify-center">
                    <Captcha token=captcha_token />
                </div>

                <MenuButton class:w-full on:click=on_submit disabled=disable_submit>
                    "Confirm"
                </MenuButton>
            </div>

            <div>
                <MenuButton on:click=navigate_to_menu>"Back"</MenuButton>
            </div>
        </main>
    }
}
