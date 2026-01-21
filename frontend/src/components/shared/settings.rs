use leptos::{html::*, prelude::*};

use crate::components::{
    settings::SettingsContext,
    ui::{
        buttons::{MenuButton, MenuButtonRed, Toggle},
        menu_panel::MenuPanel,
    },
};

#[component]
pub fn SettingsModal(open: RwSignal<bool>) -> impl IntoView {
    let settings = expect_context::<SettingsContext>();
    let settings_data = RwSignal::new(settings.read_settings_untracked().clone());

    view! {
        <MenuPanel open=open w_full=false>
            <div class="flex items-center justify-center p-4 max-h-full">
                <div class="
                bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl
                p-6 sm:p-8 space-y-6 w-full max-w-xl mx-auto
                ">
                    <h2 class="text-2xl font-bold text-amber-300 text-center">"Game Settings"</h2>

                    <SettingsSection title="Numbers">
                        <SettingToggle
                            label="Scientific notation"
                            value=Signal::derive(move || {
                                settings_data.read().scientific_notation
                            })
                            on_toggle=move |v| {
                                settings_data.write().scientific_notation = v;
                            }
                        />
                    </SettingsSection>

                    <SettingsSection title="Items">
                        <SettingToggle
                            label="Always compare items on hover"
                            value=Signal::derive(move || {
                                settings_data.read().always_compare_items
                            })
                            on_toggle=move |v| {
                                settings_data.write().always_compare_items = v;
                            }
                        />

                        <SettingToggle
                            label="Always display item affix tiers"
                            value=Signal::derive(move || {
                                settings_data.read().always_display_affix_tiers
                            })
                            on_toggle=move |v| {
                                settings_data.write().always_display_affix_tiers = v;
                            }
                        />
                    </SettingsSection>

                    <div class="flex justify-between pt-4 border-t border-zinc-700">
                        <MenuButtonRed on:click=move |_| {
                            open.set(false);
                            settings_data.set(settings.read_settings().clone())
                        }>"Cancel"</MenuButtonRed>
                        <MenuButton on:click=move |_| {
                            open.set(false);
                            settings.save_settings(settings_data.get());
                        }>"Confirm"</MenuButton>
                    </div>
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
fn SettingsSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <h3 class="text-lg font-semibold text-amber-200">{title}</h3>
            <div class="space-y-3">{children()}</div>
        </div>
    }
}

#[component]
fn SettingToggle(
    label: &'static str,
    value: Signal<bool>,
    on_toggle: impl Fn(bool) + 'static,
) -> impl IntoView {
    view! {
        <div class="
        flex items-center justify-between
        bg-zinc-800 border border-zinc-700 rounded-lg p-2 px-3
        ">
            <span class="text-sm font-normal text-white">{label}</span>

            <Toggle initial=value.get_untracked() toggle_callback=move |v| on_toggle(v)>
                {move || if value.get() { "On" } else { "Off" }}
            </Toggle>
        </div>
    }
}
