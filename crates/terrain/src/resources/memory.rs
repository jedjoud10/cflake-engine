use std::sync::Arc;

use ahash::AHashMap;
use assets::Assets;

use ecs::Entity;
use graphics::{
    Buffer, BufferMode, BufferUsage, Compiler, ComputeModule, ComputeShader,
    DrawCountIndirectBuffer, DrawIndexedIndirect, DrawIndexedIndirectBuffer, GpuPod, Graphics,
    ModuleVisibility, PushConstantLayout, StorageAccess, Texel, TriangleBuffer, Vertex, XY,
};
use parking_lot::Mutex;
use rendering::{attributes, AttributeBuffer, MultiDrawIndirectCountMesh};
use utils::{BitSet, Handle, Storage};

use crate::{create_counters, TerrainSettings, PermTriangles, PermVertices};

// Memory manager will be responsible for finding free memory and copying chunks there
pub struct MemoryManager {
    // Two buffers that contain the generated indexed indirect buffers (from the compute shaders)
    pub(crate) generated_indexed_indirect_buffers: Vec<DrawIndexedIndirectBuffer>,

    // ...and the ones that have been culled by the culling compute shader
    pub(crate) culled_indexed_indirect_buffers: Vec<Handle<DrawIndexedIndirectBuffer>>,
    pub(crate) culled_count_buffer: Handle<DrawCountIndirectBuffer>,

    // Vectors that contains the shared buffers needed for multidraw indirect
    pub(crate) shared_positions_buffers: Vec<Handle<PermVertices>>,
    pub(crate) shared_triangle_buffers: Vec<Handle<PermTriangles>>,

    // Will find us a free memory range
    pub(crate) compute_find: ComputeShader,

    // Used for copying memory to the permanent memory
    pub(crate) offsets: Buffer<u32>,
    pub(crate) counters: Buffer<u32>,

    // Vertices and triangles per allocation
    pub(crate) output_vertex_buffer_length: usize,
    pub(crate) output_triangle_buffer_length: usize,
    
    // Vertices and triangles per sub allocation
    pub(crate) vertices_per_sub_allocation: u32,
    pub(crate) triangles_per_sub_allocation: u32,

    // Used to keep track of what buffers will be used per sub-allocation
    pub sub_allocation_chunk_indices: Vec<Buffer<u32>>,
    pub(crate) compute_copy: ComputeShader,

    // Buffer to store the position and scale of each chunk
    pub(crate) generated_position_scaling_buffers: Vec<Buffer<vek::Vec4<f32>>>,
    pub(crate) culled_position_scaling_buffers: Vec<Buffer<vek::Vec4<f32>>>,

    // Buffer to store the visibility of each chunk
    // This is a bitwise buffer, so each element actually represents the visibility of 32 chunks at a time
    pub(crate) visibility_buffers: Vec<Buffer<u32>>,

    // Temporary buffer that will store the visibility of each chunk as a bitwise 32 bit uint
    // Updated everytime the manager needs it to update
    pub(crate) visibility_bitsets: Vec<BitSet<u32>>,

    // Keeps track of the mesh handles that are shared per allocation
    pub(crate) allocation_meshes: Vec<Handle<MultiDrawIndirectCountMesh>>,

    // Keeps track of the offset/counter async data of each chunk
    pub(crate) readback_offsets_and_counters: Arc<Mutex<AHashMap<Entity, (Option<vek::Vec2<u32>>, Option<vek::Vec2<u32>>)>>>,

    // Keeps track of the vertices/triangles async data of nearby chunks
    pub(crate) readback_vertices_and_triangles: Arc<Mutex<AHashMap<Entity, (Option<Vec<vek::Vec2<f32>>>, Option<Vec<[u32; 3]>>)>>>,
}

impl MemoryManager {
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        vertices: &mut Storage<PermVertices>,
        triangles: &mut Storage<PermTriangles>,
        indexed_indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
        draw_count_indirect_buffers: &mut Storage<DrawCountIndirectBuffer>,
        multi_draw_indirect_count_meshes: &mut Storage<MultiDrawIndirectCountMesh>,
        settings: &TerrainSettings,
    ) -> Self {
        let allocation_count = settings.memory.allocation_count;
        let sub_allocation_count = settings.memory.sub_allocation_count;

        let mut output_vertex_buffer_length =
            graphics.device().limits().max_storage_buffer_binding_size as usize / 32;
        let mut output_triangle_buffer_length =
            graphics.device().limits().max_storage_buffer_binding_size as usize / 12;
        
        // Reduce these numbers blud
        output_vertex_buffer_length /= 2;
        output_triangle_buffer_length /= 2;
        
        // Get number of sub-allocation chunks for two buffer types (vertices and triangles)
        let vertex_sub_allocations_length =
            (output_vertex_buffer_length as f32) / sub_allocation_count as f32;
        let triangle_sub_allocations_length =
            (output_triangle_buffer_length as f32) / sub_allocation_count as f32;
        let vertices_per_sub_allocation =
            (vertex_sub_allocations_length.floor() as u32).next_power_of_two();
        let triangles_per_sub_allocation =
            (triangle_sub_allocations_length.floor() as u32).next_power_of_two();

        // Create an empty buffer of a specific type N amounts of time
        fn create_empty_buffer_count<T: GpuPod, const TYPE: u32>(
            graphics: &Graphics,
            count: usize,
        ) -> Vec<Buffer<T, TYPE>> {
            (0..count)
                .map(|_| crate::create_empty_buffer(graphics))
                .collect::<Vec<_>>()
        }

        // Create multiple buffers for N allocations
        let generated_indexed_indirect_buffers =
            create_empty_buffer_count(graphics, allocation_count);

        // And another one that contains the culled indexed indirect elements
        let culled_indexed_indirect_buffers = (0..allocation_count)
            .map(|_| indexed_indirect_buffers.insert(crate::create_empty_buffer(graphics)))
            .collect::<Vec<_>>();

        // One that contains the culled counts (for all allocation)
        let culled_count_buffer = draw_count_indirect_buffers.insert(
            DrawCountIndirectBuffer::splatted(
                graphics,
                allocation_count,
                0,
                BufferMode::Dynamic,
                BufferUsage::WRITE | BufferUsage::STORAGE,
            )
            .unwrap(),
        );

        // Visibility bitset and GPU buffers
        let visibility_bitsets = (0..allocation_count)
            .map(|_| BitSet::new())
            .collect();
        let visibility_buffers = create_empty_buffer_count(graphics, allocation_count);

        // Generated and culled positions and scalings
        let culled_position_scaling_buffers =
            create_empty_buffer_count(graphics, allocation_count);
        let generated_position_scaling_buffers =
            create_empty_buffer_count(graphics, allocation_count);

        // Allocate the chunk indices that will be stored per allocation
        let sub_allocation_chunk_indices = (0..allocation_count)
            .map(|_| {
                Buffer::<u32>::splatted(
                    graphics,
                    sub_allocation_count,
                    u32::MAX,
                    BufferMode::Dynamic,
                    BufferUsage::STORAGE | BufferUsage::WRITE | BufferUsage::READ,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        // Allocate the required packed data buffers
        let shared_positions_buffers = (0..allocation_count)
            .map(|_| {
                let value = AttributeBuffer::<attributes::Position>::zeroed(
                    graphics,
                    output_vertex_buffer_length,
                    BufferMode::Dynamic,
                    BufferUsage::STORAGE,
                )
                .unwrap();
                vertices.insert(value)
            })
            .collect::<Vec<_>>();

        // Allocate the required triangle buffers
        let shared_triangle_buffers = (0..allocation_count)
            .map(|_| {
                triangles.insert(
                    TriangleBuffer::zeroed(
                        graphics,
                        output_triangle_buffer_length,
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

        // Spec constants
        compiler.use_constant(0, sub_allocation_count as u32);
        compiler.use_constant(1, vertices_per_sub_allocation);
        compiler.use_constant(2, triangles_per_sub_allocation);

        // Create the compute shader that will find a free memory allocation
        let compute_find = ComputeShader::new(module, &compiler).unwrap();

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
            PushConstantLayout::single(<u32 as GpuPod>::size(), ModuleVisibility::Compute).unwrap(),
        );

        // Sizes of the temp and perm buffers
        compiler.use_constant(0, settings.mesher.size);
        compiler.use_constant(1, output_vertex_buffer_length as u32);
        compiler.use_constant(2, output_triangle_buffer_length as u32);

        // Temporary buffers
        compiler.use_storage_buffer::<<XY<f32> as Vertex>::Storage>(
            "temporary_vertices",
            StorageAccess::ReadOnly,
        );
        compiler.use_storage_buffer::<u32>("temporary_triangles", StorageAccess::ReadOnly);

        // Permanent buffer allocations
        compiler.use_storage_buffer::<DrawIndexedIndirect>("indirect", StorageAccess::WriteOnly);
        compiler.use_storage_buffer::<<XY<f32> as Vertex>::Storage>(
            "output_vertices",
            StorageAccess::WriteOnly,
        );
        compiler.use_storage_buffer::<u32>("output_triangles", StorageAccess::WriteOnly);

        // Create copy the compute shader
        let compute_copy = ComputeShader::new(module, &compiler).unwrap();

        // Create an offset buffer and counters buffer that will be used to copy temp memory to permanent memory
        let counters = create_counters(graphics, 2, BufferUsage::READ | BufferUsage::WRITE);
        let offsets = create_counters(graphics, 2, BufferUsage::READ | BufferUsage::WRITE);

        // Generate multiple multi-draw indirect meshes that will be used by the global terrain renderer
        let allocation_meshes = (0..allocation_count)
            .map(|allocation| {
                let positions = shared_positions_buffers[allocation].clone();
                let triangles = shared_triangle_buffers[allocation].clone();

                multi_draw_indirect_count_meshes.insert(MultiDrawIndirectCountMesh::from_handles(
                    Some(positions),
                    None,
                    None,
                    None,
                    triangles,
                    culled_indexed_indirect_buffers[allocation].clone(),
                    0,
                    culled_count_buffer.clone(),
                    allocation,
                    0,
                ))
            })
            .collect::<Vec<_>>();

        Self {
            shared_positions_buffers,
            shared_triangle_buffers,
            compute_find,
            sub_allocation_chunk_indices,
            compute_copy,
            offsets,
            counters,
            allocation_meshes,
            generated_indexed_indirect_buffers,
            culled_indexed_indirect_buffers,
            generated_position_scaling_buffers,
            culled_position_scaling_buffers,
            visibility_buffers,
            visibility_bitsets,
            culled_count_buffer,
            readback_offsets_and_counters: Arc::new(Mutex::new(AHashMap::new())),
            readback_vertices_and_triangles: Arc::new(Mutex::new(AHashMap::new())),
            output_vertex_buffer_length,
            output_triangle_buffer_length,
            vertices_per_sub_allocation,
            triangles_per_sub_allocation,
        }
    }
}
