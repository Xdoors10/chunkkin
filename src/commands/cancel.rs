use pumpkin_plugin_api::{
    command::{CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::commands::get_task_manager;
use crate::utils::color::error;

pub struct CancelHandler;

impl CommandHandler for CancelHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let manager_arc = get_task_manager();
        let mut manager = manager_arc.lock().unwrap_or_else(|e| e.into_inner());

        match manager.cancel() {
            Ok(msg) => {
                sender.send_message(TextComponent::text(&msg));
                Ok(0)
            }
            Err(err) => {
                sender.send_message(TextComponent::text(&error(&err)));
                Ok(1)
            }
        }
    }
}
