// src/storage/config.rs

use serde::{Deserialize, Serialize};
use std::fs;
use crate::engine::shape::Shape;

/// Plugin configuration loaded from the private WASI data folder (`config.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenConfig {
    /// Default radius in blocks when not provided in command arguments.
    pub default_radius: u32,
    /// Default geometric shape (square or circle).
    pub default_shape: Shape,
    /// Number of chunks to process in a single tick.
    pub batch_size: usize,
    /// Interval in server ticks between progress messages (20 ticks = 1 second).
    pub log_interval_ticks: u64,
    /// How many generated chunks trigger a world save to disk.
    pub auto_save_interval_chunks: u64,
    /// Whether to automatically resume an interrupted task upon server start.
    pub auto_resume_on_startup: bool,
}

impl Default for PregenConfig {
    fn default() -> Self {
        Self {
            default_radius: 500,
            default_shape: Shape::Square,
            batch_size: 16,
            log_interval_ticks: 100,
            auto_save_interval_chunks: 1000,
            auto_resume_on_startup: true,
        }
    }
}

impl PregenConfig {
    /// Loads the configuration from the given data folder, or writes the default if missing.
    pub fn load_or_default(data_folder: &str) -> Self {
        let config_path = format!("{data_folder}/config.json");

        if let Ok(content) = fs::read_to_string(&config_path) {
            match serde_json::from_str::<Self>(&content) {
                Ok(cfg) => return cfg,
                Err(err) => {
                    tracing::warn!("Ошибка парсинга config.json: {err}. Используются настройки по умолчанию.");
                }
            }
        }

        let default_config = Self::default();
        let _ = fs::create_dir_all(data_folder);
        if let Ok(json) = serde_json::to_string_pretty(&default_config) {
            let _ = fs::write(&config_path, json);
        }

        default_config
    }

    /// Saves the current configuration to disk.
    pub fn save(&self, data_folder: &str) -> Result<(), String> {
        let _ = fs::create_dir_all(data_folder);
        let config_path = format!("{data_folder}/config.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Ошибка сериализации конфига: {e}"))?;

        fs::write(&config_path, json)
            .map_err(|e| format!("Ошибка записи файла {config_path}: {e}"))
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

        assert_eq!(cfg.default_radius, deserialized.default_radius);
        assert_eq!(cfg.default_shape, deserialized.default_shape);
        assert_eq!(cfg.batch_size, deserialized.batch_size);
    }
}
