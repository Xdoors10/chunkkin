use pumpkin_plugin_api::{
    command::{Arg, CommandSender, ConsumedArgs},
};

/// Reads a string argument, returning `None` when the argument was not provided.
///
/// The WASM host returns an empty string for missing keys, so an empty value is
/// treated the same way as an absent one.
#[must_use]
pub fn get_string_arg(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Simple(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// Reads an integer argument, returning `None` when the argument was not provided
/// or did not parse into a bounded number.
#[must_use]
pub fn get_int_arg(args: &ConsumedArgs, key: &str) -> Option<i64> {
    match args.get_value(key) {
        Arg::Num(Ok(pumpkin_plugin_api::command_wit::Number::Int32(v))) => Some(i64::from(v)),
        Arg::Num(Ok(pumpkin_plugin_api::command_wit::Number::Int64(v))) => Some(v),
        _ => None,
    }
}

/// Resolves the world name: the explicit `world` argument when present, otherwise
/// the sender's current world.
#[must_use]
pub fn get_world_arg(sender: &CommandSender, args: &ConsumedArgs) -> Option<String> {
    if let Some(name) = get_string_arg(args, "world") {
        return Some(name);
    }
    if let Some(player) = sender.as_player() {
        let world = player.get_world();
        return Some(world.get_name());
    }
    None
}
