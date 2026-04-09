use codee::string::JsonSerdeCodec;
use indexmap::IndexMap;
use leptos::prelude::{
    guards::{Plain, ReadGuard},
    *,
};
use leptos_use::storage;
use serde::{Deserialize, Serialize};
use web_sys::Event;

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct SettingsData {
    pub scientific_notation: bool,
    pub always_compare_items: bool,
    pub always_display_affix_tiers: bool,

    #[serde(default)]
    pub graphics_quality: GraphicsQuality,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsQuality {
    #[default]
    High,
    Medium,
    Low,
}

impl GraphicsQuality {
    pub fn to_options() -> IndexMap<GraphicsQuality, String> {
        use GraphicsQuality::*;
        [
            (High, "High".to_string()),
            (Medium, "Medium".to_string()),
            (Low, "Low".to_string()),
        ]
        .into()
    }

    pub fn uses_heavy_effects(self) -> bool {
        matches!(self, Self::High)
    }

    pub fn uses_surface_effects(self) -> bool {
        !matches!(self, Self::Low)
    }

    pub fn uses_textures(self) -> bool {
        !matches!(self, Self::Low)
    }
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

    pub fn graphics_quality(&self) -> GraphicsQuality {
        self.settings_data.read().graphics_quality
    }

    pub fn graphics_quality_untracked(&self) -> GraphicsQuality {
        self.settings_data.read_untracked().graphics_quality
    }

    pub fn uses_heavy_effects(&self) -> bool {
        self.graphics_quality().uses_heavy_effects()
    }

    pub fn uses_surface_effects(&self) -> bool {
        self.graphics_quality().uses_surface_effects()
    }

    pub fn uses_textures(&self) -> bool {
        self.graphics_quality().uses_textures()
    }

    pub fn save_settings(&self, new_settings: SettingsData) {
        let graphics_quality_changed =
            self.settings_data.read_untracked().graphics_quality != new_settings.graphics_quality;
        self.settings_data.set(new_settings.clone());
        self.set_settings.set(new_settings);

        if graphics_quality_changed {
            request_layout_refresh();
        }
    }
}

fn request_layout_refresh() {
    let dispatch_resize = || {
        if let Some(window) = web_sys::window() && let Ok(event) = Event::new("resize") {
            let _ = window.dispatch_event(&event);
        }
    };

    dispatch_resize();

    set_timeout(dispatch_resize, std::time::Duration::from_millis(16));
    set_timeout(dispatch_resize, std::time::Duration::from_millis(80));
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
