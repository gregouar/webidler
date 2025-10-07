use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::http::client::SignUpRequest;

use crate::components::{
    backend_client::BackendClient,
    captcha::Captcha,
    pages::{privacy::PrivacyContent, terms::TermsContent},
    ui::{
        buttons::MenuButton,
        input::{Input, ValidatedInput},
        toast::*,
    },
};

#[component]
pub fn AccountSettingsPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-zinc-950 via-zinc-900 to-zinc-800 text-gray-200 p-6">
            <div class="w-full max-w-lg bg-zinc-800/80 backdrop-blur-sm rounded-lg shadow-lg p-6 space-y-6 ring-1 ring-zinc-700">
                <h1 class="text-2xl font-bold text-amber-300 mb-2">"Account Settings"</h1>

                // Username
                <div class="space-y-2">
                    <label class="text-sm font-semibold text-gray-300">"Username"</label>
                    <input
                        type="text"
                        class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2 text-gray-200 focus:border-amber-400 focus:ring focus:ring-amber-400/20"
                        prop:value="CurrentUsername"
                    />
                    <MenuButton text="Save Username" on:click=move |_| {} />
                </div>

                // Email
                <div class="space-y-2">
                    <label class="text-sm font-semibold text-gray-300">"Email"</label>
                    <input
                        type="email"
                        class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2 text-gray-200 focus:border-amber-400 focus:ring focus:ring-amber-400/20"
                        prop:value="user@example.com"
                    />
                    <MenuButton text="Save Email" on:click=move |_| {} />
                </div>

                // Password update
                <div class="space-y-2">
                    <label class="text-sm font-semibold text-gray-300">"Change Password"</label>
                    <input
                        type="password"
                        placeholder="Current password"
                        class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2"
                    />
                    <input
                        type="password"
                        placeholder="New password"
                        class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2"
                    />
                    <input
                        type="password"
                        placeholder="Confirm new password"
                        class="w-full rounded-md bg-zinc-900 border border-zinc-700 p-2"
                    />
                    <MenuButton text="Update Password" on:click=move |_| {} />
                </div>

                // Danger zone
                <div class="border-t border-zinc-700 pt-4 mt-4 space-y-2">
                    <label class="text-sm font-semibold text-red-400">"Danger Zone"</label>
                    <p class="text-sm text-gray-400">
                        "Deleting your account is irreversible. All game progress will be lost."
                    </p>
                    <MenuButton
                        text="Delete Account"
                        class="bg-red-700 hover:bg-red-800 text-white"
                        on:click=move |_| {}
                    />
                </div>

                // Footer
                <div class="pt-4 border-t border-zinc-700">
                    <MenuButton text="Back to Dashboard" on:click=move |_| {} />
                </div>
            </div>
        </div>
    }
}
