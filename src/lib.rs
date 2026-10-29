// src/lib.rs

pub mod commands;
pub mod engine;
pub mod storage;
pub mod task;
pub mod utils;

use std::sync::{Arc, Mutex};
use pumpkin_plugin_api::{
    permissions,
    scheduler::SchedulerExt,
    Context, Plugin, PluginMetadata, register_plugin,
};
use storage::{PregenConfig, StateStore};
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
        let data_folder = context.get_data_folder();
        let config = PregenConfig::load_or_default(&data_folder);

        let mut task_manager = TaskManager::new(
            data_folder.clone(),
            config.batch_size,
            config.log_interval_ticks,
            config.auto_save_interval_chunks,
        );

        // Check if there is an interrupted/paused task from previous session
        if let Some(saved) = StateStore::load(&data_folder) {
            if saved.status.is_active() {
                task_manager.restore_saved_state(saved, config.auto_resume_on_startup);
            }
        }

        let manager_arc = Arc::new(Mutex::new(task_manager));
        commands::init_task_manager(Arc::clone(&manager_arc));
        commands::register_commands(&context);

        // Schedule background generation tick (runs every tick = 50ms)
        let tick_manager = Arc::clone(&manager_arc);
        context.schedule_repeating_task(1, 1, move |server| {
            let mut mgr = tick_manager.lock().unwrap_or_else(|e| e.into_inner());
            mgr.tick(&server);
        });

        tracing::info!("World pre-generation plugin initialized successfully!");
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<(), String> {
        let manager_arc = commands::get_task_manager();
        let manager = manager_arc.lock().unwrap_or_else(|e| e.into_inner());
        manager.persist_current_state();

        tracing::info!("World pre-generation plugin unloaded.");
        Ok(())
    }
}

register_plugin!(PregenPlugin);
