use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::http::client::UpdateAccountRequest;

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        input::{Input, ValidatedInput},
        toast::*,
    },
};

#[component]
pub fn AccountSettingsPage() -> impl IntoView {
    let toaster = expect_context::<Toasts>();
    let backend = expect_context::<BackendClient>();
    let auth_context = expect_context::<AuthContext>();

    let navigate_to_dashboard = {
        let navigate = use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    let (get_username_storage, set_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("username");

    let init_username = RwSignal::new(get_username_storage.get_untracked());
    let username = RwSignal::new(get_username_storage.get_untracked());

    let init_email = RwSignal::new(Some(None));
    let email = RwSignal::new(Some(None));

    let old_password = RwSignal::new(None);
    let password = RwSignal::new(None);
    let confirm_password = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let passwords_mismatch = Signal::derive(move || password.get() != confirm_password.get());

    let disable_username_submit = Signal::derive(move || {
        username.read().is_none() || *init_username.read() == *username.read() || processing.get()
    });
    let on_update_username = {
        move |_| {
            spawn_local({
                async move {
                    match backend
                        .post_update_account(
                            &auth_context.token(),
                            &UpdateAccountRequest {
                                username: username.get_untracked().unwrap(),
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        Ok(_) => {
                            set_username_storage.set(username.get_untracked());
                            init_username.set(username.get_untracked());
                            show_toast(
                                toaster,
                                format!("Update account success!"),
                                ToastVariant::Success,
                            );
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Update account error: {e}"),
                                ToastVariant::Error,
                            );
                            processing.set(false);
                        }
                    }
                }
            });
        }
    };

    let disable_email_submit = Signal::derive(move || {
        email.read().is_none() || *init_email.read() == *email.read() || processing.get()
    });
    let on_update_email = {
        move |_| {
            spawn_local({
                async move {
                    match backend
                        .post_update_account(
                            &auth_context.token(),
                            &UpdateAccountRequest {
                                email: Some(email.get_untracked().unwrap()),
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        Ok(_) => {
                            init_email.set(email.get_untracked());
                            show_toast(
                                toaster,
                                format!("Update account success!"),
                                ToastVariant::Success,
                            );
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Update account error: {e}"),
                                ToastVariant::Error,
                            );
                            processing.set(false);
                        }
                    }
                }
            });
        }
    };

    let disable_password_submit = Signal::derive(move || {
        old_password.read().is_none()
            || password.read().is_none()
            || passwords_mismatch.get()
            || processing.get()
    });
    let on_update_password = {
        move |_| {
            spawn_local({
                async move {
                    match backend
                        .post_update_account(
                            &auth_context.token(),
                            &UpdateAccountRequest {
                                old_password: Some(old_password.get().unwrap()),
                                password: Some(password.get().unwrap()),
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        Ok(_) => {
                            show_toast(
                                toaster,
                                format!("Update password success!"),
                                ToastVariant::Success,
                            );
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Update password error: {e}"),
                                ToastVariant::Error,
                            );
                            processing.set(false);
                        }
                    }
                }
            });
        }
    };

    let user_data = LocalResource::new({
        move || async move {
            backend
                .get_me(&auth_context.token())
                .await
                .map(|r| r.user_details)
                .ok()
        }
    });

    Effect::new(move || {
        if let Some(user) = user_data.get() {
            email.set(user.as_ref().map(|user| user.email.clone()));
            init_email.set(email.get_untracked());
        }
    });

    view! {
        <main class="my-0 mx-auto max-w-2xl text-center flex flex-col justify-center p-6">

            <h1 class="text-amber-200 text-4xl font-extrabold mb-6">"Account Settings"</h1>

            <div class="space-y-6 text-right text-white  mb-6">
                <div class="border-t border-zinc-700 pt-4 mt-4 space-y-2">
                    <ValidatedInput
                        label="Username"
                        id="username"
                        input_type="text"
                        placeholder="Enter your username"
                        bind=username
                    />
                    <MenuButton on:click=on_update_username disabled=disable_username_submit>
                        "Change Username"
                    </MenuButton>
                </div>

                <div class="border-t border-zinc-700 pt-4 mt-4 space-y-2">
                    <ValidatedInput
                        label="Email"
                        id="email"
                        input_type="text"
                        placeholder="Optionally enter your email for password recovery"
                        bind=email
                    />
                    <MenuButton on:click=on_update_email disabled=disable_email_submit>
                        "Change Email"
                    </MenuButton>
                </div>

                <div class="border-t border-zinc-700 pt-4 mt-4 space-y-2">
                    <ValidatedInput
                        label="Old Password"
                        id="old-password"
                        input_type="password"
                        placeholder="Enter your old password"
                        bind=old_password
                    />
                    <ValidatedInput
                        label="New Password"
                        id="password"
                        input_type="password"
                        placeholder="Enter your new password"
                        bind=password
                    />

                    <Input
                        id="confirm-password"
                        input_type="password"
                        placeholder="Confirm your new password"
                        bind=confirm_password
                        invalid=passwords_mismatch
                    />
                    <MenuButton
                        class:justify-self-end
                        on:click=on_update_password
                        disabled=disable_password_submit
                    >
                        "Change Password"
                    </MenuButton>
                </div>

                <div class="border-t border-zinc-700 pt-4 mt-4 space-y-2">
                    <p class="text-sm text-red-400">
                        "Deleting your account is irreversible. All game progress will be lost."
                    </p>
                    <MenuButtonRed on:click=move |_| {}>"Delete Account"</MenuButtonRed>
                </div>

            </div>

            <div>
                <MenuButton on:click=navigate_to_dashboard>"Back"</MenuButton>
            </div>
        </main>
    }
}
