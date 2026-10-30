use serde::{Deserialize, Serialize};
use std::fs;
use crate::task::progress::TaskProgress;
use crate::task::state::TaskStatus;

/// Serializable task state stored on disk to support resume after restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTaskState {
    pub status: TaskStatus,
    pub progress: TaskProgress,
}

/// Helper methods to persist and restore generation tasks in the plugin data folder.
pub struct StateStore;

impl StateStore {
    const STATE_FILE: &'static str = "task_state.json";

    /// Saves the current task progress to `task_state.json`.
    pub fn save(data_folder: &str, progress: &TaskProgress, status: TaskStatus) -> Result<(), String> {
        let _ = fs::create_dir_all(data_folder);
        let path = format!("{data_folder}/{}", Self::STATE_FILE);

        let state = PersistedTaskState {
            status,
            progress: progress.clone(),
        };

        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| crate::i18n::trf(crate::i18n::Msg::StateSerializeError, &[&e]))?;

        fs::write(&path, json)
            .map_err(|e| crate::i18n::trf(
                crate::i18n::Msg::StateWriteError,
                &[&path, &e],
            ))
    }

    /// Loads the persisted task state if present.
    #[must_use]
    pub fn load(data_folder: &str) -> Option<PersistedTaskState> {
        let path = format!("{data_folder}/{}", Self::STATE_FILE);
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str::<PersistedTaskState>(&content).ok()
    }

    /// Clears the saved task file once completed or cancelled.
    pub fn clear(data_folder: &str) {
        let path = format!("{data_folder}/{}", Self::STATE_FILE);
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::coord::ChunkCoord;

    #[test]
    fn test_persisted_state_serde() {
        let progress = TaskProgress::new(
            "world".into(),
            ChunkCoord::new(10, -5),
            32,
            32,
            1000,
        );

        let state = PersistedTaskState {
            status: TaskStatus::Paused,
            progress,
        };

        let json = serde_json::to_string(&state).expect("serialize");
        let restored: PersistedTaskState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.status, TaskStatus::Paused);
        assert_eq!(restored.progress.world_name, "world");
        assert_eq!(restored.progress.radius_x_chunks, 32);
        assert_eq!(restored.progress.radius_z_chunks, 32);
    }
}
