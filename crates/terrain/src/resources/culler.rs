use assets::Assets;
use graphics::Graphics;
use crate::TerrainSettings;

// Chunk culler will be responsible for culling invisible / culled (GPU frustum culled) chunks that are not visible
// TODO: Implement some sort of GPU occlusion culling with the "fill" state of each chunk
pub struct ChunkCuller {
}

impl ChunkCuller {
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        settings: &TerrainSettings,
    ) -> Self {
        Self {
            
        }
    }
}
