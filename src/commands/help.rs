// src/commands/help.rs

use pumpkin_plugin_api::{
    command::{CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::utils::color::{GOLD, GRAY, GREEN, PREFIX, RESET, YELLOW};

pub struct HelpHandler;

impl CommandHandler for HelpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let help_text = format!(
            "{PREFIX}{GOLD}Команды предварительной генерации:{RESET}\n\
             {GREEN}/pregen start [мир] [радиус] [форма]{GRAY} — Запуск генерации чанков\n\
             {GREEN}/pregen pause{GRAY} — Приостановка текущей задачи с сохранением\n\
             {GREEN}/pregen continue{GRAY} (или {YELLOW}/pregen resume{GRAY}) — Возобновление задачи\n\
             {GREEN}/pregen cancel{GRAY} (или {YELLOW}/pregen stop{GRAY}) — Полная отмена задачи\n\
             {GREEN}/pregen status{GRAY} (или {YELLOW}/pregen progress{GRAY}) — Детальный прогресс и скорость\n\
             {GREEN}/pregen trim [мир] [радиус]{GRAY} — Анализ чанков за границей мира\n\
             {GREEN}/pregen help{GRAY} — Данная справка",
        );

        sender.send_message(TextComponent::text(&help_text));
        Ok(0)
    }
}
