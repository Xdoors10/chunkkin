// src/commands/status.rs

use pumpkin_plugin_api::{
    command::{CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::commands::get_task_manager;

pub struct StatusHandler;

impl CommandHandler for StatusHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let manager_arc = get_task_manager();
        let manager = manager_arc.lock().unwrap_or_else(|e| e.into_inner());

        let report = manager.status_report();
        sender.send_message(TextComponent::text(&report));
        Ok(0)
    }
}
