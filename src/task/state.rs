// src/task/state.rs

use serde::{Deserialize, Serialize};
use std::fmt;

/// The execution state of a world pre-generation task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Idle,
    Running,
    Paused,
    Completed,
    Cancelled,
}

impl TaskStatus {
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    #[must_use]
    pub const fn is_paused(&self) -> bool {
        matches!(self, Self::Paused)
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Idle => "Ожидание (Idle)",
            Self::Running => "Выполняется (Running)",
            Self::Paused => "Приостановлено (Paused)",
            Self::Completed => "Завершено (Completed)",
            Self::Cancelled => "Отменено (Cancelled)",
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_predicates() {
        assert!(TaskStatus::Running.is_active());
        assert!(TaskStatus::Paused.is_active());
        assert!(!TaskStatus::Idle.is_active());
        assert!(!TaskStatus::Completed.is_active());
        assert!(!TaskStatus::Cancelled.is_active());
    }
}
