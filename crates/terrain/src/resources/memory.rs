use std::sync::mpsc::{Receiver, Sender, SyncSender};

use assets::Assets;

use ecs::Entity;
use graphics::{
    Buffer, BufferMode, BufferUsage, Compiler, ComputeModule, ComputeShader, DrawIndexedIndirect,
    GpuPod, Graphics, ModuleVisibility, PushConstantLayout, StorageAccess, Texel, TriangleBuffer,
    Vertex, XYZW, XY, DrawIndexedIndirectBuffer,
};
use rendering::{attributes, AttributeBuffer, MultiDrawIndirectMesh};
use utils::{Handle, Storage};

use crate::{create_counters, TerrainSettings, Triangles, Vertices};

// Memory manager will be responsible for finding free memory and copying chunks there
pub struct MemoryManager {
    // Buffer that contains the indexed indirect draw commands
    pub(crate) indexed_indirect_buffer: Handle<DrawIndexedIndirectBuffer>,
    
    // Vectors that contains the shared buffers needed for multidraw indirect
    pub(crate) shared_tex_coord_buffers: Vec<Handle<Vertices>>,
    pub(crate) shared_triangle_buffers: Vec<Handle<Triangles>>,

    // Numbers of chunks used per allocation
    pub(crate) chunks_per_allocations: Vec<usize>,
    pub(crate) compute_find: ComputeShader,

    // Used for copying memory to the permanent memory
    pub(crate) offsets: [Buffer<u32>; 2],
    pub(crate) counters: [Buffer<u32>; 2],

    // Used to keep track of what buffers will be used per sub-allocation
    pub(crate) sub_allocation_chunk_indices: Vec<Buffer<u32>>,
    pub(crate) compute_copy: ComputeShader,

    // Keeps track of the mesh handles that are shared per allocation
    pub(crate) allocation_meshes: Vec<Handle<MultiDrawIndirectMesh>>,

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
        multi_draw_indirect_meshes: &mut Storage<MultiDrawIndirectMesh>,
        settings: &TerrainSettings,
    ) -> Self {
        // Create ONE buffer that will store the indirect arguments
        let indexed_indirect_buffer = indexed_indirect_buffers.insert(
            DrawIndexedIndirectBuffer::from_slice(
                graphics,
                &[],
                BufferMode::Resizable,
                BufferUsage::STORAGE | BufferUsage::WRITE | BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
            )
            .unwrap(),
        );

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

        // Allocate the required texture coordinate (packed data) buffers
        let shared_tex_coord_buffers = (0..settings.allocation_count)
            .map(|_| {
                let value = AttributeBuffer::<attributes::TexCoord>::zeroed(
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

        // Needed to pass in the chunk index
        compiler.use_push_constant_layout(
            PushConstantLayout::single(<u32 as GpuPod>::size(), ModuleVisibility::Compute).unwrap(),
        );

        // Spec constants
        compiler.use_constant(0, settings.sub_allocation_count as u32);
        compiler.use_constant(1, settings.tex_coords_per_sub_allocation);
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
            let tex_coords = shared_tex_coord_buffers[allocation].clone();
            let triangles = shared_triangle_buffers[allocation].clone();
            let indirect = indexed_indirect_buffer.clone();
        
            multi_draw_indirect_meshes.insert(MultiDrawIndirectMesh::from_handles(
                None,
                None,
                None,
                Some(tex_coords.clone()),
                triangles.clone(),
                indirect.clone(),
                0,
                0
            ))
        }).collect::<Vec<_>>();

        Self {
            indexed_indirect_buffer,
            shared_tex_coord_buffers,
            shared_triangle_buffers,
            compute_find,
            sub_allocation_chunk_indices,
            compute_copy,
            chunks_per_allocations: vec![0; settings.allocation_count],
            readback_count_receiver: counter_receiver,
            readback_count_sender: counter_sender,
            readback_offset_receiver: offset_receiver,
            readback_offset_sender: offset_sender,
            offsets,
            counters,
            allocation_meshes,
        }
    }
}
