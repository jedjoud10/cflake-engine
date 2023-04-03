use ahash::{AHashMap, AHashSet};
use assets::Assets;
use ecs::Entity;
use graphics::{
    BufferMode, BufferUsage, DrawIndexedIndirect,
    DrawIndexedIndirectBuffer, GpuPod, Graphics, Texel, Vertex,
};
use rendering::{
    IndirectMesh, MaterialId,
    Pipelines,
};
use utils::{Handle, Storage};

use crate::{ChunkCoords, TerrainMaterial, TerrainSettings, MemoryManager};

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
                BufferUsage::STORAGE,
            )
            .unwrap(),
        );

        // Create the indirect meshes and fetch their handles
        let indirect_meshes = (0..(settings.chunks_count))
            .map(|i| {
                // Get the allocation index for this chunk
                let allocation = ((i as f32)
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
                    i,
                );

                // Set the bounding box of the mesh before hand
                mesh.set_aabb(Some(math::Aabb {
                    min: vek::Vec3::zero(),
                    max: vek::Vec3::one() * settings.size as f32,
                }));

                // Insert the mesh into the storage
                
                indirect_meshes.insert(mesh)
            })
            .collect();

        // Initial value for the terrain material
        let value = TerrainMaterial {
            bumpiness: 0.1,
            roughness: 1.0,
            metallic: 0.0,
            ambient_occlusion: 0.0,
        };

        // Create the chunk manager
        Self {
            material: materials.insert(value),
            id: pipelines.register(graphics, assets).unwrap(),
            indirect_meshes,
            chunks: Default::default(),
            entities: Default::default(),
            viewer: None,
        }
    }
}