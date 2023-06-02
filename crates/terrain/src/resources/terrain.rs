use std::{rc::Rc, cell::RefCell, cmp::min};

use crate::{
    ChunkCuller, ChunkManager, MemoryManager, MeshGenerator, VoxelGenerator,
};

use graphics::{
    ActiveComputeDispatcher, BindGroup, Compiler, Graphics, PushConstants,
};

use thiserror::Error;


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
    pub(crate) lod_multipliers: Rc<RefCell<Vec<f32>>>,
    pub(crate) min_lod_distance: Rc<RefCell<f32>>,

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
        lod_multiplier: f32,
        min_lod_distance: f32,
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
            sub_materials: sub_materials.map(|x| x.to_vec()),
            max_depth,
            min_lod_distance: Rc::new(RefCell::new(min_lod_distance)),
            lod_multipliers: Rc::new(RefCell::new(vec![lod_multiplier; max_depth as usize + 1])),
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
