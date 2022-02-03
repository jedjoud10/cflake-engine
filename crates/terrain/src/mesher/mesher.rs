use rendering::basics::model::Model;

use super::builder::*;
use super::settings::MesherSettings;
use crate::{ChunkCoords, StoredVoxelData};

// A struct for organization
// We do not store this mesher, we create it on the spot
pub struct Mesher<'a> {
    // Settings
    pub(crate) valid_data: &'a StoredVoxelData,
    pub(crate) coords: ChunkCoords,
    pub(crate) settings: MesherSettings,
    pub(crate) builder: MarchingCubes,
}

impl<'a> Mesher<'a> {
    // Create a new mesher from some new settings
    pub fn new(coords: ChunkCoords, valid_data: &'a StoredVoxelData, settings: MesherSettings) -> Self {
        Self {
            valid_data,
            coords,
            settings,
            builder: MarchingCubes::new(settings),
        }
    }
    // Generate the model from the voxel data
    pub fn build(self) -> Model {
        // We use the marching cubes algorithm as default
        self.builder.build(self.valid_data, self.coords)
    }
}
