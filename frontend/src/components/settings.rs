use codee::string::JsonSerdeCodec;
use indexmap::IndexMap;
use leptos::prelude::{
    guards::{Plain, ReadGuard},
    *,
};
use leptos_use::storage;
use serde::{Deserialize, Serialize};
use web_sys::Event;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct SettingsData {
    pub scientific_notation: bool,
    pub always_compare_items: bool,
    pub always_display_affix_tiers: bool,

    #[serde(default)]
    pub graphics_quality: GraphicsQuality,
    #[serde(default = "default_true")]
    pub enable_animations: bool,
    #[serde(default = "default_true")]
    pub shake_on_crit: bool,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            scientific_notation: false,
            always_compare_items: false,
            always_display_affix_tiers: false,
            graphics_quality: Default::default(),
            enable_animations: true,
            shake_on_crit: true,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsQuality {
    High,
    #[default]
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

    pub fn animations_enabled(&self) -> bool {
        self.settings_data.read().enable_animations
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
        if let Some(window) = web_sys::window()
            && let Ok(event) = Event::new("resize")
        {
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

    set_animations_enabled(settings_data.read_untracked().enable_animations);
    Effect::new(move |_| {
        set_animations_enabled(settings_data.read().enable_animations);
    });

    provide_context(SettingsContext {
        settings_data,
        set_settings,
    });
}

fn set_animations_enabled(enabled: bool) {
    let Some(document_element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    else {
        return;
    };

    if enabled {
        let _ = document_element.remove_attribute("data-animations-disabled");
    } else {
        let _ = document_element.set_attribute("data-animations-disabled", "");
    }
}

fn default_true() -> bool {
    true
}
