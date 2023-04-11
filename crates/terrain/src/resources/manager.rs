use ahash::{AHashMap, AHashSet};
use assets::Assets;
use coords::Position;
use ecs::{Entity, Scene};
use graphics::{
    BufferMode, BufferUsage, DrawIndexedIndirect,
    DrawIndexedIndirectBuffer, GpuPod, Graphics, Texel, Vertex,
};
use rand::seq::SliceRandom;
use rendering::{
    IndirectMesh, MaterialId,
    Pipelines, Surface, Renderer,
};
use utils::{Handle, Storage};

use crate::{ChunkCoords, TerrainMaterial, TerrainSettings, MemoryManager, Chunk, ChunkState};

// Chunk manager will store a handle to the terrain material and shit needed for rendering the chunks
pub struct ChunkManager {
    // Material handle and material ID
    pub(crate) material: Handle<TerrainMaterial>,
    pub(crate) id: MaterialId<TerrainMaterial>,

    // Buffer that contains the indexed indirect draw commands
    pub(crate) indexed_indirect_buffer: Handle<DrawIndexedIndirectBuffer>,

    // HashSet of the currently visible chunk coordinates
    pub(crate) chunks: AHashSet<ChunkCoords>,

    // Hashmap for each chunk entity
    pub(crate) entities: AHashMap<ChunkCoords, Entity>,
    pub(crate) viewer: Option<(Entity, ChunkCoords, vek::Quaternion<f32>)>,

}

impl ChunkManager {
    // Create a new chunk manager that will pre-allocathe meshes and everything else
    // This will pre-allocate the entities that we will use forever
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        settings: &TerrainSettings,
        memory: &MemoryManager,
        scene: &mut Scene,
        indirect_meshes: &mut Storage<IndirectMesh>,
        indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
        materials: &mut Storage<TerrainMaterial>,
        pipelines: &mut Pipelines,
    ) -> Self {
        // Create ONE buffer that will store the indirect arguments
        let indexed_indirect_buffer = indirect_buffers.insert(
            DrawIndexedIndirectBuffer::splatted(
                graphics,
                settings.chunks_count,
                DrawIndexedIndirect {
                    vertex_count: 0,
                    instance_count: 1,
                    base_index: 0,
                    vertex_offset: 0,
                    base_instance: 0,
                },
                BufferMode::Dynamic,
                BufferUsage::STORAGE | BufferUsage::WRITE,
            )
            .unwrap(),
        );

        // Create vector of indices, and shuffle it
        let mut vector = (0..(settings.chunks_count)).into_iter().collect::<Vec<_>>();
        let mut rng = rand::thread_rng();
        vector.shuffle(&mut rng);

        // Create the indirect meshes and fetch their handles, but keep it as an iterator
        let indirect_meshes = vector.iter()
            .map(|i| {
                // Get the allocation index for this chunk
                let allocation = ((*i as f32)
                    / (settings.chunks_per_allocation as f32))
                    .floor() as usize;

                // Get the vertex and triangle buffers that will be shared for this group
                let vertex_buffer = &memory.shared_vertex_buffers[allocation];
                let triangle_buffer =
                    &memory.shared_triangle_buffers[allocation];
                // Create the indirect mesh
                let mut mesh = IndirectMesh::from_handles(
                    Some(vertex_buffer.clone()),
                    None,
                    None,
                    None,
                    triangle_buffer.clone(),
                    indexed_indirect_buffer.clone(),
                    *i,
                );

                // Set the bounding box of the mesh before hand
                mesh.set_aabb(Some(math::Aabb {
                    min: vek::Vec3::zero(),
                    max: vek::Vec3::one() * settings.size as f32,
                }));

                // Insert the mesh into the storage
                indirect_meshes.insert(mesh)
            });

        // Initial value for the terrain material
        let material = materials.insert(TerrainMaterial {
            bumpiness: 0.1,
            roughness: 1.0,
            metallic: 0.0,
            ambient_occlusion: 0.0,
        });

        // Material Id
        let id = pipelines.register(graphics, assets).unwrap();        

        // Add the required chunk entities
        scene.extend_from_iter(vector.iter().zip(indirect_meshes).map(|(i, mesh)| {
            // Create the surface for rendering
            let mut surface = Surface::indirect(
                mesh.clone(),
                material.clone(),
                id.clone(),
            );

            // Hide the surface at first
            surface.visible = false;

            // Create a renderer an a position component
            let mut renderer = Renderer::default();
            renderer.instant_initialized = None;
            let position = Position::default();

            // Indices *should* be shuffled now
            let allocation = i / settings.chunks_per_allocation;
            let local_index = i % settings.chunks_per_allocation;
            let global_index = *i;

            // Create the chunk component
            let chunk = Chunk {
                state: ChunkState::Free,
                coords: ChunkCoords::zero(),
                allocation,
                local_index,
                global_index,
                priority: f32::MIN,
                ranges: None,
            };

            // Create the bundle
            (surface, renderer, position, chunk)
        }));

        // Create the chunk manager
        Self {
            material,
            id,
            chunks: Default::default(),
            entities: Default::default(),
            viewer: None,
            indexed_indirect_buffer,
        }
    }
}