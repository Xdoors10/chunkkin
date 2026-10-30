//! Minecraft color and text formatting constants and helpers.

/// Logs to the server console without the plugin.target/module/file/line
/// metadata that plain `tracing` events get stamped with by the host.
pub fn log_console(level: pumpkin_plugin_api::logging::LogLevel, message: &str) {
    pumpkin_plugin_api::logging::log(level, message);
}

/// Strips `§`-style chat color codes (§ + single char) for clean console output.
#[must_use]
pub fn strip_colors(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

pub const PREFIX: &str = "§8[§6Chunkkin§8] §r";
pub const GREEN: &str = "§a";
pub const GOLD: &str = "§6";
pub const YELLOW: &str = "§e";
pub const RED: &str = "§c";
pub const GRAY: &str = "§7";
pub const RESET: &str = "§r";

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
pub fn error(msg: &str) -> String {
    format!(
        "{PREFIX}{RED}{}{msg}{RESET}",
        crate::i18n::tr(crate::i18n::Msg::ErrorPrefix)
    )
}
