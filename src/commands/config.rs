use std::sync::{Mutex, OnceLock};
use crate::storage::config::PregenConfig;

static CONFIG: OnceLock<Mutex<Option<PregenConfig>>> = OnceLock::new();

fn config_slot() -> &'static Mutex<Option<PregenConfig>> {
    CONFIG.get_or_init(|| Mutex::new(None))
}

/// Initializes the global config reference from the data folder.
pub fn init_config(data_folder: &str) -> PregenConfig {
    let config = PregenConfig::load_or_default(data_folder);
    *config_slot()
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(config.clone());
    config
}

/// Re-reads `config.json` from disk and swaps the in-memory config.
///
/// Language is re-applied immediately. Returns the freshly loaded config, or an
/// error when the file could not be read; in that case the old config stays active.
pub fn reload_config() -> Result<PregenConfig, String> {
    let data_folder = config_data_folder();
    if data_folder.is_empty() {
        return Err(crate::i18n::tr(crate::i18n::Msg::ConfigNotInitialized).into());
    }

    // Warn about overwriting hand edits while a task is running
    let config = PregenConfig::load_or_default(&data_folder);

    *config_slot()
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(config.clone());

    Ok(config)
}

/// Applies `f` to the loaded config and persists it to disk.
///
/// Returns the value produced by `f`, or an error when the config write failed.
pub fn with_config<T>(f: impl FnOnce(&mut PregenConfig) -> T) -> Result<T, String> {
    let mut guard = config_slot()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let Some(config) = guard.as_mut() else {
        return Err(crate::i18n::tr(crate::i18n::Msg::ConfigNotInitialized).into());
    };

    let result = f(config);

    let data_folder = config_data_folder();
    config.save(&data_folder)?;

    Ok(result)
}

/// Returns the plugin data folder captured at init time.
#[must_use]
pub fn config_data_folder() -> String {
    DATA_FOLDER
        .get()
        .map_or_else(String::new, std::clone::Clone::clone)
}

static DATA_FOLDER: OnceLock<String> = OnceLock::new();

/// Captures the data folder path for later config saves.
pub fn set_data_folder(folder: &str) {
    let _ = DATA_FOLDER.set(folder.to_string());
}
