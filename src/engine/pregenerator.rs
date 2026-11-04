use pumpkin_plugin_api::world::World;
use crate::utils::coord::ChunkCoord;

/// Statistics gathered from processing a single chunk batch.
#[derive(Debug, Default, Clone, Copy)]
pub struct BatchStats {
    pub processed: u64,
    pub newly_loaded: u64,
    pub already_cached: u64,
}

/// Core generation worker that interfaces with Pumpkin's world and chunk APIs.
pub struct Pregenerator;

impl Pregenerator {
    /// Processes a single chunk coordinate:
    /// Checks if the chunk is already in memory or generates/loads it.
    pub fn process_chunk(world: &World, coord: ChunkCoord) -> bool {
        // Check if chunk is already loaded in memory
        let already_loaded = world.get_chunk(coord.x, coord.z).is_some();

        if !already_loaded {
            // Trigger world generation / chunk load by querying the world heightmap
            // at the center block of this chunk. In Pumpkin, get_top_block_y triggers
            // read_chunk_sync and generation if unpopulated.
            let (bx, bz) = coord.center_block();
            let _ = world.get_top_block_y(bx, bz);
            true
        } else {
            false
        }
    }

    /// Processes a batch of chunk coordinates in the given world.
    pub fn process_batch(world: &World, coords: &[ChunkCoord]) -> BatchStats {
        let mut stats = BatchStats::default();

        for &coord in coords {
            stats.processed += 1;
            let was_new = Self::process_chunk(world, coord);
            if was_new {
                stats.newly_loaded += 1;
            } else {
                stats.already_cached += 1;
            }
        }

        stats
    }
}
