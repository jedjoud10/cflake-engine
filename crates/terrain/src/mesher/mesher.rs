use rendering::basics::mesh::Mesh;

use super::builder::*;
use super::settings::MesherSettings;
use crate::{ChunkCoords, GlobalStoredVoxelData};

// A struct for organization
// We do not store this mesher, we create it on the spot
pub struct Mesher {
    // Settings
    pub(crate) coords: ChunkCoords,
    pub(crate) builder: MarchingCubes,
    pub(crate) skirts_builder: MarchingCubesSkirts,
}

impl Mesher {
    // Create a new mesher from some new settings
    pub fn new(coords: ChunkCoords, settings: MesherSettings) -> Self {
        Self {
            coords,
            builder: MarchingCubes::new(settings),
            skirts_builder: MarchingCubesSkirts::new(settings),
        }
    }
    // Generate the mesh from the voxel data
    pub fn build(self, data: GlobalStoredVoxelData) -> Mesh {
        // Gotta combine the main mesh and the skirts one
        let main = self.builder.build(&data, self.coords);
        let skirts = self.skirts_builder.build(&data, self.coords);
        Mesh::combine(main, skirts)
    }
}
