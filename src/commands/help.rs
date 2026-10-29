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
             {GREEN}/chunkkin start [мир] [радиус] [форма]{GRAY} — Запуск генерации чанков\n\
             {GREEN}/chunkkin pause{GRAY} — Приостановка текущей задачи с сохранением\n\
             {GREEN}/chunkkin continue{GRAY} (или {YELLOW}/chunkkin resume{GRAY}) — Возобновление задачи\n\
             {GREEN}/chunkkin cancel{GRAY} (или {YELLOW}/chunkkin stop{GRAY}) — Полная отмена задачи\n\
             {GREEN}/chunkkin status{GRAY} (или {YELLOW}/chunkkin progress{GRAY}) — Детальный прогресс и скорость\n\
             {GREEN}/chunkkin trim [мир] [радиус]{GRAY} — Анализ чанков за границей мира\n\
             {GREEN}/chunkkin help{GRAY} — Данная справка\n\
             {GRAY}Алиас команды: {YELLOW}/ck{RESET}",
        );

        sender.send_message(TextComponent::text(&help_text));
        Ok(0)
    }
}
