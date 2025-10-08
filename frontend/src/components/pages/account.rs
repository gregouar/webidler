use codee::string::JsonSerdeCodec;
use nutype::nutype;

use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::{data::user::UserId, http::client::UpdateAccountRequest};

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

    let user_id = RwSignal::new(None);

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

    let show_delete_modal = RwSignal::new(false);

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
            user_id.set(user.as_ref().map(|user| user.user.user_id.clone()));
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
                    <MenuButtonRed on:click=move |_| {
                        show_delete_modal.set(true)
                    }>"Delete Account"</MenuButtonRed>
                </div>

            </div>

            <div>
                <MenuButton on:click=navigate_to_dashboard>"Back"</MenuButton>
            </div>

            <ConfirmAccountDeletionModal open=show_delete_modal user_id />
        </main>
    }
}

fn validate_delete(s: &str) -> anyhow::Result<()> {
    if s.eq_ignore_ascii_case("delete") {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Type DELETE to confirm."))
    }
}

#[nutype(
    sanitize(trim),
    validate(with = validate_delete, error = anyhow::Error),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct DeleteInput(String);

#[component]
pub fn ConfirmAccountDeletionModal(
    open: RwSignal<bool>,
    user_id: RwSignal<Option<UserId>>,
) -> impl IntoView {
    let confirm_input = RwSignal::new(None::<DeleteInput>);

    let node_ref = NodeRef::new();
    let _ = leptos_use::on_click_outside(node_ref, move |_| open.set(false));

    let do_delete = {
        let toaster = expect_context::<Toasts>();
        let backend = expect_context::<BackendClient>();
        let auth_context = expect_context::<AuthContext>();
        let navigate = use_navigate();
        let user_id = user_id.get_untracked().unwrap_or_default();
        move |_| {
            spawn_local({
                let navigate = navigate.clone();
                async move {
                    match backend
                        .delete_account(&auth_context.token(), &user_id)
                        .await
                    {
                        Ok(_) => {
                            show_toast(toaster, format!("Account deleted!"), ToastVariant::Warning);
                            navigate("/", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Account deletion error: {e}"),
                                ToastVariant::Error,
                            );
                        }
                    }
                }
            });
        }
    };

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
                <div
                    node_ref=node_ref
                    class="bg-zinc-900 ring-1 ring-zinc-700 rounded-lg p-6 w-full max-w-md shadow-xl text-gray-200 space-y-4"
                >
                    <h2 class="text-xl font-bold text-red-400">"Confirm Account Deletion"</h2>
                    <p class="text-gray-400 text-sm leading-relaxed">
                        "This action is "
                        <span class="text-red-500 font-semibold">"permanent"</span>
                        ". All your characters, progress, and items will be lost."
                    </p>

                    <ValidatedInput
                        label="Confirm"
                        id="confirm"
                        input_type="text"
                        placeholder="Type DELETE to confirm"
                        bind=confirm_input
                    />

                    <div class="flex justify-between gap-2 pt-2">
                        <MenuButton on:click=move |_| open.set(false)>"Cancel"</MenuButton>
                        <MenuButtonRed
                            on:click=do_delete.clone()
                            disabled=Signal::derive(move || confirm_input.read().is_none())
                        >
                            "Delete"
                        </MenuButtonRed>
                    </div>
                </div>
            </div>
        </Show>
    }
}
