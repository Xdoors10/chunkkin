use std::collections::HashSet;
use std::time::Instant;
use pumpkin_plugin_api::{text::TextComponent, Server};
use crate::engine::pregenerator::Pregenerator;
use crate::engine::spiral::SpiralIterator;
use crate::i18n::{tr, trf, Msg};
use crate::storage::config::Selection;
use crate::storage::state_store::{PersistedTaskState, StateStore};
use crate::task::progress::TaskProgress;
use crate::task::state::TaskStatus;
use crate::utils::color::{highlight, info, log_console, strip_colors, PREFIX};
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
    pub console_log: bool,
    pub chat_progress: bool,
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
        console_log: bool,
    ) -> Self {
        Self {
            status: TaskStatus::Idle,
            progress: None,
            iterator: None,
            visited_cache: HashSet::new(),
            batch_size,
            log_interval_ticks,
            auto_save_interval_chunks,
            console_log,
            chat_progress: true,
            last_save_chunk_count: 0,
            ticks_since_last_log: 0,
            data_folder,
        }
    }

    /// Toggles periodic chat progress messages on/off. Returns the new state.
    pub fn toggle_chat_progress(&mut self) -> bool {
        self.chat_progress = !self.chat_progress;
        self.chat_progress
    }

    /// Restores a previously saved task state from disk.
    pub fn restore_saved_state(&mut self, state: PersistedTaskState, auto_resume: bool) {
        let p = state.progress;
        let iterator = SpiralIterator::new(
            p.center,
            p.radius_x_chunks,
            p.radius_z_chunks,
            p.current_index,
        );

        let initial_status = if auto_resume && state.status == TaskStatus::Running {
            TaskStatus::Running
        } else {
            TaskStatus::Paused
        };

        log_console(
            pumpkin_plugin_api::logging::LogLevel::Info,
            &trf(
                Msg::TaskRestored,
                &[
                    &p.world_name,
                    &p.completed_chunks,
                    &p.total_chunks,
                    &format!("{:.1}", p.percentage()),
                    &initial_status,
                ],
            ),
        );

        self.progress = Some(p);
        self.iterator = Some(iterator);
        self.status = initial_status;
        self.visited_cache.clear();
    }

    /// Persists current task state to disk if active.
    pub fn persist_current_state(&self) {
        if let Some(p) = &self.progress
            && self.status.is_active()
        {
            let _ = StateStore::save(&self.data_folder, p, self.status);
        }
    }

    /// Applies freshly reloaded engine settings (batch size, intervals, console
    /// logging) without touching an active task's progress or iterator.
    pub fn update_runtime_settings(
        &mut self,
        batch_size: usize,
        log_interval_ticks: u64,
        auto_save_interval_chunks: u64,
        console_log: bool,
    ) {
        self.batch_size = batch_size;
        self.log_interval_ticks = log_interval_ticks;
        self.auto_save_interval_chunks = auto_save_interval_chunks;
        self.console_log = console_log;
    }

    /// Starts a new pre-generation task from the persisted selection.
    pub fn start(&mut self, selection: &Selection) -> Result<String, String> {
        if self.status == TaskStatus::Running {
            return Err(tr(Msg::TaskAlreadyRunning).into());
        }

        let center = ChunkCoord::from_block(selection.center_x, selection.center_z);
        let (radius_x, radius_z) = selection.radius_chunks();

        // Rectangle: (2*rx+1) x (2*rz+1) chunks
        let total_chunks = ((2 * u64::from(radius_x)) + 1) * ((2 * u64::from(radius_z)) + 1);
        let iterator = SpiralIterator::new(center, radius_x, radius_z, 0);
        let progress = TaskProgress::new(
            selection.world.clone(),
            center,
            radius_x,
            radius_z,
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
            "{}{}",
            PREFIX,
            trf(
                Msg::TaskStarted,
                &[
                    &selection.world,
                    &selection.center_x,
                    &selection.center_z,
                    &radius_x,
                    &radius_z,
                    &total_chunks,
                ],
            )
        );
        // Console copy without chat color codes
        log_console(
            pumpkin_plugin_api::logging::LogLevel::Info,
            &strip_colors(&msg),
        );
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
                let msg = format!("{}{}", PREFIX, tr(Msg::TaskPaused));
                log_console(
                    pumpkin_plugin_api::logging::LogLevel::Info,
                    &strip_colors(&msg),
                );
                Ok(msg)
            }
            TaskStatus::Paused => Err(tr(Msg::TaskAlreadyPaused).into()),
            _ => Err(tr(Msg::NoTaskToPause).into()),
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
                let msg = format!("{}{}", PREFIX, tr(Msg::TaskResumed));
                log_console(
                    pumpkin_plugin_api::logging::LogLevel::Info,
                    &strip_colors(&msg),
                );
                Ok(msg)
            }
            TaskStatus::Running => Err(tr(Msg::TaskRunningAlready).into()),
            _ => Err(tr(Msg::NoTaskToResume).into()),
        }
    }

    /// Cancels and clears the current task.
    pub fn cancel(&mut self) -> Result<String, String> {
        if !self.status.is_active() {
            return Err(tr(Msg::NoTaskToCancel).into());
        }

        self.status = TaskStatus::Cancelled;
        let completed = self.progress.as_ref().map_or(0, |p| p.completed_chunks);
        let total = self.progress.as_ref().map_or(0, |p| p.total_chunks);

        self.progress = None;
        self.iterator = None;
        self.visited_cache.clear();

        StateStore::clear(&self.data_folder);

        let msg = format!(
            "{}{}",
            PREFIX,
            trf(Msg::TaskCancelled, &[&completed, &total])
        );
        log_console(
            pumpkin_plugin_api::logging::LogLevel::Info,
            &strip_colors(&msg),
        );
        Ok(msg)
    }

    /// Generates a status report string for display in commands.
    #[must_use]
    pub fn status_report(&self) -> String {
        let Some(progress) = &self.progress else {
            return info(&trf(Msg::StatusTask, &[&self.status]));
        };

        let mut lines = Vec::with_capacity(6);
        lines.push(format!(
            "{PREFIX}{}: §e{}",
            tr(Msg::LabelStatusGen),
            self.status
        ));
        lines.push(highlight(tr(Msg::LabelWorld), &progress.world_name));
        lines.push(highlight(
            tr(Msg::LabelCenter),
            &format!("({}, {})", progress.center.x, progress.center.z),
        ));
        lines.push(highlight(
            tr(Msg::LabelRegion),
            &trf(
                Msg::ChunksXY,
                &[&progress.radius_x_chunks, &progress.radius_z_chunks]
            ),
        ));
        lines.push(progress.format_summary());

        lines.join("\n")
    }

    /// Formats the selection summary shown by `/chunkkin selection`.
    #[must_use]
    pub fn selection_report(selection: &Selection) -> String {
        let (rx, rz) = selection.radius_chunks();
        [
            format!("{PREFIX}{}", tr(Msg::SelectionHeader)),
            highlight(tr(Msg::LabelWorld), &selection.world),
            highlight(
                tr(Msg::LabelCenter),
                &format!("({}, {})", selection.center_x, selection.center_z),
            ),
            highlight(
                tr(Msg::LabelRadius),
                &(if selection.radius_x == selection.radius_z {
                    trf(Msg::BlocksX, &[&selection.radius_x])
                } else {
                    trf(
                        Msg::BlocksXY,
                        &[&selection.radius_x, &selection.radius_z],
                    )
                }),
            ),
            highlight(
                tr(Msg::LabelRegion),
                &(if rx == rz {
                    trf(Msg::ChunksX, &[&rx])
                } else {
                    trf(Msg::ChunksXY, &[&rx, &rz])
                }),
            ),
        ]
        .join("\n")
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
                log_console(
                    pumpkin_plugin_api::logging::LogLevel::Warn,
                    &trf(Msg::WorldNotFoundTick, &[&world_name]),
                );
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
            if self.chat_progress {
                let summary = format!("{PREFIX}{}", progress.format_summary());
                // System message: no "<Server>" prefix (unlike server.broadcast)
                world.broadcast_system_message(
                    TextComponent::text(&summary),
                    false,
                );
            }
            if self.console_log {
                // Short one-liner without chat color codes or tracing metadata
                log_console(
                    pumpkin_plugin_api::logging::LogLevel::Info,
                    &format!(
                        "{} {}/{} ({:.0}%) {:.1} ch/s",
                        progress.world_name,
                        progress.completed_chunks,
                        progress.total_chunks,
                        progress.percentage(),
                        progress.current_speed
                    ),
                );
            }
            let _ = StateStore::save(&self.data_folder, progress, self.status);
        }

        // Check if finished
        if is_spiral_done || progress.completed_chunks >= progress.total_chunks {
            self.status = TaskStatus::Completed;
            let _ = world.save();
            StateStore::clear(&self.data_folder);

            let completion_msg = format!(
                "{}{}",
                PREFIX,
                trf(
                    Msg::TaskCompleted,
                    &[&progress.world_name, &progress.completed_chunks]
                )
            );
            log_console(
                pumpkin_plugin_api::logging::LogLevel::Info,
                &strip_colors(&completion_msg),
            );
            world.broadcast_system_message(
                TextComponent::text(&completion_msg),
                false,
            );
        }
    }
}
