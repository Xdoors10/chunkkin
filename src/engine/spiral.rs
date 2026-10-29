// src/engine/spiral.rs

use crate::engine::shape::Shape;
use crate::utils::coord::ChunkCoord;

/// Converts a 0-based spiral index into a relative 2D offset `(dx, dz)` from center.
///
/// Time complexity: O(1) arithmetic.
#[must_use]
pub fn spiral_index_to_offset(index: u64) -> (i32, i32) {
    if index == 0 {
        return (0, 0);
    }

    // Find the layer k where (2k-1)^2 <= index < (2k+1)^2
    let s = (index as f64).sqrt().floor() as u64;
    let m = if s % 2 == 0 { s - 1 } else { s };
    let k = ((m + 1) / 2) as i64;
    let start = (2 * k - 1) * (2 * k - 1);
    let offset = (index as i64) - start;

    let side_len = 2 * k;
    let side = offset / side_len;
    let step = offset % side_len;

    let k_i32 = k as i32;
    let step_i32 = step as i32;

    match side {
        0 => (k_i32, -k_i32 + 1 + step_i32),
        1 => (k_i32 - 1 - step_i32, k_i32),
        2 => (-k_i32, k_i32 - 1 - step_i32),
        _ => (-k_i32 + 1 + step_i32, -k_i32),
    }
}

/// An iterator that traverses 2D chunk coordinates spiraling outward from a center point,
/// filtered by the specified geometric boundary.
#[derive(Debug, Clone)]
pub struct SpiralIterator {
    center: ChunkCoord,
    radius_chunks: u32,
    shape: Shape,
    current_index: u64,
    max_square_index: u64,
}

impl SpiralIterator {
    /// Creates a new spiral iterator starting at `start_index` (0 for a new task).
    #[must_use]
    pub fn new(center: ChunkCoord, radius_chunks: u32, shape: Shape, start_index: u64) -> Self {
        let side = (2 * (radius_chunks as u64)) + 1;
        let max_square_index = side * side;

        Self {
            center,
            radius_chunks,
            shape,
            current_index: start_index,
            max_square_index,
        }
    }

    /// Returns the current spiral index.
    #[inline]
    #[must_use]
    pub const fn current_index(&self) -> u64 {
        self.current_index
    }

    /// Returns the maximum bounding square index.
    #[inline]
    #[must_use]
    pub const fn max_square_index(&self) -> u64 {
        self.max_square_index
    }

    /// Returns the center chunk.
    #[inline]
    #[must_use]
    pub const fn center(&self) -> ChunkCoord {
        self.center
    }

    /// Returns the radius in chunks.
    #[inline]
    #[must_use]
    pub const fn radius_chunks(&self) -> u32 {
        self.radius_chunks
    }

    /// Returns the active shape filter.
    #[inline]
    #[must_use]
    pub const fn shape(&self) -> Shape {
        self.shape
    }

    /// Calculates a batch of chunk coordinates to process next.
    ///
    /// Skips any coordinates outside the shape mask, advancing `current_index`.
    /// Returns the list of valid chunk coordinates and whether the iteration is finished.
    pub fn next_batch(&mut self, batch_size: usize) -> (Vec<ChunkCoord>, bool) {
        let mut batch = Vec::with_capacity(batch_size);

        while self.current_index < self.max_square_index && batch.len() < batch_size {
            let (dx, dz) = spiral_index_to_offset(self.current_index);
            let coord = ChunkCoord::new(self.center.x + dx, self.center.z + dz);

            if self.shape.contains(self.center, coord, self.radius_chunks) {
                batch.push(coord);
            }

            self.current_index += 1;
        }

        let is_completed = self.current_index >= self.max_square_index;
        (batch, is_completed)
    }
}

impl Iterator for SpiralIterator {
    type Item = (u64, ChunkCoord);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_index < self.max_square_index {
            let idx = self.current_index;
            let (dx, dz) = spiral_index_to_offset(idx);
            let coord = ChunkCoord::new(self.center.x + dx, self.center.z + dz);
            self.current_index += 1;

            if self.shape.contains(self.center, coord, self.radius_chunks) {
                return Some((idx, coord));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_spiral_offsets_uniqueness_and_coverage() {
        let total = 25; // radius 2 -> 5x5 = 25
        let mut seen = HashSet::new();

        for i in 0..total {
            let (dx, dz) = spiral_index_to_offset(i);
            assert!(
                dx.abs() <= 2 && dz.abs() <= 2,
                "Offset ({dx}, {dz}) at index {i} exceeded radius 2"
            );
            assert!(
                seen.insert((dx, dz)),
                "Duplicate offset ({dx}, {dz}) at index {i}"
            );
        }

        assert_eq!(seen.len(), 25);
    }

    #[test]
    fn test_spiral_first_elements() {
        assert_eq!(spiral_index_to_offset(0), (0, 0));
        assert_eq!(spiral_index_to_offset(1), (1, 0));
        assert_eq!(spiral_index_to_offset(2), (1, 1));
        assert_eq!(spiral_index_to_offset(3), (0, 1));
        assert_eq!(spiral_index_to_offset(4), (-1, 1));
        assert_eq!(spiral_index_to_offset(5), (-1, 0));
        assert_eq!(spiral_index_to_offset(6), (-1, -1));
        assert_eq!(spiral_index_to_offset(7), (0, -1));
        assert_eq!(spiral_index_to_offset(8), (1, -1));
        assert_eq!(spiral_index_to_offset(9), (2, -1));
    }

    #[test]
    fn test_spiral_iterator_batch() {
        let center = ChunkCoord::new(10, 20);
        let mut iter = SpiralIterator::new(center, 1, Shape::Square, 0);

        let (batch1, done1) = iter.next_batch(4);
        assert_eq!(batch1.len(), 4);
        assert!(!done1);
        assert_eq!(batch1[0], ChunkCoord::new(10, 20));

        let (batch2, done2) = iter.next_batch(10);
        assert_eq!(batch2.len(), 5);
        assert!(done2);
    }

    #[test]
    fn test_spiral_iterator_circle() {
        let center = ChunkCoord::new(0, 0);
        let iter = SpiralIterator::new(center, 1, Shape::Circle, 0);
        let coords: Vec<_> = iter.map(|(_, c)| c).collect();
        // Circle of radius 1 contains 5 chunks
        assert_eq!(coords.len(), 5);
    }
}
