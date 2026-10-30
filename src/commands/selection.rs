use pumpkin_plugin_api::{
    command::{Arg, CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
    text::TextComponent,
};
use crate::commands::args::{get_int_arg, get_world_arg};
use crate::commands::config::with_config;
use crate::i18n::{tr, trf, Msg};
use crate::utils::color::{error, PREFIX};

/// `/chunkkin world [мир]` — sets the target world (player's world when omitted).
pub struct WorldHandler;

impl CommandHandler for WorldHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let world_name = match get_world_arg(&sender, &args) {
            Some(name) => name,
            None => {
                sender.send_message(TextComponent::text(&error(tr(Msg::WorldUndetermined))));
                return Ok(1);
            }
        };

        // Validate the world exists
        if server.get_world_by_name(&world_name).is_none() {
            sender.send_message(TextComponent::text(&error(&trf(
                Msg::WorldNotFound,
                &[&world_name],
            ))));
            return Ok(1);
        }

        let msg = with_config(|config| {
            config.selection.world = world_name.clone();
            format!("{PREFIX}{}", trf(Msg::WorldSelected, &[&world_name]))
        });

        match msg {
            Ok(m) => {
                sender.send_message(TextComponent::text(&m));
                Ok(0)
            }
            Err(e) => {
                sender.send_message(TextComponent::text(&error(&e)));
                Ok(1)
            }
        }
    }
}

/// `/chunkkin center [x z]` — sets the selection center (player's position when omitted).
pub struct CenterHandler;

impl CommandHandler for CenterHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let (x, z) = match args.get_value("center") {
            Arg::Pos2d((x, z)) => (x.floor() as i32, z.floor() as i32),
            _ => {
                let Some(player) = sender.as_player() else {
                    sender.send_message(TextComponent::text(&error(tr(Msg::CenterSpecify))));
                    return Ok(1);
                };
                let pos = player.get_position();
                (pos.0.floor() as i32, pos.2.floor() as i32)
            }
        };

        let msg = with_config(|config| {
            config.selection.center_x = x;
            config.selection.center_z = z;
            format!("{PREFIX}{}", trf(Msg::CenterSet, &[&x, &z]))
        });

        match msg {
            Ok(m) => {
                sender.send_message(TextComponent::text(&m));
                Ok(0)
            }
            Err(e) => {
                sender.send_message(TextComponent::text(&error(&e)));
                Ok(1)
            }
        }
    }
}

/// `/chunkkin radius <радиус> [радиус]` — sets per-axis radii in blocks.
pub struct RadiusHandler;

impl CommandHandler for RadiusHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let Some(radius_x) = get_int_arg(&args, "radius") else {
            sender.send_message(TextComponent::text(&error(tr(Msg::RadiusSpecify))));
            return Ok(1);
        };

        let Ok(radius_x) = u32::try_from(radius_x) else {
            sender.send_message(TextComponent::text(&error(tr(Msg::RadiusPositive))));
            return Ok(1);
        };

        let radius_z = get_int_arg(&args, "radius_z")
            .map_or(radius_x, |rz| u32::try_from(rz).unwrap_or(radius_x));

        let msg = with_config(|config| {
            config.selection.radius_x = radius_x;
            config.selection.radius_z = radius_z;
            let dims = trf(Msg::BlocksXY, &[&(2 * radius_x), &(2 * radius_z)]);
            let radii = if radius_x == radius_z {
                radius_x.to_string()
            } else {
                format!("{radius_x} x {radius_z}")
            };
            format!("{PREFIX}{}", trf(Msg::RadiusSet, &[&radii, &dims]))
        });

        match msg {
            Ok(m) => {
                sender.send_message(TextComponent::text(&m));
                Ok(0)
            }
            Err(e) => {
                sender.send_message(TextComponent::text(&error(&e)));
                Ok(1)
            }
        }
    }
}

/// `/chunkkin selection` — shows the current selection.
pub struct SelectionHandler;

impl CommandHandler for SelectionHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let report =
            with_config(|config| crate::task::TaskManager::selection_report(&config.selection));

        match report {
            Ok(r) => {
                sender.send_message(TextComponent::text(&r));
                Ok(0)
            }
            Err(e) => {
                sender.send_message(TextComponent::text(&error(&e)));
                Ok(1)
            }
        }
    }
}

