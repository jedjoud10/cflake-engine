use ahash::{AHashMap, AHashSet};
use assets::Assets;
use ecs::Entity;
use graphics::{
    Buffer, BufferMode, BufferUsage, Compiler, ComputeModule,
    ComputePass, ComputeShader, DrawIndexedIndirect,
    DrawIndexedIndirectBuffer, GpuPod, Graphics, ModuleVisibility,
    Normalized, PushConstantLayout, SamplerSettings, Texel, Texture,
    Texture3D, TextureMipMaps, TextureMode, TextureUsage,
    TriangleBuffer, Vertex, VertexBuffer, R, RGBA, XYZ, XYZW,
};
use rendering::{
    attributes, AttributeBuffer, IndirectMesh, MaterialId, Mesh,
    Pipelines,
};
use utils::{Handle, Storage};

use crate::{ChunkCoords, TerrainMaterial, TerrainSettings, create_counters, Vertices, Triangles};

// Memory manager will be responsible for finding free memory and copying chunks there
pub struct MemoryManager {
    pub(crate) shared_vertex_buffers: Vec<Handle<Vertices>>,
    pub(crate) shared_triangle_buffers: Vec<Handle<Triangles>>,
    pub(crate) compute_find: ComputeShader,
    pub(crate) offsets: Buffer<u32>,
    pub(crate) sub_allocation_chunk_indices: Vec<Buffer<u32>>,
    pub(crate) compute_copy: ComputeShader,
}

impl MemoryManager {
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        vertices: &mut Storage<Vertices>,
        triangles: &mut Storage<Triangles>,
        settings: &TerrainSettings
    ) -> Self {
        Self {
            shared_vertex_buffers: create_vertex_buffers(graphics, vertices, settings.allocations_count, settings.output_vertex_buffer_length),
            shared_triangle_buffers: create_triangle_buffers(graphics, triangles, settings.allocations_count, settings.output_triangle_buffer_length),
            compute_find: load_compute_find_shader(assets, graphics, settings.sub_allocations_count, settings.vertices_per_sub_allocation, settings.triangles_per_sub_allocation),
            offsets: create_counters(graphics, 2),
            sub_allocation_chunk_indices: create_sub_allocation_chunk_indices(graphics, settings.allocations_count, settings.sub_allocations_count),
            compute_copy: load_compute_copy_shader(assets, graphics, settings.output_triangle_buffer_length, settings.output_vertex_buffer_length, settings.allocations_count, settings.size),
        }
    }
}


fn create_sub_allocation_chunk_indices(
    graphics: &Graphics,
    allocations: usize,
    sub_allocations: usize,
) -> Vec<Buffer<u32>> {
    (0..allocations)
        .into_iter()
        .map(|_| {
            Buffer::<u32>::splatted(
                graphics,
                sub_allocations,
                u32::MAX,
                BufferMode::Dynamic,
                BufferUsage::STORAGE,
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}


// Creates multiple big triangle buffers that will contain our data
fn create_triangle_buffers(
    graphics: &Graphics,
    triangles: &mut Storage<TriangleBuffer<u32>>,
    allocations: usize,
    output_triangle_buffer_length: usize,
) -> Vec<Handle<TriangleBuffer<u32>>> {
    (0..allocations)
        .into_iter()
        .map(|_| {
            triangles.insert(
                TriangleBuffer::zeroed(
                    graphics,
                    output_triangle_buffer_length,
                    BufferMode::Dynamic,
                    BufferUsage::STORAGE | BufferUsage::WRITE,
                )
                .unwrap(),
            )
        })
        .collect::<Vec<_>>()
}

// Creates multiple big vertex buffers that will contain our data
fn create_vertex_buffers(
    graphics: &Graphics,
    vertices: &mut Storage<AttributeBuffer<attributes::Position>>,
    allocations: usize,
    output_vertex_buffer_length: usize,
) -> Vec<Handle<AttributeBuffer<attributes::Position>>> {
    (0..allocations)
        .into_iter()
        .map(|_| {
            let value =
                AttributeBuffer::<attributes::Position>::zeroed(
                    graphics,
                    output_vertex_buffer_length as usize,
                    BufferMode::Dynamic,
                    BufferUsage::STORAGE | BufferUsage::WRITE,
                )
                .unwrap();
            vertices.insert(value)
        })
        .collect::<Vec<_>>()
}


// Load the compute shader that will find a free memory range
fn load_compute_find_shader(
    assets: &Assets,
    graphics: &Graphics,
    sub_allocations: usize,
    vertices_per_sub_allocation: u32,
    triangles_per_sub_allocation: u32,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/find.comp")
        .unwrap();

    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);

    // Set storage buffers and counters
    compiler.use_storage_buffer::<u32>("counters", true, true);
    compiler.use_storage_buffer::<u32>("offsets", true, false);
    compiler.use_storage_buffer::<u32>("indices", true, true);

    // Needed to pass in the chunk index
    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <u32 as GpuPod>::size(),
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );

    // Spec constants
    compiler.use_constant(0, sub_allocations as u32);
    compiler.use_constant(1, vertices_per_sub_allocation);
    compiler.use_constant(2, triangles_per_sub_allocation);

    // Create the compute shader that will find a free memory allocation
    ComputeShader::new(module, compiler).unwrap()
}


// Load the compute shader that will copy the temp data to perm allocation space
fn load_compute_copy_shader(
    assets: &Assets,
    graphics: &Graphics,
    output_triangle_buffer_length: usize,
    output_vertex_buffer_length: usize,
    allocations: usize,
    size: u32,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/copy.comp")
        .unwrap();

    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);

    // Needed to find how many and where should we copy data
    compiler.use_storage_buffer::<u32>("counters", true, false);
    compiler.use_storage_buffer::<u32>("offsets", true, false);

    // Required since we must write to the right indirect buffer element
    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <u32 as GpuPod>::size(),
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );

    // Sizes of the temp and perm buffers
    compiler.use_constant(0, size);
    compiler.use_constant(1, output_triangle_buffer_length as u32);
    compiler.use_constant(2, output_vertex_buffer_length as u32);

    // Temporary buffers
    compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>(
        "temporary_vertices",
        true,
        false,
    );
    compiler.use_storage_buffer::<u32>(
        "temporary_triangles",
        true,
        false,
    );

    // Permanent buffer allocations
    compiler.use_storage_buffer::<DrawIndexedIndirect>(
        "indirect", false, true,
    );
    compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>(
        "output_vertices",
        false,
        true,
    );
    compiler.use_storage_buffer::<u32>(
        "output_triangles",
        false,
        true,
    );

    // Create copy the compute shader
    ComputeShader::new(module, compiler).unwrap()
}