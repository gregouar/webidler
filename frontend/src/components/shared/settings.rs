use leptos::{html::*, prelude::*};

use crate::components::{
    settings::SettingsContext,
    ui::{
        buttons::{MenuButton, MenuButtonRed, Toggle},
        card::{Card, CardInset, CardTitle},
        menu_panel::MenuPanel,
    },
};

#[component]
pub fn SettingsModal(open: RwSignal<bool>) -> impl IntoView {
    let settings = expect_context::<SettingsContext>();
    let settings_data = RwSignal::new(settings.read_settings_untracked().clone());

    view! {
        <MenuPanel open=open w_full=false h_full=false class:items-center>
            <Card class="w-xl mx-auto">
                <CardTitle>"Game Settings"</CardTitle>

                <SettingsSection title="Numbers">
                    <SettingToggle
                        label="Scientific notation"
                        value=Signal::derive(move || { settings_data.read().scientific_notation })
                        on_toggle=move |v| {
                            settings_data.write().scientific_notation = v;
                        }
                    />
                </SettingsSection>

                <SettingsSection title="Items">
                    <SettingToggle
                        label="Always compare items on hover"
                        value=Signal::derive(move || { settings_data.read().always_compare_items })
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
            </Card>
        </MenuPanel>
    }
}

#[component]
fn SettingsSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <CardInset class="space-y-2 xl:space-y-4">
            <h3 class="text-base xl:text-lg font-semibold text-amber-300">{title}</h3>
            <div class="space-y-3">{children()}</div>
        </CardInset>
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
        gap-2 xl:gap-4
        ">
            <span class="text-sm font-normal text-white">{label}</span>

            <Toggle initial=value.get_untracked() toggle_callback=move |v| on_toggle(v)>
                {move || if value.get() { "On" } else { "Off" }}
            </Toggle>
        </div>
    }
}
