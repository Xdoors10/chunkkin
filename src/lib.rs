// src/lib.rs

pub mod commands;
pub mod engine;
pub mod task;
pub mod utils;

use std::sync::{Arc, Mutex};
use pumpkin_plugin_api::{
    scheduler::SchedulerExt,
    Context, Plugin, PluginMetadata, register_plugin,
    permissions,
};
use task::TaskManager;

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
        let manager = Arc::new(Mutex::new(TaskManager::new(16, 100, 1000)));
        commands::init_task_manager(Arc::clone(&manager));
        commands::register_commands(&context);

        // Schedule background generation tick (runs every tick = 50ms)
        let manager_clone = Arc::clone(&manager);
        context.schedule_repeating_task(1, 1, move |server| {
            let mut mgr = manager_clone.lock().unwrap_or_else(|e| e.into_inner());
            mgr.tick(&server);
        });

        tracing::info!("World pre-generation plugin initialized successfully!");
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<(), String> {
        tracing::info!("World pre-generation plugin unloaded.");
        Ok(())
    }
}

register_plugin!(PregenPlugin);
