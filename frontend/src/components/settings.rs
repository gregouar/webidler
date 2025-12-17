use codee::string::JsonSerdeCodec;
use leptos::prelude::{
    guards::{Plain, ReadGuard},
    *,
};
use leptos_use::storage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct SettingsData {
    pub scientific_notation: bool,
    pub always_compare_items: bool,
    pub always_display_affix_tiers: bool,
}

#[derive(Clone, Copy)]
pub struct SettingsContext {
    settings_data: RwSignal<SettingsData>,
    set_settings: WriteSignal<SettingsData>,
}

impl SettingsContext {
    pub fn read_settings(&self) -> ReadGuard<SettingsData, Plain<SettingsData>> {
        self.settings_data.read()
    }

    pub fn read_settings_untracked(&self) -> ReadGuard<SettingsData, Plain<SettingsData>> {
        self.settings_data.read_untracked()
    }

    pub fn save_settings(&self, new_settings: SettingsData) {
        self.settings_data.set(new_settings.clone());
        self.set_settings.set(new_settings);
    }
}

pub fn provide_settings_context() {
    let (get_settings, set_settings, _) =
        storage::use_local_storage::<SettingsData, JsonSerdeCodec>("settings");
    let settings_data = RwSignal::new(get_settings.get_untracked());

    provide_context(SettingsContext {
        settings_data,
        set_settings,
    });
}
