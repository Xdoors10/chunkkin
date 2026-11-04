use pumpkin_plugin_api::{
    command::{CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::commands::get_task_manager;
use crate::i18n::{tr, Msg};
use crate::utils::color::{info, PREFIX};

/// `/chunkkin status` — shows the current task report without modifying state.
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

/// `/chunkkin progress` (or `/chunkkin status toggle`) — toggles periodic chat progress messages on/off.
pub struct ProgressHandler;

impl CommandHandler for ProgressHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let manager_arc = get_task_manager();
        let mut manager = manager_arc.lock().unwrap_or_else(|e| e.into_inner());

        let enabled = manager.toggle_chat_progress();
        let toggle_msg = tr(if enabled {
            Msg::StatusProgressEnabled
        } else {
            Msg::StatusProgressDisabled
        });
        sender.send_message(TextComponent::text(&info(toggle_msg)));

        tracing::debug!("{PREFIX}chat progress toggled: {enabled}");
        Ok(0)
    }
}
