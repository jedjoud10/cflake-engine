use crate::{ChunkCuller, ChunkManager, MemoryManager, MeshGenerator, VoxelGenerator};

use graphics::{SamplerSettings, TextureScale};

use thiserror::Error;

// Settings needed for terrain mesh generation
pub struct TerrainMeshSettings {
    // Used for mesh generation
    pub size: u32,
    pub collisions: bool,

    // Used for octree generation
    pub max_octree_depth: u32,
    pub octree_depths_size_factors: Option<Vec<f32>>,

    // Used for skirts generation
    pub skirts_threshold_exp: f32,
    pub skirts_threshold_bias: f32,
    pub skirts_threshold_min_density: f32,
}

impl Default for TerrainMeshSettings {
    fn default() -> Self {
        Self {
            size: 64,
            collisions: false,
            max_octree_depth: 8,
            octree_depths_size_factors: None,
            skirts_threshold_exp: 1.7f32,
            skirts_threshold_bias: 0.02f32,
            skirts_threshold_min_density: -800.0f32,
        }
    }
}

// Settings needed for terrain memory management
pub struct TerrainMemorySettings {
    pub allocation_count: usize,
    pub sub_allocation_count: usize,
}

impl Default for TerrainMemorySettings {
    fn default() -> Self {
        Self {
            allocation_count: 1,
            sub_allocation_count: 1024,
        }
    }
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

// How mesh attributes (normals / color) will be calculated for the "Flat" terrain rendering mode
// For the "Smooth" rendering mode, surface normals are always assumed to use Averaged
pub enum TerrainRenderingLowPolyMode {
    // Passed in as a flat attribute
    Flat,

    // Takes the vertex data of 3 vertices and averages them
    Averaged,
}

// How the terrain will be rendered
pub enum TerrainRenderingMode {
    // Blocky rendering (and mesh generation)
    Blocky,

    // Lowpoly shaded, either derived or not derived
    LowPoly(TerrainRenderingLowPolyMode),

    // Smooth shaded
    Smooth,
}

// Per vertex estimated voxel ambient occlusion
pub struct TerrainRenderingAmbientOcclusion {}

// Settings needed for terrain rendering
pub struct TerrainRenderingSettings {
    pub mode: TerrainRenderingMode,
    pub ambient_occlusion: Option<TerrainRenderingAmbientOcclusion>,
    pub submaterials: Option<TerrainSubMaterialsSettings>,
}

impl Default for TerrainRenderingSettings {
    fn default() -> Self {
        Self {
            mode: TerrainRenderingMode::LowPoly(TerrainRenderingLowPolyMode::Averaged),
            ambient_occlusion: Some(TerrainRenderingAmbientOcclusion {}),
            submaterials: None,
        }
    }
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
