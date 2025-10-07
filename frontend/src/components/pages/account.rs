use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::http::client::SignUpRequest;

use crate::components::{
    backend_client::BackendClient,
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        input::{Input, ValidatedInput},
        toast::*,
    },
};

#[component]
pub fn AccountSettingsPage() -> impl IntoView {
    let navigate_to_dashboard = {
        let navigate = use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    let (get_username_storage, set_username_storage, _) =
        storage::use_local_storage::<Option<_>, JsonSerdeCodec>("username");

    let username = RwSignal::new(get_username_storage.get_untracked());
    let email = RwSignal::new(Some(None));
    let password = RwSignal::new(None);
    let confirm_password = RwSignal::new(None);
    let accepted_terms = RwSignal::new(false);
    let captcha_token = RwSignal::new(None);

    let processing = RwSignal::new(false);
    let passwords_mismatch = Signal::derive(move || password.get() != confirm_password.get());
    let disable_submit = Signal::derive(move || {
        username.read().is_none()
            || password.read().is_none()
            || passwords_mismatch.get()
            || email.read().is_none()
            || !accepted_terms.get()
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
            spawn_local({
                async move {
                    match backend
                        .post_signup(&SignUpRequest {
                            captcha_token: captcha_token.get().unwrap_or_default(),
                            username: username.get().unwrap(),
                            email: email.get().unwrap(),
                            password: password.get().unwrap(),
                            accepted_terms: accepted_terms.get(),
                        })
                        .await
                    {
                        Ok(_) => {
                            // Or directly signin and go to user dashboard?
                            // set_jwt_storage.set(response.jwt);
                            set_username_storage.set(username.get());
                            navigate("/", Default::default());
                        }
                        Err(e) => {
                            show_toast(
                                toaster,
                                format!("Registration error: {e}"),
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
        <main class="my-0 mx-auto max-w-2xl text-center flex flex-col justify-center p-6 gap-4">
            <h1 class="text-amber-200 text-4xl font-extrabold mb-6">"Account Settings"</h1>

            <div class="space-y-4 text-left text-white">
                <div class="space-y-2">
                    <ValidatedInput
                        label="Username"
                        id="username"
                        input_type="text"
                        placeholder="Enter your username"
                        bind=username
                    />
                    <MenuButton on:click=move |_| {}>"Change Username"</MenuButton>
                </div>

                <div class="space-y-2">
                    <ValidatedInput
                        label="Email recovery"
                        id="email"
                        input_type="text"
                        placeholder="Optionally enter your email for password recovery"
                        bind=email
                    />
                    <MenuButton on:click=move |_| {}>"Change Email"</MenuButton>
                </div>

                <div class="space-y-2">
                    <ValidatedInput
                        label="Old Password"
                        id="old-password"
                        input_type="password"
                        placeholder="Enter your old password"
                        bind=password
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
                    <MenuButton on:click=move |_| {}>"Change Password"</MenuButton>
                </div>

                <MenuButton class:w-full on:click=on_submit disabled=disable_submit>
                    "Confirm"
                </MenuButton>
            </div>

            <div class="border-t border-zinc-700 pt-4 mt-4 space-y-2">
                <label class="text-sm font-semibold text-red-400">"Danger Zone"</label>
                <p class="text-sm text-gray-400">
                    "Deleting your account is irreversible. All game progress will be lost."
                </p>
                <MenuButtonRed on:click=move |_| {}>"Delete Account"</MenuButtonRed>
            </div>

            <div>
                <MenuButton on:click=navigate_to_dashboard>"Back"</MenuButton>
            </div>
        </main>
    }

    // view! {
    //     <main class="my-0 mx-auto max-w-2xl text-center flex flex-col justify-center p-6">
    //         <h1 class="text-amber-200 text-4xl font-extrabold mb-6">"Account Settings"</h1>
    //         <div class="w-full max-w-lg bg-zinc-800/80 backdrop-blur-sm rounded-lg shadow-lg p-6 space-y-6 ring-1 ring-zinc-700">

    //             // Username
    //             <div class="space-y-2">
    //                 <label class="text-sm font-semibold text-gray-300">"Username"</label>
    //                 <input
    //                     type="text"
    //                     class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2 text-gray-200 focus:border-amber-400 focus:ring focus:ring-amber-400/20"
    //                     prop:value="CurrentUsername"
    //                 />
    //                 <MenuButton on:click=move |_| {}>"Save Username"</MenuButton>
    //             </div>

    //             // Email
    //             <div class="space-y-2">
    //                 <label class="text-sm font-semibold text-gray-300">"Email"</label>
    //                 <input
    //                     type="email"
    //                     class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2 text-gray-200 focus:border-amber-400 focus:ring focus:ring-amber-400/20"
    //                     prop:value="user@example.com"
    //                 />
    //                 <MenuButton on:click=move |_| {}>"Save Email"</MenuButton>
    //             </div>

    //             // Password update
    //             <div class="space-y-2">
    //                 <label class="text-sm font-semibold text-gray-300">"Change Password"</label>
    //                 <input
    //                     type="password"
    //                     placeholder="Current password"
    //                     class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2"
    //                 />
    //                 <input
    //                     type="password"
    //                     placeholder="New password"
    //                     class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2"
    //                 />
    //                 <input
    //                     type="password"
    //                     placeholder="Confirm new password"
    //                     class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2"
    //                 />
    //                 <MenuButton on:click=move |_| {}>"Update Password"</MenuButton>
    //             </div>

    //             // Danger zone
    //             <div class="border-t border-zinc-700 pt-4 mt-4 space-y-2">
    //                 <label class="text-sm font-semibold text-red-400">"Danger Zone"</label>
    //                 <p class="text-sm text-gray-400">
    //                     "Deleting your account is irreversible. All game progress will be lost."
    //                 </p>
    //                 <MenuButtonRed on:click=move |_| {}>"Delete Account"</MenuButtonRed>
    //             </div>

    //             // Footer
    //             <div class="pt-4 border-t border-zinc-700">
    //                 <MenuButton on:click=move |_| {}>"Back to Dashboard"</MenuButton>
    //             </div>
    //         </div>
    //     </main>
    // }
}
