// src/utils/format.rs

use std::time::Duration;
use crate::utils::color::{GREEN, GRAY, RESET, YELLOW};

/// Formats a duration into a human-readable string (e.g. "1h 23m 45s" or "12s").
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    if total_secs == 0 {
        return "0s".to_string();
    }

    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{hours}h {mins:02}m {secs:02}s")
    } else if mins > 0 {
        format!("{mins}m {secs:02}s")
    } else {
        format!("{secs}s")
    }
}

/// Formats generation speed (chunks per second).
#[must_use]
pub fn format_speed(chunks_per_sec: f64) -> String {
    if chunks_per_sec < 0.01 {
        "0.0 ch/s".to_string()
    } else {
        format!("{chunks_per_sec:.1} ch/s")
    }
}

/// Generates a visual colored progress bar with percentage.
#[must_use]
pub fn format_progress_bar(completed: u64, total: u64, bar_length: usize) -> String {
    if total == 0 {
        return format!("{GREEN}[{}] {YELLOW}100.0%{RESET}", "=".repeat(bar_length));
    }

    let fraction = (completed as f64) / (total as f64);
    let clamped_fraction = fraction.clamp(0.0, 1.0);
    let filled = ((clamped_fraction * (bar_length as f64)).round() as usize).min(bar_length);
    let empty = bar_length.saturating_sub(filled);

    let percent = clamped_fraction * 100.0;
    format!(
        "{GREEN}[{}{GRAY}{}{GREEN}] {YELLOW}{percent:.1}%{RESET}",
        "=".repeat(filled),
        "-".repeat(empty)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(42)), "42s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 05s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 01m 05s");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0.0), "0.0 ch/s");
        assert_eq!(format_speed(15.42), "15.4 ch/s");
    }

    #[test]
    fn test_progress_bar() {
        let bar = format_progress_bar(50, 100, 10);
        assert!(bar.contains("50.0%"));
    }
}
