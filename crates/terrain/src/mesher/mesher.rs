use rendering::basics::model::{Model, CustomVertexDataBuffer};

use super::builder::*;
use super::settings::MesherSettings;
use crate::{ChunkCoords, StoredVoxelData};

// A struct for organization
// We do not store this mesher, we create it on the spot
pub struct Mesher<'a> {
    // Settings
    pub(crate) valid_data: &'a StoredVoxelData,
    pub(crate) coords: ChunkCoords,
    pub(crate) builder: MarchingCubes,
    pub(crate) skirts_builder: MarchingCubesSkirts,
}

impl<'a> Mesher<'a> {
    // Create a new mesher from some new settings
    pub fn new(coords: ChunkCoords, valid_data: &'a StoredVoxelData, settings: MesherSettings) -> Self {
        Self {
            valid_data,
            coords,
            builder: MarchingCubes::new(settings),
            skirts_builder: MarchingCubesSkirts::new(settings),
        }
    }
    // Generate the model from the voxel data
    pub fn build(self) -> Model {
        // Gotta combine the main model and the skirts one
        let main = self.builder.build(self.valid_data, self.coords);
        main
        //let skirts = self.skirts_builder.build(self.valid_data, self.coords);
        //Model::combine(main, skirts)
    }
}
