// src/utils/color.rs

//! Minecraft color and text formatting constants and helpers.

pub const PREFIX: &str = "§8[§6Chunkkin§8] §r";
pub const GREEN: &str = "§a";
pub const GOLD: &str = "§6";
pub const YELLOW: &str = "§e";
pub const RED: &str = "§c";
pub const GRAY: &str = "§7";
pub const DARK_GRAY: &str = "§8";
pub const WHITE: &str = "§f";
pub const AQUA: &str = "§b";
pub const RESET: &str = "§r";
pub const BOLD: &str = "§l";

#[inline]
pub fn success(msg: &str) -> String {
    format!("{PREFIX}{GREEN}{msg}{RESET}")
}

#[inline]
pub fn info(msg: &str) -> String {
    format!("{PREFIX}{GRAY}{msg}{RESET}")
}

#[inline]
pub fn highlight(label: &str, value: &str) -> String {
    format!("{GRAY}{label}: {GOLD}{value}{RESET}")
}

#[inline]
pub fn warn(msg: &str) -> String {
    format!("{PREFIX}{YELLOW}Внимание: {msg}{RESET}")
}

#[inline]
pub fn error(msg: &str) -> String {
    format!("{PREFIX}{RED}Ошибка: {msg}{RESET}")
}
