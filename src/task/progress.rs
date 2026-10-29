// src/task/progress.rs

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::engine::shape::Shape;
use crate::utils::coord::ChunkCoord;
use crate::utils::format::{format_duration, format_progress_bar, format_speed};

/// Window sample for calculating rolling speed (chunks/second).
#[derive(Debug, Clone, Copy)]
struct SpeedSample {
    timestamp: Instant,
    completed: u64,
}

/// Progress metrics and state snapshot of a pre-generation task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub world_name: String,
    pub center: ChunkCoord,
    pub radius_chunks: u32,
    pub shape: Shape,
    pub current_index: u64,
    pub completed_chunks: u64,
    pub total_chunks: u64,
    pub elapsed_secs: f64,

    #[serde(skip)]
    pub last_tick_time: Option<Instant>,

    #[serde(skip)]
    speed_window: VecDeque<SpeedSample>,

    #[serde(skip)]
    pub current_speed: f64,
}

impl TaskProgress {
    /// Creates a new progress tracker.
    #[must_use]
    pub fn new(
        world_name: String,
        center: ChunkCoord,
        radius_chunks: u32,
        shape: Shape,
        total_chunks: u64,
    ) -> Self {
        Self {
            world_name,
            center,
            radius_chunks,
            shape,
            current_index: 0,
            completed_chunks: 0,
            total_chunks,
            elapsed_secs: 0.0,
            last_tick_time: None,
            speed_window: VecDeque::with_capacity(16),
            current_speed: 0.0,
        }
    }

    /// Updates metrics after processing a batch of chunks.
    pub fn record_batch(&mut self, chunks_in_batch: u64, spiral_index: u64) {
        let now = Instant::now();

        if let Some(prev) = self.last_tick_time {
            let delta = now.duration_since(prev).as_secs_f64();
            self.elapsed_secs += delta;
        }
        self.last_tick_time = Some(now);

        self.completed_chunks += chunks_in_batch;
        self.current_index = spiral_index;

        // Maintain 5-second sliding window for accurate speed calculation
        self.speed_window.push_back(SpeedSample {
            timestamp: now,
            completed: self.completed_chunks,
        });

        // Prune entries older than 5 seconds
        while let Some(front) = self.speed_window.front() {
            if now.duration_since(front.timestamp).as_secs_f64() > 5.0 && self.speed_window.len() > 2 {
                self.speed_window.pop_front();
            } else {
                break;
            }
        }

        if let (Some(oldest), Some(newest)) = (self.speed_window.front(), self.speed_window.back()) {
            let dt = newest.timestamp.duration_since(oldest.timestamp).as_secs_f64();
            let d_chunks = newest.completed.saturating_sub(oldest.completed);
            if dt > 0.05 {
                self.current_speed = (d_chunks as f64) / dt;
            }
        }
    }

    /// Calculates completion percentage [0.0 .. 100.0].
    #[must_use]
    pub fn percentage(&self) -> f64 {
        if self.total_chunks == 0 {
            100.0
        } else {
            ((self.completed_chunks as f64) / (self.total_chunks as f64) * 100.0).clamp(0.0, 100.0)
        }
    }

    /// Calculates Estimated Time of Arrival (ETA).
    #[must_use]
    pub fn eta(&self) -> Option<Duration> {
        let remaining = self.total_chunks.saturating_sub(self.completed_chunks);
        if remaining == 0 {
            return Some(Duration::from_secs(0));
        }

        if self.current_speed > 0.05 {
            let secs = (remaining as f64) / self.current_speed;
            Some(Duration::from_secs_f64(secs))
        } else if self.elapsed_secs > 1.0 && self.completed_chunks > 0 {
            // Fallback to overall average speed
            let avg_speed = (self.completed_chunks as f64) / self.elapsed_secs;
            if avg_speed > 0.05 {
                let secs = (remaining as f64) / avg_speed;
                Some(Duration::from_secs_f64(secs))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Formats a single-line progress summary for chat and console.
    #[must_use]
    pub fn format_summary(&self) -> String {
        let bar = format_progress_bar(self.completed_chunks, self.total_chunks, 16);
        let speed_str = format_speed(self.current_speed);
        let elapsed_str = format_duration(Duration::from_secs_f64(self.elapsed_secs));
        let eta_str = self
            .eta()
            .map_or_else(|| "вычисление...".to_string(), format_duration);

        format!(
            "{bar} §7({}/{}) §e{} §8| §7Прошло: §f{} §8| §7Осталось: §a{}",
            self.completed_chunks,
            self.total_chunks,
            speed_str,
            elapsed_str,
            eta_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_percentage() {
        let mut progress = TaskProgress::new(
            "world".into(),
            ChunkCoord::new(0, 0),
            10,
            Shape::Square,
            100,
        );

        assert_eq!(progress.percentage(), 0.0);
        progress.record_batch(50, 50);
        assert!((progress.percentage() - 50.0).abs() < 0.01);
        progress.record_batch(50, 100);
        assert_eq!(progress.percentage(), 100.0);
    }
}
