use pumpkin_plugin_api::{
    command::{CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::commands::config::reload_config;
use crate::commands::get_task_manager;
use crate::i18n::{tr, Msg};
use crate::utils::color::{error, info, success, PREFIX};

/// `/chunkkin reload` — re-reads `config.json` from disk and applies it live:
/// language switches immediately, engine settings (batch size, intervals) update
/// on the fly. An active generation task keeps running untouched.
pub struct ReloadHandler;

impl CommandHandler for ReloadHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let config = match reload_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                sender.send_message(TextComponent::text(&error(&e)));
                return Ok(1);
            }
        };

        // Push fresh engine settings into the running task manager
        let task_running = {
            let manager_arc = get_task_manager();
            let mut manager = manager_arc.lock().unwrap_or_else(|e| e.into_inner());
            let running = manager.status.is_running();
            manager.update_runtime_settings(
                config.batch_size,
                config.log_interval_ticks,
                config.auto_save_interval_chunks,
                config.console_log,
            );
            running
        };

        sender.send_message(TextComponent::text(&success(tr(
            Msg::ConfigReloaded,
        ))));

        if task_running {
            sender.send_message(TextComponent::text(&info(tr(
                Msg::ConfigReloadTaskNote,
            ))));
        }

        crate::utils::color::log_console(
            pumpkin_plugin_api::logging::LogLevel::Info,
            &crate::utils::color::strip_colors(&format!("{PREFIX}{}", tr(Msg::ConfigReloaded))),
        );
        Ok(0)
    }
}
