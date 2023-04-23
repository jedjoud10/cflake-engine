use std::thread::Thread;

use ahash::{AHashMap, AHashSet};
use assets::Assets;
use coords::Position;
use ecs::{Entity, Scene};
use graphics::{
    combine_into_layered, BufferMode, BufferUsage, DrawIndexedIndirect, DrawIndexedIndirectBuffer,
    GpuPod, Graphics, ImageTexel, LayeredTexture2D, RawTexels, SamplerFilter, SamplerMipMaps,
    SamplerSettings, SamplerWrap, Texel, TextureMipMaps, TextureMode, TextureUsage, Vertex,
};
use rand::seq::SliceRandom;
use rendering::{AlbedoTexel, IndirectMesh, MaterialId, Pipelines, Renderer, Surface};
use utils::{Handle, Storage, ThreadPool};

use crate::{
    Chunk, ChunkCoords, ChunkState, LayeredAlbedoMap, LayeredMaskMap, LayeredNormalMap,
    MemoryManager, TerrainMaterial, TerrainSettings, TerrainSubMaterial,
};

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
        settings: &mut TerrainSettings,
        memory: &MemoryManager,
        scene: &mut Scene,
        indirect_meshes: &mut Storage<IndirectMesh>,
        indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
        materials: &mut Storage<TerrainMaterial>,
        layered_albedo_maps: &mut Storage<LayeredAlbedoMap>,
        layered_normal_maps: &mut Storage<LayeredNormalMap>,
        layered_mask_maps: &mut Storage<LayeredMaskMap>,
        pipelines: &mut Pipelines,
        threadpool: &mut ThreadPool,
    ) -> Self {
        // Create ONE buffer that will store the indirect arguments
        let indexed_indirect_buffer = indirect_buffers.insert(
            DrawIndexedIndirectBuffer::splatted(
                graphics,
                settings.chunk_count,
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
        let mut vector = (0..(settings.chunk_count)).into_iter().collect::<Vec<_>>();
        let mut rng = rand::thread_rng();
        vector.shuffle(&mut rng);

        // Create the indirect meshes and fetch their handles, but keep it as an iterator
        let indirect_meshes = vector.iter().map(|i| {
            // Get the allocation index for this chunk
            let allocation =
                ((*i as f32) / (settings.chunks_per_allocation as f32)).floor() as usize;

            // Get the vertex and triangle buffers that will be shared for this group
            let tex_coord_buffer = &memory.shared_tex_coord_buffers[allocation];
            let triangle_buffer = &memory.shared_triangle_buffers[allocation];

            // Create the indirect mesh
            let mut mesh = IndirectMesh::from_handles(
                None,
                None,
                None,
                Some(tex_coord_buffer.clone()),
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

        // Create a layered texture 2D that contains the diffuse maps
        let layered_albedo_map = load_layered_texture(
            &settings,
            &assets,
            &graphics,
            layered_albedo_maps,
            |x| &x.diffuse,
            threadpool,
        );

        // Create a layered texture 2D that contains the normal maps
        let layered_normal_map = load_layered_texture(
            &settings,
            &assets,
            &graphics,
            layered_normal_maps,
            |x| &x.normal,
            threadpool,
        );

        // Create a layered texture 2D that contains the mask maps
        let layered_mask_map = load_layered_texture(
            &settings,
            &assets,
            &graphics,
            layered_mask_maps,
            |x| &x.mask,
            threadpool,
        );

        // Create a new material
        let material = TerrainMaterial {
            layered_albedo_map,
            layered_normal_map,
            layered_mask_map,
        };

        // Initial value for the terrain material
        let material = materials.insert(material);

        // Material Id
        let id = pipelines
            .register_with(graphics, &*settings, assets)
            .unwrap();

        // Add the required chunk entities
        scene.extend_from_iter(vector.iter().zip(indirect_meshes).map(|(i, mesh)| {
            // Create the surface for rendering
            let mut surface = Surface::new(mesh.clone(), material.clone(), id.clone());

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

// Load a 2D layered texture for the given texel type and callback (to get the name of asset files)
fn load_layered_texture<T: ImageTexel>(
    settings: &TerrainSettings,
    assets: &Assets,
    graphics: &Graphics,
    storage: &mut Storage<LayeredTexture2D<T>>,
    get_name_callback: impl Fn(&TerrainSubMaterial) -> &str,
    threadpool: &mut ThreadPool,
) -> Option<Handle<LayeredTexture2D<T>>> {
    let paths = settings
        .sub_materials
        .as_ref()?
        .iter()
        .map(|sub| get_name_callback(&sub))
        .collect::<Vec<_>>();

    let loaded = assets.async_load_from_iter::<RawTexels<T>>(paths, threadpool);

    let raw = assets
        .wait_from_iter(loaded)
        .into_iter()
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    Some(
        storage.insert(
            combine_into_layered(
                graphics,
                raw,
                Some(SamplerSettings {
                    filter: SamplerFilter::Linear,
                    wrap: SamplerWrap::Repeat,
                    mipmaps: SamplerMipMaps::Auto,
                }),
                TextureMipMaps::Manual { mips: &[] },
                TextureMode::Dynamic,
                TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            )
            .unwrap(),
        ),
    )
}
