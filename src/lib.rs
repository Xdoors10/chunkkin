// src/lib.rs

use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata, register_plugin,
    permissions,
};

pub struct PregenPlugin;

impl Plugin for PregenPlugin {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "pumpkin-pregen".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Dzhibr0ny".into()],
            description: "High-performance world pre-generation plugin for the Pumpkin Minecraft server.".into(),
            dependencies: vec![],
            permissions: vec![
                permissions::FS_READ_DATA.into(),
                permissions::FS_WRITE_DATA.into(),
            ],
        }
    }

    fn on_load(&self, context: Context) -> Result<(), String> {
        tracing::info!("World pre-generation plugin initialized successfully!");
        tracing::info!("Data folder: {}", context.get_data_folder());
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<(), String> {
        tracing::info!("World pre-generation plugin unloaded.");
        Ok(())
    }
}

register_plugin!(PregenPlugin);
