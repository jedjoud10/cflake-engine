

use std::{cell::RefCell, rc::Rc, time::Instant};

use ahash::{AHashMap};
use assets::{Assets, AsyncHandle};

use ecs::{Entity, Scene};
use graphics::{
    combine_into_layered, GpuPod, Graphics, ImageTexel, LayeredTexture2D, RawTexels,
    SamplerFilter, SamplerMipMaps, SamplerSettings, SamplerWrap, Texel, TextureMipMaps,
    TextureMode, TextureUsage, Vertex,
};
use math::{Node, Octree};

use rendering::{
    AlbedoTexel, MaskTexel, MaterialId, NormalTexel,
    Pipelines, Renderer, SubSurface, Surface,
};
use utils::{Handle, Storage};

use crate::{
    LayeredAlbedoMap, LayeredMaskMap, LayeredNormalMap,
    MemoryManager, TerrainMaterial, TerrainSettings, TerrainSubMaterial,
};

// Chunk manager will store a handle to the terrain material and shit needed for rendering the chunks
pub struct ChunkManager {
    // Material handle and material ID
    pub(crate) material: Handle<TerrainMaterial>,
    pub(crate) id: MaterialId<TerrainMaterial>,

    // Material sub-material handles
    pub(crate) layered_albedo_map: Option<Handle<LayeredAlbedoMap>>,
    pub(crate) layered_normal_map: Option<Handle<LayeredNormalMap>>,
    pub(crate) layered_mask_map: Option<Handle<LayeredMaskMap>>,

    // Octree used for chunk generation
    pub(crate) octree: Octree,
    pub lod_multipliers: Rc<RefCell<Vec<f32>>>,

    pub(crate) entities: AHashMap<Node, Entity>,

    // Single entity that contains multiple meshes that represent the terrain
    pub(crate) global_draw_entity: Entity,
    pub(crate) chunks_per_allocation: usize,
    pub(crate) new_visibilities: Vec<(usize, usize)>,

    // Viewer (camera) position and last instant when it moved
    pub(crate) viewer: Option<(Entity, vek::Vec3<f32>, vek::Quaternion<f32>)>,
}

impl ChunkManager {
    // Create a new chunk manager that will pre-allocathe meshes and everything else
    // This will pre-allocate the entities that we will use forever
    pub(crate) fn new(
        assets: &Assets,
        graphics: &Graphics,
        memory: &MemoryManager,
        scene: &mut Scene,
        settings: &mut TerrainSettings,
        materials: &mut Storage<TerrainMaterial>,
        layered_albedo_maps: &mut Storage<LayeredAlbedoMap>,
        layered_normal_maps: &mut Storage<LayeredNormalMap>,
        layered_mask_maps: &mut Storage<LayeredMaskMap>,
        pipelines: &mut Pipelines,
    ) -> Self {
        // Textures that we will load
        let mut layered_albedo_map: Option<LayeredAlbedoMap> = None;
        let mut layered_normal_map: Option<LayeredNormalMap> = None;
        let mut layered_mask_map: Option<LayeredMaskMap> = None;

        // Load the raw texels asynchronously
        let raw_albedo_texels =
            load_raw_texels_handles::<AlbedoTexel>(assets, settings, |x| &x.diffuse);
        let raw_normal_texels =
            load_raw_texels_handles::<NormalTexel>(assets, settings, |x| &x.normal);
        let raw_mask_texels = load_raw_texels_handles::<MaskTexel>(assets, settings, |x| &x.mask);

        // Wait till we load ALL the raw texels
        let raw_albedo_texels = raw_albedo_texels.map(|handles| assets.wait_from_iter(handles));
        let raw_normal_texels = raw_normal_texels.map(|handles| assets.wait_from_iter(handles));
        let raw_mask_texels = raw_mask_texels.map(|handles| assets.wait_from_iter(handles));

        // Get rid of ze errors
        let raw_albedo_texels =
            raw_albedo_texels.map(|x| x.into_iter().collect::<Result<Vec<_>, _>>().unwrap());
        let raw_normal_texels =
            raw_normal_texels.map(|x| x.into_iter().collect::<Result<Vec<_>, _>>().unwrap());
        let raw_mask_texels =
            raw_mask_texels.map(|x| x.into_iter().collect::<Result<Vec<_>, _>>().unwrap());

        rayon::scope(|scope| {
            // Create a layered texture 2D that contains the diffuse maps
            scope.spawn(|_| {
                layered_albedo_map = load_layered_texture(graphics, raw_albedo_texels);
            });

            // Create a layered texture 2D that contains the normal maps
            scope.spawn(|_| {
                layered_normal_map = load_layered_texture(graphics, raw_normal_texels);
            });

            // Create a layered texture 2D that contains the mask maps
            scope.spawn(|_| {
                layered_mask_map = load_layered_texture(graphics, raw_mask_texels);
            });
        });

        // After creating the textures, convert them to handles
        let layered_albedo_map = layered_albedo_map.map(|x| layered_albedo_maps.insert(x));
        let layered_normal_map = layered_normal_map.map(|x| layered_normal_maps.insert(x));
        let layered_mask_map = layered_mask_map.map(|x| layered_mask_maps.insert(x));

        // Initial value for the terrain material
        let material = TerrainMaterial;
        let material = materials.insert(material);

        // Material Id
        let id = pipelines
            .register_with(graphics, &*settings, assets)
            .unwrap();

        // Convert the newly created meshes to multiple sub-surfaces
        let subsurfaces = memory.allocation_meshes.iter().map(|mesh| SubSurface {
            mesh: mesh.clone(),
            material: material.clone(),
        });

        // Create one whole "terrain" surface
        let surface = Surface {
            subsurfaces: subsurfaces.collect(),
            visible: true,
            culled: false,
            shadow_caster: true,
            shadow_receiver: true,
            shadow_culled: false,
            id: id.clone(),
        };

        // Create the global terrain renderer entity
        let global_draw_entity = scene.insert((Renderer::default(), surface));

        // Custom octree heuristic
        let size = settings.size;
        let lod_multiplier = Rc::new(RefCell::new(vec![1f32, 1.1, 1.3, 0.9, 0.9, 1.2, 0.2, 0.0]));
        let lod_multiplier_cloned = lod_multiplier.clone();
        let heuristic = math::OctreeHeuristic::Boxed(Box::new(move |target, node| {
            let div = (node.size() / size).next_power_of_two();

            let multiplier = lod_multiplier.borrow()[node.depth() as usize];

            math::aabb_sphere(
                &node.aabb(),
                &math::Sphere {
                    center: *target,
                    radius: (size as f32 * div as f32 * multiplier * 0.5),
                },
            )
        }));

        // Create an octree for LOD chunk generation
        let octree = Octree::new(settings.max_depth, settings.size, heuristic);

        // Create the chunk manager
        Self {
            material,
            id,
            viewer: None,
            octree,
            entities: Default::default(),
            global_draw_entity,
            layered_albedo_map,
            layered_normal_map,
            layered_mask_map,
            new_visibilities: Default::default(),
            chunks_per_allocation: 0,
            lod_multipliers: lod_multiplier_cloned,
        }
    }
}

// Load the raw texels asynchronously using our asset system
fn load_raw_texels_handles<T: ImageTexel>(
    assets: &Assets,
    settings: &TerrainSettings,
    get_name_callback: impl Fn(&TerrainSubMaterial) -> &str,
) -> Option<Vec<AsyncHandle<RawTexels<T>>>> {
    let paths = settings
        .sub_materials
        .as_ref()?
        .iter()
        .map(get_name_callback)
        .collect::<Vec<_>>();
    Some(assets.async_load_from_iter::<RawTexels<T>>(paths))
}

// Load a 2D layered texture for the given texel type and the multitude of raw texels
fn load_layered_texture<T: ImageTexel>(
    graphics: &Graphics,
    raw: Option<Vec<RawTexels<T>>>,
) -> Option<LayeredTexture2D<T>> {
    raw.map(|raw| {
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
        .unwrap()
    })
}
