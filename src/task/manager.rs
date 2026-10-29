// src/task/manager.rs

use std::collections::HashSet;
use std::time::Instant;
use pumpkin_plugin_api::Server;
use crate::engine::pregenerator::Pregenerator;
use crate::engine::shape::Shape;
use crate::engine::spiral::SpiralIterator;
use crate::storage::state_store::{PersistedTaskState, StateStore};
use crate::task::progress::TaskProgress;
use crate::task::state::TaskStatus;
use crate::utils::color::{highlight, info, PREFIX};
use crate::utils::coord::ChunkCoord;

/// Coordinates and drives the background chunk pre-generation task.
pub struct TaskManager {
    pub status: TaskStatus,
    pub progress: Option<TaskProgress>,
    pub iterator: Option<SpiralIterator>,
    pub visited_cache: HashSet<ChunkCoord>,
    pub batch_size: usize,
    pub log_interval_ticks: u64,
    pub auto_save_interval_chunks: u64,
    pub last_save_chunk_count: u64,
    pub ticks_since_last_log: u64,
    pub data_folder: String,
}

impl TaskManager {
    /// Creates a new, idle task manager.
    #[must_use]
    pub fn new(
        data_folder: String,
        batch_size: usize,
        log_interval_ticks: u64,
        auto_save_interval_chunks: u64,
    ) -> Self {
        Self {
            status: TaskStatus::Idle,
            progress: None,
            iterator: None,
            visited_cache: HashSet::new(),
            batch_size,
            log_interval_ticks,
            auto_save_interval_chunks,
            last_save_chunk_count: 0,
            ticks_since_last_log: 0,
            data_folder,
        }
    }

    /// Restores a previously saved task state from disk.
    pub fn restore_saved_state(&mut self, state: PersistedTaskState, auto_resume: bool) {
        let p = state.progress;
        let iterator = SpiralIterator::new(
            p.center,
            p.radius_chunks,
            p.shape,
            p.current_index,
        );

        let initial_status = if auto_resume && state.status == TaskStatus::Running {
            TaskStatus::Running
        } else {
            TaskStatus::Paused
        };

        tracing::info!(
            "Восстановлена задача предварительной генерации мира '{}': прогресс {}/{} чанков ({:.1}%). Статус: {}",
            p.world_name,
            p.completed_chunks,
            p.total_chunks,
            p.percentage(),
            initial_status
        );

        self.progress = Some(p);
        self.iterator = Some(iterator);
        self.status = initial_status;
        self.visited_cache.clear();
    }

    /// Persists current task state to disk if active.
    pub fn persist_current_state(&self) {
        if let Some(p) = &self.progress {
            if self.status.is_active() {
                let _ = StateStore::save(&self.data_folder, p, self.status);
            }
        }
    }

    /// Starts a new pre-generation task.
    pub fn start(
        &mut self,
        world_name: String,
        center: ChunkCoord,
        radius_chunks: u32,
        shape: Shape,
    ) -> Result<String, String> {
        if self.status == TaskStatus::Running {
            return Err("Задача предварительной генерации уже выполняется! Используйте /pregen pause или /pregen cancel.".into());
        }

        let total_chunks = shape.total_chunks(radius_chunks);
        let iterator = SpiralIterator::new(center, radius_chunks, shape, 0);
        let progress = TaskProgress::new(
            world_name.clone(),
            center,
            radius_chunks,
            shape,
            total_chunks,
        );

        self.visited_cache.clear();
        self.progress = Some(progress);
        self.iterator = Some(iterator);
        self.status = TaskStatus::Running;
        self.ticks_since_last_log = 0;
        self.last_save_chunk_count = 0;

        self.persist_current_state();

        let msg = format!(
            "{}Запущена предварительная генерация: мир §a{}§r, радиус §e{} чанков§r ({}), всего §6{} чанков§r.",
            PREFIX,
            world_name,
            radius_chunks,
            shape.name(),
            total_chunks
        );
        tracing::info!("{msg}");
        Ok(msg)
    }

    /// Pauses the currently running task.
    pub fn pause(&mut self) -> Result<String, String> {
        match self.status {
            TaskStatus::Running => {
                self.status = TaskStatus::Paused;
                if let Some(p) = &mut self.progress {
                    p.last_tick_time = None;
                }
                self.persist_current_state();
                let msg = format!("{}Генерация приостановлена. Прогресс сохранен. Для продолжения используйте /pregen continue.", PREFIX);
                tracing::info!("{msg}");
                Ok(msg)
            }
            TaskStatus::Paused => Err("Задача уже приостановлена.".into()),
            _ => Err("Нет активной задачи для приостановки.".into()),
        }
    }

    /// Resumes a paused task.
    pub fn resume(&mut self) -> Result<String, String> {
        match self.status {
            TaskStatus::Paused => {
                self.status = TaskStatus::Running;
                if let Some(p) = &mut self.progress {
                    p.last_tick_time = Some(Instant::now());
                }
                self.persist_current_state();
                let msg = format!("{}Генерация возобновлена.", PREFIX);
                tracing::info!("{msg}");
                Ok(msg)
            }
            TaskStatus::Running => Err("Задача уже выполняется.".into()),
            _ => Err("Нет приостановленной задачи для возобновления.".into()),
        }
    }

    /// Cancels and clears the current task.
    pub fn cancel(&mut self) -> Result<String, String> {
        if !self.status.is_active() {
            return Err("Нет активной задачи для отмены.".into());
        }

        self.status = TaskStatus::Cancelled;
        let completed = self.progress.as_ref().map_or(0, |p| p.completed_chunks);
        let total = self.progress.as_ref().map_or(0, |p| p.total_chunks);

        self.progress = None;
        self.iterator = None;
        self.visited_cache.clear();

        StateStore::clear(&self.data_folder);

        let msg = format!(
            "{}Задача предварительной генерации отменена. Сгенерировано {}/{} чанков.",
            PREFIX, completed, total
        );
        tracing::info!("{msg}");
        Ok(msg)
    }

    /// Generates a status report string for display in commands.
    #[must_use]
    pub fn status_report(&self) -> String {
        let Some(progress) = &self.progress else {
            return info(&format!("Статус задачи: {}", self.status));
        };

        let mut lines = Vec::with_capacity(6);
        lines.push(format!("{PREFIX}Статус генерации: §e{}", self.status));
        lines.push(highlight("Мир", &progress.world_name));
        lines.push(highlight(
            "Центр",
            &format!("({}, {})", progress.center.x, progress.center.z),
        ));
        lines.push(highlight(
            "Область",
            &format!("{} чанков ({})", progress.radius_chunks, progress.shape.name()),
        ));
        lines.push(progress.format_summary());

        lines.join("\n")
    }

    /// Ticks the task engine once (called from server repeating scheduler).
    pub fn tick(&mut self, server: &Server) {
        if self.status != TaskStatus::Running {
            return;
        }

        let Some(progress) = &mut self.progress else {
            self.status = TaskStatus::Idle;
            return;
        };

        let Some(iterator) = &mut self.iterator else {
            self.status = TaskStatus::Idle;
            return;
        };

        let world_name = progress.world_name.clone();
        let world = match server.get_world_by_name(&world_name) {
            Some(w) => w,
            None => {
                tracing::warn!("Мир '{world_name}' не найден во время тика генерации! Приостановка.");
                self.status = TaskStatus::Paused;
                return;
            }
        };

        // Fetch and process batch
        let (batch, is_spiral_done) = iterator.next_batch(self.batch_size);
        if !batch.is_empty() {
            let stats = Pregenerator::process_batch(&world, &batch, &mut self.visited_cache);
            progress.record_batch(stats.processed, iterator.current_index());
        }

        // Periodic auto-save world chunks and task state
        if progress.completed_chunks.saturating_sub(self.last_save_chunk_count) >= self.auto_save_interval_chunks {
            self.last_save_chunk_count = progress.completed_chunks;
            let _ = world.save();
            let _ = StateStore::save(&self.data_folder, progress, self.status);
        }

        // Periodic logging and chat notification
        self.ticks_since_last_log += 1;
        if self.ticks_since_last_log >= self.log_interval_ticks {
            self.ticks_since_last_log = 0;
            let summary = format!("{PREFIX}{}", progress.format_summary());
            tracing::info!("{summary}");
            server.broadcast(&summary);
            let _ = StateStore::save(&self.data_folder, progress, self.status);
        }

        // Check if finished
        if is_spiral_done || progress.completed_chunks >= progress.total_chunks {
            self.status = TaskStatus::Completed;
            let _ = world.save();
            StateStore::clear(&self.data_folder);

            let completion_msg = format!(
                "{}§aГенерация мира §6{} §aуспешно завершена! Обработано §e{}§a чанков.",
                PREFIX, progress.world_name, progress.completed_chunks
            );
            tracing::info!("{completion_msg}");
            server.broadcast(&completion_msg);
        }
    }
}
