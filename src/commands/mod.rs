pub mod args;
pub mod cancel;
pub mod config;
pub mod help;
pub mod pause;
pub mod reload;
pub mod resume;
pub mod selection;
pub mod start;
pub mod status;

use std::sync::{Arc, Mutex};
use pumpkin_plugin_api::{
    command::{ArgumentType, Command, CommandNode, StringType},
    permission::{Permission, PermissionDefault},
    Context,
};
use crate::task::TaskManager;
use cancel::CancelHandler;
use help::HelpHandler;
use pause::PauseHandler;
use reload::ReloadHandler;
use resume::ResumeHandler;
use selection::{CenterHandler, RadiusHandler, SelectionHandler, WorldHandler};
use start::StartHandler;
use status::{ProgressHandler, StatusHandler};

static TASK_MANAGER: Mutex<Option<Arc<Mutex<TaskManager>>>> = Mutex::new(None);

/// Initializes the global task manager reference.
pub fn init_task_manager(manager: Arc<Mutex<TaskManager>>) {
    *TASK_MANAGER.lock().unwrap_or_else(|e| e.into_inner()) = Some(manager);
}

/// Returns the thread-safe global task manager.
#[must_use]
pub fn get_task_manager() -> Arc<Mutex<TaskManager>> {
    TASK_MANAGER
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .expect("TaskManager must be initialized in on_load")
}

/// Registers the complete `/chunkkin` command tree with tab completions and permissions.
pub fn register_commands(context: &Context) {
    let aliases = ["chunkkin".to_string(), "ck".to_string()];

    let word_type = ArgumentType::String(StringType::SingleWord);
    let int_type = ArgumentType::Integer((Some(1), None));

    // Task management
    let start_tree = CommandNode::literal("start").execute(StartHandler);

    // Selection: world
    // NOTE: no suggestion provider here — a bug in Pumpkin's WASM host panics on
    // the rayon simulation thread when tab-completion queries plugin suggestions
    // (crash: "there is no reactor running" in pumpkin-wasm-host-v0_1
    // commands/executor.rs). Re-add `.suggest(...)` only after the core fix.
    let world_tree = CommandNode::literal("world")
        .then(
            CommandNode::argument("world", &word_type).execute(WorldHandler),
        )
        .execute(WorldHandler);

    // Selection: center [x z]
    let center_tree = CommandNode::literal("center")
        .then(
            CommandNode::argument("center", &ArgumentType::Position2d).execute(CenterHandler),
        )
        .execute(CenterHandler);

    // Selection: radius <radius> [radius_z]
    let radius_tree = CommandNode::literal("radius")
        .then(
            CommandNode::argument("radius", &int_type)
                .then(
                    CommandNode::argument("radius_z", &int_type).execute(RadiusHandler),
                )
                .execute(RadiusHandler),
        )
        .execute(RadiusHandler);

    // Selection helpers
    let selection_tree = CommandNode::literal("selection").execute(SelectionHandler);

    let status_tree = CommandNode::literal("status")
        .then(CommandNode::literal("toggle").execute(ProgressHandler))
        .execute(StatusHandler);

    let progress_tree = CommandNode::literal("progress").execute(ProgressHandler);

    let command = Command::new(&aliases, "World pre-generation control command")
        .then(start_tree)
        .then(CommandNode::literal("pause").execute(PauseHandler))
        .then(CommandNode::literal("continue").execute(ResumeHandler))
        .then(CommandNode::literal("cancel").execute(CancelHandler))
        .then(status_tree)
        .then(progress_tree)
        .then(world_tree)
        .then(center_tree)
        .then(radius_tree)
        .then(selection_tree)
        .then(CommandNode::literal("reload").execute(ReloadHandler))
        .then(CommandNode::literal("help").execute(HelpHandler))
        .execute(HelpHandler);

    let perm = Permission {
        node: "chunkkin:command".to_string(),
        description: "Allows using Chunkkin pre-generation commands".to_string(),
        default: PermissionDefault::Allow,
        children: vec![],
    };
    if let Err(err) = context.register_permission(&perm) {
        tracing::warn!("Failed to register permission chunkkin:command: {err}");
    }

    context.register_command(command, "chunkkin:command");
}

