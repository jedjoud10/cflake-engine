// Terrain stats
#[derive(Default, Debug)]
pub struct TerrainStats {
    pub max_chunks_generated: usize,
    pub max_chunks_deleted: usize,
    pub terrain_size: usize,
    pub worst_update_speed: u128,
    pub best_update_speed: u128,
}
