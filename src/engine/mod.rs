// src/engine/mod.rs

pub mod pregenerator;
pub mod shape;
pub mod spiral;

pub use shape::Shape;
pub use spiral::{spiral_index_to_offset, SpiralIterator};
