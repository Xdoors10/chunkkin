// src/commands/suggestions.rs

use pumpkin_plugin_api::{
    command::{CommandSender, CommandSuggestion, CommandSuggestions, SuggestionRequest},
    commands::CommandSuggestionHandler,
    Server,
    text::TextComponent,
};

/// Suggests available world names currently loaded on the server.
pub struct WorldSuggestions;

impl CommandSuggestionHandler for WorldSuggestions {
    fn suggest(
        &self,
        _sender: CommandSender,
        server: Server,
        request: SuggestionRequest,
    ) -> CommandSuggestions {
        let query = request.remaining.to_lowercase();
        let worlds = server.get_all_worlds();
        let mut values = Vec::new();

        for w in worlds {
            let name = w.get_name();
            if name.to_lowercase().starts_with(&query) {
                values.push(CommandSuggestion {
                    value: name.clone(),
                    tooltip: Some(TextComponent::text(&format!("Мир: {name}"))),
                });
            }
        }

        // Always provide a fallback default if no worlds matched
        if values.is_empty() && "world".starts_with(&query) {
            values.push(CommandSuggestion {
                value: "world".to_string(),
                tooltip: Some(TextComponent::text("Основной мир")),
            });
        }

        CommandSuggestions {
            start: request.start,
            length: request.remaining.len() as u32,
            values,
        }
    }
}

/// Suggests supported pre-generation shapes (`square`, `circle`).
pub struct ShapeSuggestions;

impl CommandSuggestionHandler for ShapeSuggestions {
    fn suggest(
        &self,
        _sender: CommandSender,
        _server: Server,
        request: SuggestionRequest,
    ) -> CommandSuggestions {
        let query = request.remaining.to_lowercase();
        let shapes = [
            ("square", "Квадратная область генерации"),
            ("circle", "Радиальная (круглая) область генерации"),
        ];

        let mut values = Vec::new();
        for (name, desc) in shapes {
            if name.starts_with(&query) {
                values.push(CommandSuggestion {
                    value: name.to_string(),
                    tooltip: Some(TextComponent::text(desc)),
                });
            }
        }

        CommandSuggestions {
            start: request.start,
            length: request.remaining.len() as u32,
            values,
        }
    }
}
