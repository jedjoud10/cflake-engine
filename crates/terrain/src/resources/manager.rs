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

use crate::{ChunkCoords, TerrainMaterial, TerrainSettings, MemoryManager, Vertices, Triangles};

// Chunk manager will store a handle to the terrain material and shit needed for rendering the chunks
pub struct ChunkManager {
    pub(crate) material: Handle<TerrainMaterial>,
    pub(crate) id: MaterialId<TerrainMaterial>,
    pub(crate) indirect_meshes: Vec<Handle<IndirectMesh>>,
    pub(crate) chunks: AHashSet<ChunkCoords>,
    pub(crate) entities: AHashMap<ChunkCoords, Entity>,
    pub(crate) viewer: Option<(Entity, ChunkCoords)>,
}

impl ChunkManager {
    // Create a new chunk manager that will pre-allocathe meshes and everything else
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        settings: &TerrainSettings,
        memory: &MemoryManager,
        indirect_meshes: &mut Storage<IndirectMesh>,
        indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
        materials: &mut Storage<TerrainMaterial>,
        pipelines: &mut Pipelines,
    ) -> Self {
        // Create ONE buffer that will store the indirect arguments
        let indexed_indirect_buffer = create_draw_indexed_indirect_buffer(
            graphics,
            indirect_buffers,
            settings.chunks_count,
        );

        Self {
            material: materials.insert(TerrainMaterial {
                bumpiness: 0.1,
                roughness: 1.0,
                metallic: 0.0,
                ambient_occlusion: 0.0,
            }),
            id: pipelines.register(graphics, assets).unwrap(),
            indirect_meshes: preallocate_meshes(
                &memory.shared_vertex_buffers,
                &memory.shared_triangle_buffers,
                indirect_meshes,
                indexed_indirect_buffer,
                settings.chunks_count,
                settings.size,
                settings.chunks_per_allocation
            ),

            chunks: Default::default(),
            entities: Default::default(),
            viewer: None,
        }
    }
}

// Create a buffer that will contain all DrawIndexedIndirect elements
fn create_draw_indexed_indirect_buffer(
    graphics: &Graphics,
    buffers: &mut Storage<DrawIndexedIndirectBuffer>,
    chunks_count: usize,
) -> Handle<DrawIndexedIndirectBuffer> {
    let elements = vec![
        DrawIndexedIndirect {
            vertex_count: 0,
            instance_count: 1,
            base_index: 0,
            vertex_offset: 0,
            base_instance: 0,
        };
        chunks_count
    ];

    buffers.insert(
        DrawIndexedIndirectBuffer::from_slice(
            graphics,
            &elements,
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::WRITE,
        )
        .unwrap(),
    )
}


// Create the meshes that we will use for terrain generation before hand
fn preallocate_meshes(
    shared_vertex_buffers: &[Handle<Vertices>],
    shared_triangle_buffers: &[Handle<Triangles>],
    meshes: &mut Storage<IndirectMesh>,
    indexed_indirect_buffer: Handle<DrawIndexedIndirectBuffer>,
    chunks_count: usize,
    chunk_size: u32,
    chunks_per_allocation: usize,
) -> Vec<Handle<IndirectMesh>> {
    (0..(chunks_count as usize))
        .into_iter()
        .map(|i| {
            // Get the allocation index for this chunk
            let allocation = ((i as f32)
                / (chunks_per_allocation as f32))
                .floor() as usize;

            // Get the vertex and triangle buffers that will be shared for this group
            let vertex_buffer = &shared_vertex_buffers[allocation];
            let triangle_buffer =
                &shared_triangle_buffers[allocation];
            // Create the indirect mesh
            let mut mesh = IndirectMesh::from_handles(
                Some(vertex_buffer.clone()),
                None,
                None,
                None,
                triangle_buffer.clone(),
                indexed_indirect_buffer.clone(),
                i,
            );

            // Set the bounding box of the mesh before hand
            mesh.set_aabb(Some(math::Aabb {
                min: vek::Vec3::zero(),
                max: vek::Vec3::one() * chunk_size as f32,
            }));

            // Insert the mesh into the storage
            let handle = meshes.insert(mesh);
            handle
        })
        .collect()
}