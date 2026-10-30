use pumpkin_plugin_api::{
    command::{CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::commands::config::with_config;
use crate::commands::get_task_manager;
use crate::i18n::{trf, Msg};
use crate::utils::color::error;

pub struct StartHandler;

impl CommandHandler for StartHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let selection = with_config(|config| config.selection.clone());

        let selection = match selection {
            Ok(s) => s,
            Err(e) => {
                sender.send_message(TextComponent::text(&error(&e)));
                return Ok(1);
            }
        };

        // Validate the world still exists on this server
        if server.get_world_by_name(&selection.world).is_none() {
            sender.send_message(TextComponent::text(&error(&trf(
                Msg::StartWorldNotFound,
                &[&selection.world],
            ))));
            return Ok(1);
        }

        let manager_arc = get_task_manager();
        let mut manager = manager_arc.lock().unwrap_or_else(|e| e.into_inner());

        match manager.start(&selection) {
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
