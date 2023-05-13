use crate::{ChunkManager, MemoryManager, MeshGenerator, TerrainMaterial, VoxelGenerator, ChunkCuller};
use assets::{Asset, Assets};
use graphics::{
    combine_into_layered, ActiveComputeDispatcher, BindGroup, Compiler, Graphics, PushConstants,
    RawTexels, SamplerFilter, SamplerMipMaps, SamplerSettings, SamplerWrap, TextureMipMaps,
    TextureMode, TextureUsage,
};
use rendering::{AlbedoTexel, MaskTexel, MaterialId, NormalTexel};
use thiserror::Error;
use utils::Handle;

// Terrain generator settings that the user will need to add to configure the terrain gen
// This will also contain computed common data like number of sub allocations and such
pub struct TerrainSettings {
    // Chunk resolution
    pub(crate) size: u32,

    // Mesh generation parameters
    pub(crate) blocky: bool,
    pub(crate) lowpoly: bool,

    // Octree params
    pub(crate) max_depth: u32,

    // Memory managing settings
    pub(crate) allocation_count: usize,
    pub(crate) sub_allocation_count: usize,

    // Vertices and triangles per allocation
    pub(crate) output_triangle_buffer_length: usize,
    pub(crate) output_tex_coord_buffer_length: usize,

    // Vertices and triangles per sub allocation
    pub(crate) vertices_per_sub_allocation: u32,
    pub(crate) triangles_per_sub_allocation: u32,

    // Callbacks for custom voxel data
    pub(crate) voxel_compiler_callback: Option<Box<dyn FnOnce(&mut Compiler) + 'static>>,
    pub(crate) voxel_set_push_constants_callback:
        Option<Box<dyn Fn(&mut PushConstants<ActiveComputeDispatcher>) + 'static>>,
    pub(crate) voxel_set_group_callback: Option<Box<dyn Fn(&mut BindGroup) + 'static>>,
    pub(crate) sub_materials: Option<Vec<TerrainSubMaterial>>,
}

// Terrain "sub-materials" (aka layered textures) that we can load in
// Contains the paths of the sub material textures that we will load
#[derive(Clone)]
pub struct TerrainSubMaterial {
    pub diffuse: String,
    pub normal: String,
    pub mask: String,
}

impl TerrainSettings {
    // Create some new terrain settings for terrain generation
    pub fn new(
        graphics: &Graphics,
        resolution: u32,
        blocky: bool,
        lowpoly: bool,
        allocations: usize,
        sub_allocations: usize,
        max_depth: u32,
        sub_materials: Option<&[TerrainSubMaterial]>,
    ) -> Result<Self, TerrainSettingsError> {
        let mut output_vertex_buffer_length =
            graphics.device().limits().max_storage_buffer_binding_size as usize / 32;
        let mut output_triangle_buffer_length =
            graphics.device().limits().max_storage_buffer_binding_size as usize / 12;

        // Reduce these numbers blud
        output_vertex_buffer_length /= 2;
        output_triangle_buffer_length /= 2;

        // Validate resolution
        if resolution < 16 {
            return Err(TerrainSettingsError::ChunkSizeTooSmall);
        } else if resolution >= 128 {
            return Err(TerrainSettingsError::ChunkSizeTooBig);
        } else if !resolution.is_power_of_two() {
            return Err(TerrainSettingsError::ChunkSizeNotPowerOfTwo);
        }

        // Validate sub-allocations count
        if !sub_allocations.is_power_of_two() {
            return Err(TerrainSettingsError::SubAllocationCountNotPowerOfTwo);
        }

        // Get number of sub-allocation chunks for two buffer types (vertices and triangles)
        let vertex_sub_allocations_length =
            (output_vertex_buffer_length as f32) / sub_allocations as f32;
        let triangle_sub_allocations_length =
            (output_triangle_buffer_length as f32) / sub_allocations as f32;
        let vertices_per_sub_allocation =
            (vertex_sub_allocations_length.floor() as u32).next_power_of_two();
        let triangles_per_sub_allocation =
            (triangle_sub_allocations_length.floor() as u32).next_power_of_two();

        // Decompose the "callbacks" struct into raw options
        /*
        let (vpc, vspc, vsgc) = callbacks.map(|c| {
            (Some(Box::new(c.compiler)), Some(Box::new(c.set_group_callback)), Some(Box::new(c.set_push_constants)))
        }).unwrap_or_default();
        */

        Ok(Self {
            size: resolution,
            blocky,
            lowpoly,
            allocation_count: allocations,
            sub_allocation_count: sub_allocations,
            output_triangle_buffer_length,
            output_tex_coord_buffer_length: output_vertex_buffer_length,
            vertices_per_sub_allocation,
            triangles_per_sub_allocation,
            voxel_compiler_callback: None,
            voxel_set_push_constants_callback: None,
            voxel_set_group_callback: None,
            sub_materials: sub_materials.map(|x| x.to_vec()),
            max_depth,
        })
    }

    // Get the resolution of the terrain chunks
    pub fn resolution(&self) -> u32 {
        self.size
    }

    // Is the terrain blocky looking?
    pub fn blocky(&self) -> bool {
        self.blocky
    }

    // Is the terrain low-poly looking?
    pub fn lowpoly(&self) -> bool {
        self.lowpoly
    }

    // Number of allocations used for vertices and indices
    pub fn allocation_count(&self) -> usize {
        self.allocation_count
    }

    // Number of sub allocations used
    pub fn sub_allocation_count(&self) -> usize {
        self.sub_allocation_count
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
