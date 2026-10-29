// src/commands/mod.rs

pub mod cancel;
pub mod help;
pub mod pause;
pub mod resume;
pub mod start;
pub mod status;
pub mod suggestions;
pub mod trim;

use std::sync::{Arc, Mutex};
use pumpkin_plugin_api::{
    command::{ArgumentType, Command, CommandNode, StringType},
    Context,
};
use crate::task::TaskManager;
use cancel::CancelHandler;
use help::HelpHandler;
use pause::PauseHandler;
use resume::ResumeHandler;
use start::StartHandler;
use status::StatusHandler;
use suggestions::{ShapeSuggestions, WorldSuggestions};
use trim::TrimHandler;

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
    let aliases = ["chunkkin".to_string(), "pregen".to_string(), "ck".to_string()];

    let word_type = ArgumentType::String(StringType::SingleWord);
    let int_type = ArgumentType::Integer((Some(1), None));

    let start_tree = CommandNode::literal("start")
        .then(
            CommandNode::argument("world", &word_type)
                .suggest(WorldSuggestions)
                .then(
                    CommandNode::argument("radius", &int_type).then(
                        CommandNode::argument("shape", &word_type)
                            .suggest(ShapeSuggestions)
                            .execute(StartHandler),
                    )
                    .execute(StartHandler),
                )
                .execute(StartHandler),
        )
        .execute(StartHandler);

    let trim_tree = CommandNode::literal("trim")
        .then(
            CommandNode::argument("world", &word_type)
                .suggest(WorldSuggestions)
                .then(
                    CommandNode::argument("radius", &int_type)
                        .execute(TrimHandler),
                )
                .execute(TrimHandler),
        )
        .execute(TrimHandler);

    let command = Command::new(&aliases, "World pre-generation control command")
        .then(start_tree)
        .then(CommandNode::literal("pause").execute(PauseHandler))
        .then(CommandNode::literal("continue").execute(ResumeHandler))
        .then(CommandNode::literal("resume").execute(ResumeHandler))
        .then(CommandNode::literal("cancel").execute(CancelHandler))
        .then(CommandNode::literal("stop").execute(CancelHandler))
        .then(CommandNode::literal("status").execute(StatusHandler))
        .then(CommandNode::literal("progress").execute(StatusHandler))
        .then(trim_tree)
        .then(CommandNode::literal("help").execute(HelpHandler))
        .execute(HelpHandler);

    context.register_command(command, "pumpkin.command.chunkkin");
}
