// src/commands/start.rs

use pumpkin_plugin_api::{
    command::{Arg, CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::commands::get_task_manager;
use crate::engine::shape::Shape;
use crate::utils::color::error;
use crate::utils::coord::ChunkCoord;

pub struct StartHandler;

impl CommandHandler for StartHandler {
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
            Arg::Simple(s) => s.parse::<u32>().unwrap_or(500),
            _ => 500,
        };

        let shape: Shape = match args.get_value("shape") {
            Arg::Simple(s) => s.parse::<Shape>().unwrap_or(Shape::Square),
            _ => Shape::Square,
        };

        let Some(world) = server.get_world_by_name(&world_name) else {
            sender.send_message(TextComponent::text(&error(&format!("Мир '{world_name}' не найден! Доступные миры можно посмотреть через Tab."))));
            return Ok(1);
        };

        // Determine center coordinate from world spawn
        let spawn = world.get_spawn_location();
        let center = ChunkCoord::from_block(spawn.pos.x, spawn.pos.z);
        let radius_chunks = ChunkCoord::block_radius_to_chunks(radius_blocks);

        let manager_arc = get_task_manager();
        let mut manager = manager_arc.lock().unwrap_or_else(|e| e.into_inner());

        match manager.start(world_name, center, radius_chunks, shape) {
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
