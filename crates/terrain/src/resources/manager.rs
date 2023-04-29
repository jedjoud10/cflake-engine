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
use math::{Octree, Node};
use rand::seq::SliceRandom;
use rendering::{AlbedoTexel, IndirectMesh, MaterialId, Pipelines, Renderer, Surface};
use utils::{Handle, Storage};

use crate::{
    Chunk, ChunkState, LayeredAlbedoMap, LayeredMaskMap, LayeredNormalMap,
    MemoryManager, TerrainMaterial, TerrainSettings, TerrainSubMaterial,
};

// Chunk manager will store a handle to the terrain material and shit needed for rendering the chunks
pub struct ChunkManager {
    // Material handle and material ID
    pub(crate) material: Handle<TerrainMaterial>,
    pub(crate) id: MaterialId<TerrainMaterial>,

    // Octree used for chunk generation
    pub(crate) octree: Octree,
    pub(crate) entities: AHashMap<Node, Entity>,

    // Viewer (camera) position
    pub(crate) viewer: Option<(Entity, vek::Vec3<f32>, vek::Quaternion<f32>)>,
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
        materials: &mut Storage<TerrainMaterial>,
        layered_albedo_maps: &mut Storage<LayeredAlbedoMap>,
        layered_normal_maps: &mut Storage<LayeredNormalMap>,
        layered_mask_maps: &mut Storage<LayeredMaskMap>,
        pipelines: &mut Pipelines,
    ) -> Self {
        // Create a layered texture 2D that contains the diffuse maps
        let layered_albedo_map = load_layered_texture(
            &settings,
            &assets,
            &graphics,
            layered_albedo_maps,
            |x| &x.diffuse,
        );

        // Create a layered texture 2D that contains the normal maps
        let layered_normal_map = load_layered_texture(
            &settings,
            &assets,
            &graphics,
            layered_normal_maps,
            |x| &x.normal,
        );

        // Create a layered texture 2D that contains the mask maps
        let layered_mask_map = load_layered_texture(
            &settings,
            &assets,
            &graphics,
            layered_mask_maps,
            |x| &x.mask,
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

        // Create an octree for LOD chunk generation
        let octree = Octree::new(settings.max_depth, settings.size);

        // Create the chunk manager
        Self {
            material,
            id,
            viewer: None,
            octree,
            entities: Default::default(),
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
) -> Option<Handle<LayeredTexture2D<T>>> {
    let paths = settings
        .sub_materials
        .as_ref()?
        .iter()
        .map(|sub| get_name_callback(&sub))
        .collect::<Vec<_>>();

    let loaded = assets.async_load_from_iter::<RawTexels<T>>(paths);

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
