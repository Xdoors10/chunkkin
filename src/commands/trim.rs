// src/commands/trim.rs

use pumpkin_plugin_api::{
    command::{Arg, CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::utils::color::{error, highlight, info, PREFIX};
use crate::utils::coord::ChunkCoord;

pub struct TrimHandler;

impl CommandHandler for TrimHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let world_name = match args.get_value("world") {
            Arg::Simple(s) => s,
            _ => "world".to_string(),
        };

        let radius_blocks: u32 = match args.get_value("radius") {
            Arg::Simple(s) => s.parse::<u32>().unwrap_or(1000),
            _ => 1000,
        };

        let Some(world) = server.get_world_by_name(&world_name) else {
            sender.send_message(TextComponent::text(&error(&format!("Мир '{world_name}' не найден!"))));
            return Ok(1);
        };

        let radius_chunks = ChunkCoord::block_radius_to_chunks(radius_blocks);
        let border = world.get_border();
        let border_size = border.get_size();

        sender.send_message(TextComponent::text(&format!(
            "{PREFIX}Анализ границ мира {}:\n{}\n{}\n{}",
            world_name,
            highlight("Заданный радиус", &format!("{radius_blocks} блоков ({radius_chunks} чанков)")),
            highlight("Размер границы мира", &format!("{border_size:.0} блоков")),
            info("Чанки вне границы защищены от автоматической генерации.")
        )));

        Ok(0)
    }
}
