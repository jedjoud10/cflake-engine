use crate::{ValidGeneratedVoxelData, ChunkCoords};

use super::settings::MesherSettings;



// A struct for organization
// We do not store this mesher, we create it on the spot
pub struct Mesher<'a> {
    // Settings
    pub(crate) valid_data: &'a ValidGeneratedVoxelData,
    pub(crate) coords: ChunkCoords,
    pub(crate) settings: MesherSettings
}

impl<'a> Mesher<'a> {
    // Create a new mesher from some new settings
    pub fn new(coords: ChunkCoords, valid_data: &'a ValidGeneratedVoxelData, settings: MesherSettings) -> Self {
        Self {
            valid_data,
            coords,
            settings,
        }
    }

    // Generate the model from the voxel data
}