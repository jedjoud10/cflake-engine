use std::{rc::Rc, cell::RefCell, cmp::min};

use crate::{
    ChunkCuller, ChunkManager, MemoryManager, MeshGenerator, VoxelGenerator,
};

use graphics::{
    ActiveComputeDispatcher, BindGroup, Compiler, Graphics, PushConstants, TextureImportSettings, TextureScale, SamplerSettings,
};

use thiserror::Error;

// Settings needed for terrain mesh generation
pub struct TerrainMeshSettings {
    pub size: u32,
    pub collisions: bool,
    pub max_octree_depth: u32,
    pub quality: f32,
}

// Settings needed for terrain memory management
pub struct TerrainMemorySettings {
    pub allocation_count: usize,
    pub sub_allocation_count: usize,
}


// Terrain "sub-materials" (aka layered textures) that we can load in
// Contains the paths of the sub material textures that we will load
#[derive(Clone)]
pub struct TerrainSubMaterial {
    pub diffuse: String,
    pub normal: String,
    pub mask: String,
}

// Settings that contain sub material data
pub struct TerrainSubMaterialsSettings {
    pub materials: Vec<TerrainSubMaterial>,
    pub scale: TextureScale,
    pub sampler: SamplerSettings,
}

// Settings needed for terrain rendering
pub struct TerrainRenderingSettings {
    pub flat_normals: bool,
    pub derived_normals: bool,
    pub flat_colors: bool,
    pub blocky: bool,
    pub submaterials: Option<TerrainSubMaterialsSettings>,
}

// Terrain generator settings that the user will need to add to configure the terrain gen
// This will also contain computed common data like number of sub allocations and such
pub struct TerrainSettings {
    pub(crate) mesher: TerrainMeshSettings,
    pub(crate) memory: TerrainMemorySettings,
    pub(crate) rendering: TerrainRenderingSettings,
}

impl TerrainSettings {
    // Create some new terrain settings for terrain generation
    pub fn new(
        mesher: TerrainMeshSettings,
        memory: TerrainMemorySettings,
        rendering: TerrainRenderingSettings,
    ) -> Result<Self, TerrainSettingsError> {
        // Validate resolution
        let resolution = mesher.size;
        if resolution < 16 {
            return Err(TerrainSettingsError::ChunkSizeTooSmall);
        } else if resolution > 128 {
            return Err(TerrainSettingsError::ChunkSizeTooBig);
        } else if !resolution.is_power_of_two() {
            return Err(TerrainSettingsError::ChunkSizeNotPowerOfTwo);
        }

        // Validate sub-allocations count
        if !memory.sub_allocation_count.is_power_of_two() {
            return Err(TerrainSettingsError::SubAllocationCountNotPowerOfTwo);
        }

        Ok(Self {
            mesher,
            memory,
            rendering,
        })
    }
}

// Errors that could possibly get returned when trying to initialize a terrain
#[derive(Debug, Error)]
pub enum TerrainSettingsError {
    #[error("Given chunk size is not a power of two")]
    ChunkSizeNotPowerOfTwo,

    #[error("Given chunk size is too small (less than 16)")]
    ChunkSizeTooSmall,

    #[error("Given chunk size is too big (greater than or equal to 128)")]
    ChunkSizeTooBig,

    #[error("Given sub allocation count is not a power of two")]
    SubAllocationCountNotPowerOfTwo,
}

// TODO: EXPLAIN
pub struct Terrain {
    // Compute generators and managers
    pub voxelizer: VoxelGenerator,
    pub mesher: MeshGenerator,
    pub memory: MemoryManager,
    pub culler: ChunkCuller,

    // Chunk manager and rendering
    pub manager: ChunkManager,

    // Terrain settings
    pub settings: TerrainSettings,

    // Is the terrain active?
    pub active: bool,
}
