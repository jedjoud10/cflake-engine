


use std::rc::Rc;

use graphics::{
    Graphics, Compiler, BindGroup,
};



use crate::{VoxelGenerator, MeshGenerator, MemoryManager, ChunkManager};

// Terrain generator settings that the user will need to add to configure the terrain gen
// This will also contain computed common data like number of sub allocations and such
pub struct TerrainSettings {
    // Chunk resolution
    pub(crate) size: u32,

    // Chunk render distance
    pub(crate) chunk_render_distance: usize,
    
    // Mesh generation parameters
    pub(crate) blocky: bool,
    pub(crate) lowpoly: bool,

    // Memory managing settings
    pub(crate) allocations_count: usize,
    pub(crate) sub_allocations_count: usize,

    // Number of chunks sepcified by the render distance 
    pub(crate) chunks_count: usize,

    // Same as chunk count, but rounded up to a multiple of allocations_count
    pub(crate) over_allocated_chunks_count: usize,
    pub(crate) chunks_per_allocation: usize,

    // Vertices and triangles per allocation
    pub(crate) output_triangle_buffer_length: usize,
    pub(crate) output_vertex_buffer_length: usize,
    
    // Vertices and triangles per sub allocation
    pub(crate) vertices_per_sub_allocation: u32,
    pub(crate) triangles_per_sub_allocation: u32,

    // Callbacks for custom voxel data
    pub(crate) compiler_callback: Option<Box<dyn FnOnce(&mut Compiler) + 'static>>,
    pub(crate) set_group_callback: Option<Box<dyn Fn(&mut BindGroup) + 'static>>,
}

impl TerrainSettings {
    // Create some new terrain settings for terrain generation
    // TODO: Handle validation
    pub fn new(
        graphics: &Graphics,
        resolution: u32,
        render_distance: usize,
        blocky: bool,
        lowpoly: bool,
        allocations: usize,
        sub_allocations: usize,
        /*
        voxel_compiler_callback: impl FnOnce(&mut Compiler) + 'static,
        voxel_set_bind_group_callback: impl Fn(&mut BindGroup) + 'static,
        */
    ) -> Self {
        let output_vertex_buffer_length = graphics
            .device()
            .limits()
            .max_storage_buffer_binding_size as usize
            / 16;
        let output_triangle_buffer_length = graphics
            .device()
            .limits()
            .max_storage_buffer_binding_size as usize
            / 12;


        // Calculate the number of chunk meshes/indirect elements that must be created
        let chunks = (render_distance * 2 + 1).pow(3);

        // Do this so each allocation contains the same amount of chunks
        let over_allocated_chunks_count = ((chunks as f32 / allocations as f32).ceil()
            * (allocations as f32)) as usize;

        // Get number of sub-allocation chunks for two buffer types (vertices and triangles)
        let vertex_sub_allocations_length = (output_vertex_buffer_length as f32) / sub_allocations as f32;
        let triangle_sub_allocations_length = (output_triangle_buffer_length as f32) / sub_allocations as f32;
        let vertices_per_sub_allocation = (vertex_sub_allocations_length.floor() as u32).next_power_of_two();
        let triangles_per_sub_allocation = (triangle_sub_allocations_length.floor() as u32).next_power_of_two();
        let chunks_per_allocation = over_allocated_chunks_count / allocations;

        Self {
            size: resolution,
            chunk_render_distance: render_distance,
            blocky,
            lowpoly,
            allocations_count: allocations,
            sub_allocations_count: sub_allocations,
            chunks_count: chunks,
            over_allocated_chunks_count,
            output_triangle_buffer_length,
            output_vertex_buffer_length,
            vertices_per_sub_allocation,
            triangles_per_sub_allocation,
            chunks_per_allocation,
            compiler_callback: Some(Box::new(|_| {})),
            set_group_callback: Some(Box::new(|_| {})),
        }
    }
}

// TODO: EXPLAIN
pub struct Terrain {
    // Compute generators and managers
    pub voxelizer: VoxelGenerator,
    pub mesher: MeshGenerator,
    pub memory: MemoryManager,

    // Chunk manager and rendering
    pub manager: ChunkManager,

    // Terrain settings
    pub settings: TerrainSettings,
}