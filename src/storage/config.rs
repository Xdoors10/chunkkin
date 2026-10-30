use serde::{Deserialize, Serialize};
use std::fs;
use crate::i18n::{self, Lang};

/// The persistent pre-generation selection (target world, center, radii),
/// equivalent to Chunky's selection commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    /// Target world name.
    pub world: String,
    /// Center block X coordinate.
    pub center_x: i32,
    /// Center block Z coordinate.
    pub center_z: i32,
    /// Radius along the X axis, in blocks.
    pub radius_x: u32,
    /// Radius along the Z axis, in blocks.
    pub radius_z: u32,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            world: "world".into(),
            center_x: 0,
            center_z: 0,
            radius_x: 500,
            radius_z: 500,
        }
    }
}

/// Default `language` config value.
fn default_language() -> String {
    "en".into()
}

fn default_batch_size() -> usize {
    16
}

fn default_log_interval_ticks() -> u64 {
    100
}

fn default_auto_save_interval_chunks() -> u64 {
    1000
}

fn default_auto_resume_on_startup() -> bool {
    true
}

fn default_console_log() -> bool {
    true
}

impl Selection {
    /// Chunk-space radii, rounded up.
    #[must_use]
    pub fn radius_chunks(&self) -> (u32, u32) {
        (
            crate::utils::coord::ChunkCoord::block_radius_to_chunks(self.radius_x),
            crate::utils::coord::ChunkCoord::block_radius_to_chunks(self.radius_z),
        )
    }
}

/// Plugin configuration loaded from the private WASI data folder (`config.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenConfig {
    /// Plugin message language: `"en"` (default) or `"ru"`.
    #[serde(default = "default_language")]
    pub language: String,
    /// Current pre-generation selection, persisted across restarts.
    #[serde(default)]
    pub selection: Selection,
    /// Number of chunks to process in a single tick.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Interval in server ticks between progress messages (20 ticks = 1 second).
    #[serde(default = "default_log_interval_ticks")]
    pub log_interval_ticks: u64,
    /// How many generated chunks trigger a world save to disk.
    #[serde(default = "default_auto_save_interval_chunks")]
    pub auto_save_interval_chunks: u64,
    /// Whether to automatically resume an interrupted task upon server start.
    #[serde(default = "default_auto_resume_on_startup")]
    pub auto_resume_on_startup: bool,
    /// Whether generation progress is also written to the server console.
    #[serde(default = "default_console_log")]
    pub console_log: bool,
}

impl Default for PregenConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            selection: Selection::default(),
            batch_size: 16,
            log_interval_ticks: 100,
            auto_save_interval_chunks: 1000,
            auto_resume_on_startup: true,
            console_log: default_console_log(),
        }
    }
}

impl PregenConfig {
    /// Loads the configuration from the given data folder, or writes the default if missing.
    ///
    /// Also activates the configured language for [`crate::i18n`].
    pub fn load_or_default(data_folder: &str) -> Self {
        let config_path = format!("{data_folder}/config.json");

        if let Ok(content) = fs::read_to_string(&config_path) {
            match serde_json::from_str::<Self>(&content) {
                Ok(cfg) => {
                    let raw = cfg.language.trim();
                    let lang = Lang::parse(raw);
                    if lang == Lang::En
                        && !raw.is_empty()
                        && !raw.eq_ignore_ascii_case("en")
                        && !raw.eq_ignore_ascii_case("english")
                    {
                        tracing::warn!(
                            "{}",
                            i18n::trf(i18n::Msg::ConfigLanguageUnknown, &[&cfg.language])
                        );
                    }
                    i18n::set_lang(lang);
                    return cfg;
                }
                Err(err) => {
                    tracing::warn!("{}", i18n::trf(i18n::Msg::ConfigParseError, &[&err]));
                }
            }
        }

        let default_config = Self::default();
        let _ = fs::create_dir_all(data_folder);
        if let Ok(json) = serde_json::to_string_pretty(&default_config) {
            let _ = fs::write(&config_path, json);
        }

        i18n::set_lang(Lang::En);
        default_config
    }

    /// Saves the current configuration to disk.
    pub fn save(&self, data_folder: &str) -> Result<(), String> {
        let _ = fs::create_dir_all(data_folder);
        let config_path = format!("{data_folder}/config.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| i18n::trf(i18n::Msg::ConfigSerializeError, &[&e]))?;

        fs::write(&config_path, json)
            .map_err(|e| i18n::trf(i18n::Msg::ConfigWriteError, &[&config_path, &e]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serde_roundtrip() {
        let cfg = PregenConfig::default();
        let json = serde_json::to_string_pretty(&cfg).expect("serialize");
        let deserialized: PregenConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(cfg.selection.world, deserialized.selection.world);
        assert_eq!(cfg.selection.center_x, deserialized.selection.center_x);
        assert_eq!(cfg.selection.radius_x, deserialized.selection.radius_x);
        assert_eq!(cfg.language, deserialized.language);
        assert_eq!(cfg.batch_size, deserialized.batch_size);
    }

    #[test]
    fn test_language_defaults_and_parses() {
        // Language field defaults to English when absent (legacy config)
        let legacy = r#"{"batch_size": 16}"#;
        let cfg: PregenConfig = serde_json::from_str(legacy).expect("deserialize");
        assert_eq!(cfg.language, "en");
    }

    #[test]
    fn test_config_without_selection_uses_default() {
        // Old config.json files written before the selection feature have no
        // `selection` field and must still load with the default selection.
        let legacy = r#"{
            "default_radius": 500,
            "batch_size": 16,
            "log_interval_ticks": 100,
            "auto_save_interval_chunks": 1000,
            "auto_resume_on_startup": true
        }"#;
        let cfg: PregenConfig = serde_json::from_str(legacy).expect("deserialize legacy");
        assert_eq!(cfg.selection, Selection::default());
    }

    #[test]
    fn test_selection_radius_chunks() {
        let sel = Selection {
            radius_x: 500,
            radius_z: 1000,
            ..Selection::default()
        };
        assert_eq!(sel.radius_chunks(), (32, 63));
    }
}
