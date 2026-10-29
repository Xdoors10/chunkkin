// src/engine/shape.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use crate::utils::coord::ChunkCoord;

/// Geometric shapes supported for world pre-generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    #[default]
    Square,
    Circle,
}

impl Shape {
    /// Checks whether a given chunk coordinate falls within the shape's boundary.
    #[must_use]
    pub fn contains(&self, center: ChunkCoord, coord: ChunkCoord, radius_chunks: u32) -> bool {
        let dx = (coord.x - center.x).abs() as u32;
        let dz = (coord.z - center.z).abs() as u32;

        match self {
            Self::Square => dx <= radius_chunks && dz <= radius_chunks,
            Self::Circle => {
                let dx_i64 = dx as i64;
                let dz_i64 = dz as i64;
                let rad_i64 = radius_chunks as i64;
                (dx_i64 * dx_i64 + dz_i64 * dz_i64) <= (rad_i64 * rad_i64)
            }
        }
    }

    /// Calculates or estimates the total number of chunks contained within the shape.
    #[must_use]
    pub fn total_chunks(&self, radius_chunks: u32) -> u64 {
        match self {
            Self::Square => {
                let side = (2 * (radius_chunks as u64)) + 1;
                side * side
            }
            Self::Circle => {
                let r = radius_chunks as i64;
                let r_sq = r * r;
                let mut count = 0u64;
                for x in -r..=r {
                    let x_sq = x * x;
                    let remaining = r_sq - x_sq;
                    if remaining >= 0 {
                        let max_z = (remaining as f64).sqrt().floor() as i64;
                        count += ((2 * max_z) + 1) as u64;
                    }
                }
                count
            }
        }
    }

    /// Canonical display name for feedback messages.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Circle => "circle",
        }
    }
}

impl FromStr for Shape {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "square" | "box" | "sq" => Ok(Self::Square),
            "circle" | "round" | "radial" | "circ" => Ok(Self::Circle),
            unknown => Err(format!(
                "Неизвестная форма области: '{unknown}'. Допустимые: 'square', 'circle'"
            )),
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_contains() {
        let center = ChunkCoord::new(0, 0);
        let shape = Shape::Square;
        assert!(shape.contains(center, ChunkCoord::new(0, 0), 2));
        assert!(shape.contains(center, ChunkCoord::new(2, 2), 2));
        assert!(shape.contains(center, ChunkCoord::new(-2, 2), 2));
        assert!(!shape.contains(center, ChunkCoord::new(3, 0), 2));
    }

    #[test]
    fn test_circle_contains() {
        let center = ChunkCoord::new(0, 0);
        let shape = Shape::Circle;
        assert!(shape.contains(center, ChunkCoord::new(0, 0), 2));
        assert!(shape.contains(center, ChunkCoord::new(2, 0), 2));
        assert!(shape.contains(center, ChunkCoord::new(0, 2), 2));
        // (2, 2) -> dx^2 + dz^2 = 8 > 4 -> outside
        assert!(!shape.contains(center, ChunkCoord::new(2, 2), 2));
    }

    #[test]
    fn test_total_chunks() {
        assert_eq!(Shape::Square.total_chunks(0), 1);
        assert_eq!(Shape::Square.total_chunks(1), 9);
        assert_eq!(Shape::Square.total_chunks(2), 25);

        assert_eq!(Shape::Circle.total_chunks(0), 1);
        assert_eq!(Shape::Circle.total_chunks(1), 5); // (0,0), (+-1,0), (0,+-1)
        assert_eq!(Shape::Circle.total_chunks(2), 13); // 1 + 4 (dist 1) + 4 (dist 2 axis) + 4 (dist sqrt(2)) = 13
    }

    #[test]
    fn test_parse_shape() {
        assert_eq!("square".parse::<Shape>(), Ok(Shape::Square));
        assert_eq!("CIRCLE".parse::<Shape>(), Ok(Shape::Circle));
        assert_eq!("round".parse::<Shape>(), Ok(Shape::Circle));
        assert!("invalid".parse::<Shape>().is_err());
    }
}
