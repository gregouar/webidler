use indexmap::IndexMap;
use leptos::{html::*, prelude::*};

use crate::components::{
    settings::{GraphicsQuality, SettingsContext},
    ui::{
        buttons::{MenuButton, MenuButtonRed, Toggle},
        card::{CardInset, CardInsetTitle, CardTitle, MenuCard},
        dropdown::DropdownMenu,
        list_row::MenuListRow,
        menu_panel::MenuPanel,
    },
};

#[component]
pub fn SettingsModal(open: RwSignal<bool>) -> impl IntoView {
    let settings = expect_context::<SettingsContext>();
    let settings_data = RwSignal::new(settings.read_settings_untracked().clone());

    view! {
        <MenuPanel open=open w_full=false h_full=false class:items-center>
            <MenuCard class="w-xl mx-auto">
                <CardTitle>"Game Settings"</CardTitle>

                <CardInset class="space-y-2 xl:space-y-4">

                    <SettingsSection title="Graphics">
                        <SettingDropdown
                            label="Graphics quality"
                            options=GraphicsQuality::to_options()
                            value=Signal::derive(move || { settings_data.read().graphics_quality })
                            on_change=move |v| {
                                settings_data.write().graphics_quality = v;
                            }
                        />
                        <SettingToggle
                            label="Shake on critical hit"
                            value=Signal::derive(move || { settings_data.read().shake_on_crit })
                            on_toggle=move |v| {
                                settings_data.write().shake_on_crit = v;
                            }
                        />
                    </SettingsSection>

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
                </CardInset>

                <div class="flex justify-between px-4">
                    <MenuButtonRed on:click=move |_| {
                        open.set(false);
                        settings_data.set(settings.read_settings().clone())
                    }>"Cancel"</MenuButtonRed>
                    <MenuButton on:click=move |_| {
                        open.set(false);
                        settings.save_settings(settings_data.get());
                    }>"Confirm"</MenuButton>
                </div>
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn SettingsSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <CardInsetTitle>{title}</CardInsetTitle>
        <div class="space-y-3">{children()}</div>
    }
}

#[component]
fn SettingToggle(
    label: &'static str,
    value: Signal<bool>,
    on_toggle: impl Fn(bool) + Send + 'static,
) -> impl IntoView {
    view! {
        <MenuListRow>
            <div class="w-full
            flex items-center justify-between
            p-2 px-3 gap-2 xl:gap-4
            ">
                <span class="text-sm font-normal text-white">{label}</span>

                <Toggle initial=value.get_untracked() toggle_callback=move |v| on_toggle(v)>
                    {move || if value.get() { "On" } else { "Off" }}
                </Toggle>
            </div>
        </MenuListRow>
    }
}

#[component]
fn SettingDropdown<T>(
    label: &'static str,
    options: IndexMap<T, String>,
    value: Signal<T>,
    on_change: impl Fn(T) + 'static,
) -> impl IntoView
where
    T: Clone + std::hash::Hash + Eq + Send + Sync + 'static,
{
    let chosen_option = RwSignal::new(value.get_untracked());

    Effect::new(move || on_change(chosen_option.get()));

    view! {
        <MenuListRow class="flex items-center justify-between">
            <div class="w-full
            flex items-center justify-between
            p-2 px-3 gap-2 xl:gap-4
            ">
                <span class="text-sm font-normal text-white">{label}</span>

                <DropdownMenu options chosen_option />
            </div>
        </MenuListRow>
    }
}
