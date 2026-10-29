// src/utils/coord.rs

use serde::{Deserialize, Serialize};

/// Represents 2D chunk coordinates in the Minecraft world grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    /// Creates a new chunk coordinate.
    #[inline]
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Converts block coordinates `(x, z)` to chunk coordinates.
    #[inline]
    #[must_use]
    pub const fn from_block(block_x: i32, block_z: i32) -> Self {
        Self {
            x: block_x >> 4,
            z: block_z >> 4,
        }
    }

    /// Converts radius in blocks to radius in chunks (rounding up).
    #[inline]
    #[must_use]
    pub const fn block_radius_to_chunks(block_radius: u32) -> u32 {
        (block_radius + 15) >> 4
    }

    /// Returns the center block coordinates `(x, z)` of this chunk.
    #[inline]
    #[must_use]
    pub const fn center_block(&self) -> (i32, i32) {
        ((self.x << 4) + 8, (self.z << 4) + 8)
    }

    /// Returns the minimum block coordinates `(x, z)` in this chunk.
    #[inline]
    #[must_use]
    pub const fn min_block(&self) -> (i32, i32) {
        (self.x << 4, self.z << 4)
    }

    /// Calculates the squared Euclidean distance to another chunk coordinate.
    #[inline]
    #[must_use]
    pub fn distance_squared(&self, other: Self) -> u64 {
        let dx = (self.x as i64) - (other.x as i64);
        let dz = (self.z as i64) - (other.z as i64);
        (dx * dx + dz * dz) as u64
    }

    /// Calculates the Chebyshev (maximum coordinate difference) distance.
    #[inline]
    #[must_use]
    pub fn chebyshev_distance(&self, other: Self) -> u32 {
        let dx = (self.x - other.x).abs() as u32;
        let dz = (self.z - other.z).abs() as u32;
        dx.max(dz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_to_chunk_conversion() {
        assert_eq!(ChunkCoord::from_block(0, 0), ChunkCoord::new(0, 0));
        assert_eq!(ChunkCoord::from_block(15, 15), ChunkCoord::new(0, 0));
        assert_eq!(ChunkCoord::from_block(16, 16), ChunkCoord::new(1, 1));
        assert_eq!(ChunkCoord::from_block(-1, -1), ChunkCoord::new(-1, -1));
        assert_eq!(ChunkCoord::from_block(-16, -16), ChunkCoord::new(-1, -1));
        assert_eq!(ChunkCoord::from_block(-17, -17), ChunkCoord::new(-2, -2));
    }

    #[test]
    fn test_block_radius_to_chunks() {
        assert_eq!(ChunkCoord::block_radius_to_chunks(0), 0);
        assert_eq!(ChunkCoord::block_radius_to_chunks(1), 1);
        assert_eq!(ChunkCoord::block_radius_to_chunks(16), 1);
        assert_eq!(ChunkCoord::block_radius_to_chunks(17), 2);
        assert_eq!(ChunkCoord::block_radius_to_chunks(500), 32);
    }

    #[test]
    fn test_distances() {
        let c1 = ChunkCoord::new(0, 0);
        let c2 = ChunkCoord::new(3, 4);
        assert_eq!(c1.distance_squared(c2), 25);
        assert_eq!(c1.chebyshev_distance(c2), 4);
    }
}
