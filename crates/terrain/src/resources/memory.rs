
use assets::Assets;

use graphics::{
    Buffer, BufferMode, BufferUsage, Compiler, ComputeModule, ComputeShader, DrawIndexedIndirect, GpuPod, Graphics, ModuleVisibility, PushConstantLayout, Texel,
    TriangleBuffer, Vertex, XYZW, StorageAccess,
};
use rendering::{
    attributes, AttributeBuffer,
};
use utils::{Handle, Storage};

use crate::{TerrainSettings, create_counters, Vertices, Triangles};

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
        let sub_allocation_chunk_indices = (0..settings.allocations_count)
        .map(|_| {
            Buffer::<u32>::splatted(
                graphics,
                settings.sub_allocations_count,
                u32::MAX,
                BufferMode::Dynamic,
                BufferUsage::STORAGE,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

        let shared_vertex_buffers = (0..settings.allocations_count)
            .map(|_| {
                let value =
                    AttributeBuffer::<attributes::Position>::zeroed(
                        graphics,
                        settings.output_vertex_buffer_length,
                        BufferMode::Dynamic,
                        BufferUsage::STORAGE,
                    )
                    .unwrap();
                vertices.insert(value)
            })
            .collect::<Vec<_>>();

        let shared_triangle_buffers = (0..settings.allocations_count)
            .map(|_| {
                triangles.insert(
                    TriangleBuffer::zeroed(
                        graphics,
                        settings.output_triangle_buffer_length,
                        BufferMode::Dynamic,
                        BufferUsage::STORAGE,
                    )
                    .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/find.comp")
            .unwrap();

        // Create a simple compute shader compiler
        let mut compiler = Compiler::new(assets, graphics);

        // Set storage buffers and counters
        compiler.use_storage_buffer::<u32>("counters", StorageAccess::ReadOnly);
        compiler.use_storage_buffer::<u32>("offsets", StorageAccess::ReadWrite);
        compiler.use_storage_buffer::<u32>("indices", StorageAccess::ReadWrite);

        // Needed to pass in the chunk index
        compiler.use_push_constant_layout(
            PushConstantLayout::single(
                <u32 as GpuPod>::size(),
                ModuleVisibility::Compute,
            )
            .unwrap(),
        );

        // Spec constants
        compiler.use_constant(0, settings.sub_allocations_count as u32);
        compiler.use_constant(1, settings.vertices_per_sub_allocation);
        compiler.use_constant(2, settings.triangles_per_sub_allocation);

        // Create the compute shader that will find a free memory allocation
        let compute_find = ComputeShader::new(module, compiler).unwrap();

        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/copy.comp")
            .unwrap();

        // Create a simple compute shader compiler
        let mut compiler = Compiler::new(assets, graphics);

        // Needed to find how many and where should we copy data
        compiler.use_storage_buffer::<u32>("counters", StorageAccess::ReadOnly);
        compiler.use_storage_buffer::<u32>("offsets", StorageAccess::ReadOnly);

        // Required since we must write to the right indirect buffer element
        compiler.use_push_constant_layout(
            PushConstantLayout::single(
                <u32 as GpuPod>::size(),
                ModuleVisibility::Compute,
            )
            .unwrap(),
        );

        // Sizes of the temp and perm buffers
        compiler.use_constant(0, settings.size);
        compiler.use_constant(1, settings.output_triangle_buffer_length as u32);
        compiler.use_constant(2, settings.output_vertex_buffer_length as u32);

        // Temporary buffers
        compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>(
            "temporary_vertices",
            StorageAccess::ReadOnly
        );
        compiler.use_storage_buffer::<u32>(
            "temporary_triangles",
            StorageAccess::ReadOnly
        );

        // Permanent buffer allocations
        compiler.use_storage_buffer::<DrawIndexedIndirect>(
            "indirect", StorageAccess::WriteOnly
        );
        compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>(
            "output_vertices",
            StorageAccess::WriteOnly
        );
        compiler.use_storage_buffer::<u32>(
            "output_triangles",
            StorageAccess::WriteOnly
        );

        // Create copy the compute shader
        let compute_copy = ComputeShader::new(module, compiler).unwrap();
        
        Self {
            shared_vertex_buffers,
            shared_triangle_buffers,
            compute_find,
            offsets: create_counters(graphics, 2, BufferUsage::READ | BufferUsage::WRITE),
            sub_allocation_chunk_indices,
            compute_copy,
        }
    }
}