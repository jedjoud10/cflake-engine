use std::sync::mpsc::{Receiver, Sender, SyncSender};

use assets::Assets;

use ecs::Entity;
use graphics::{
    Buffer, BufferMode, BufferUsage, Compiler, ComputeModule, ComputeShader, DrawIndexedIndirect,
    GpuPod, Graphics, ModuleVisibility, PushConstantLayout, StorageAccess, Texel, TriangleBuffer,
    Vertex, XYZW, XY, DrawIndexedIndirectBuffer, DrawCountIndirectBuffer,
};
use rendering::{attributes, AttributeBuffer, MultiDrawIndirectMesh, MultiDrawIndirectCountMesh};
use utils::{Handle, Storage, BitSet};

use crate::{create_counters, TerrainSettings, Triangles, Vertices};

// Memory manager will be responsible for finding free memory and copying chunks there
pub struct MemoryManager {
    // Two buffers that contain the generated indexed indirect buffers (from the compute shaders)
    pub(crate) generated_indexed_indirect_buffers: Vec<DrawIndexedIndirectBuffer>,

    // ...and the ones that have been culled by the culling compute shader
    pub(crate) culled_indexed_indirect_buffers: Vec<Handle<DrawIndexedIndirectBuffer>>,
    
    // Vectors that contains the shared buffers needed for multidraw indirect
    pub(crate) shared_positions_buffers: Vec<Handle<Vertices>>,
    pub(crate) shared_triangle_buffers: Vec<Handle<Triangles>>,

    // Will find us a free memory range
    pub(crate) compute_find: ComputeShader,

    // Used for copying memory to the permanent memory
    pub(crate) offsets: [Buffer<u32>; 2],
    pub(crate) counters: [Buffer<u32>; 2],

    // Used to keep track of what buffers will be used per sub-allocation
    pub sub_allocation_chunk_indices: Vec<Buffer<u32>>,
    pub(crate) compute_copy: ComputeShader,

    // Buffer to store the position and scale of each chunk
    pub(crate) position_scaling_buffers: Vec<Buffer<vek::Vec4<f32>>>,
    pub(crate) culled_position_scaling_buffers: Vec<Buffer<vek::Vec4<f32>>>,

    // Buffer to store the visibility of each chunk
    // This is a bitwise buffer, so each element actually represents the visibility of 32 chunks at a time
    pub(crate) visibility_buffers: Vec<Buffer<u32>>,

    // Temporary buffer that will store the visibility of each chunk as a bitwise 32 bit uint
    // Updated everytime the manager needs it to update
    pub(crate) visibility_bitsets: Vec<BitSet<u32>>,

    // Keeps track of the mesh handles that are shared per allocation
    pub(crate) allocation_meshes: Vec<Handle<MultiDrawIndirectCountMesh>>,

    // Channel to receive the asyncrhnoously readback data
    pub(crate) readback_count_receiver: Receiver<(Entity, vek::Vec2<u32>)>,
    pub(crate) readback_count_sender: Sender<(Entity, vek::Vec2<u32>)>,
    pub(crate) readback_offset_receiver: Receiver<(Entity, vek::Vec2<u32>)>,
    pub(crate) readback_offset_sender: Sender<(Entity, vek::Vec2<u32>)>,
}

impl MemoryManager {
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        vertices: &mut Storage<Vertices>,
        triangles: &mut Storage<Triangles>,
        indexed_indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
        draw_count_indirect_buffers: &mut Storage<DrawCountIndirectBuffer>,
        multi_draw_indirect_count_meshes: &mut Storage<MultiDrawIndirectCountMesh>,
        settings: &TerrainSettings,
    ) -> Self {
        // Create one buffer that contains the generated indexed indirect elements
        let generated_indexed_indirect_buffers: Vec<DrawIndexedIndirectBuffer> = (0..settings.allocation_count).into_iter().map(|_| {
            crate::create_empty_buffer(graphics)
        }).collect::<Vec<_>>();

        // And another one that contains the culled indexed indirect elements
        let culled_indexed_indirect_buffers: Vec<Handle<DrawIndexedIndirectBuffer>> = (0..settings.allocation_count).into_iter().map(|_| {
            indexed_indirect_buffers.insert(
                crate::create_empty_buffer(graphics)
            )
        }).collect::<Vec<_>>();

        // ... and another one that contains the culled counts (for all allocation)
        let culled_count_buffers: Handle<DrawCountIndirectBuffer> = draw_count_indirect_buffers.insert(DrawCountIndirectBuffer::splatted(
            graphics,
            settings.allocation_count,
            0,
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        ).unwrap());

        // Allocate the chunk indices that will be stored per allocation
        let sub_allocation_chunk_indices = (0..settings.allocation_count)
            .map(|_| {
                Buffer::<u32>::splatted(
                    graphics,
                    settings.sub_allocation_count,
                    u32::MAX,
                    BufferMode::Dynamic,
                    BufferUsage::STORAGE | BufferUsage::WRITE | BufferUsage::READ,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        // Allocate the required packed data buffers
        let shared_positions_buffers = (0..settings.allocation_count)
            .map(|_| {
                let value = AttributeBuffer::<attributes::Position>::zeroed(
                    graphics,
                    settings.output_tex_coord_buffer_length,
                    BufferMode::Dynamic,
                    BufferUsage::STORAGE,
                )
                .unwrap();
                vertices.insert(value)
            })
            .collect::<Vec<_>>();

        // Allocate the required triangle buffers
        let shared_triangle_buffers = (0..settings.allocation_count)
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

        // Spec constants
        compiler.use_constant(0, settings.sub_allocation_count as u32);
        compiler.use_constant(1, settings.vertices_per_sub_allocation);
        compiler.use_constant(2, settings.triangles_per_sub_allocation);

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
        compiler.use_constant(0, settings.size);
        compiler.use_constant(1, settings.output_tex_coord_buffer_length as u32);
        compiler.use_constant(2, settings.output_triangle_buffer_length as u32);

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

        // Create two offset buffers and counter buffers to be able to do async readback
        let counters = [
            create_counters(graphics, 2, BufferUsage::READ | BufferUsage::WRITE),
            create_counters(graphics, 2, BufferUsage::READ | BufferUsage::WRITE)
        ];
        let offsets = [
            create_counters(graphics, 2, BufferUsage::READ | BufferUsage::WRITE),
            create_counters(graphics, 2, BufferUsage::READ | BufferUsage::WRITE),
        ];

        // Transmitter and receiver to send/receive async data
        let (offset_sender, offset_receiver) = std::sync::mpsc::channel::<(Entity, vek::Vec2<u32>)>();
        let (counter_sender, counter_receiver) = std::sync::mpsc::channel::<(Entity, vek::Vec2<u32>)>();

        // Generate multiple multi-draw indirect meshes that will be used by the global terrain renderer
        let allocation_meshes = (0..settings.allocation_count).into_iter().map(|allocation| {
            let positions = shared_positions_buffers[allocation].clone();
            let triangles = shared_triangle_buffers[allocation].clone();
        
            multi_draw_indirect_count_meshes.insert(MultiDrawIndirectCountMesh::from_handles(
                Some(positions.clone()),
                None,
                None,
                None,
                triangles.clone(),
                culled_indexed_indirect_buffers[allocation].clone(),
                0,
                culled_count_buffers.clone(),
                allocation,
            ))
        }).collect::<Vec<_>>();

        Self {
            shared_positions_buffers,
            shared_triangle_buffers,
            compute_find,
            sub_allocation_chunk_indices,
            compute_copy,
            readback_count_receiver: counter_receiver,
            readback_count_sender: counter_sender,
            readback_offset_receiver: offset_receiver,
            readback_offset_sender: offset_sender,
            offsets,
            counters,
            allocation_meshes,
            generated_indexed_indirect_buffers: todo!(),
            culled_indexed_indirect_buffers: todo!(),
            position_scaling_buffers: todo!(),
            culled_position_scaling_buffers: todo!(),
            visibility_buffers: todo!(),
            visibility_bitsets: todo!(),
        }
    }
}
