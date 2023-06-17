

use std::{cell::RefCell, rc::Rc, time::Instant, num::NonZeroU8};

use ahash::{AHashMap};
use assets::{Assets, AsyncHandle};

use ecs::{Entity, Scene};
use graphics::{
    combine_into_layered, GpuPod, Graphics, ImageTexel, LayeredTexture2D, RawTexels,
    SamplerFilter, SamplerMipMaps, SamplerSettings, SamplerWrap, Texel, TextureMipMaps,
    TextureUsage, Vertex, SamplerBorderColor, TextureScale, TextureResizeFilter, TextureViewSettings, Texture,
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
            shadow_culled: false,
            id: id.clone(),
        };

        // Create the global terrain renderer entity
        let global_draw_entity = scene.insert((Renderer::default(), surface));
        
        // Generate the lod multipliers programatically based on the quality setting
        let splits = [0.0f32, 0.3, 0.7, 1.0];
        let percents = [0.2f32, 0.4, 0.7, 1.2];
        let max = settings.max_depth as f32;
        let mut lod = (0..settings.max_depth).into_iter().map(|x| {
            let percent = x as f32 / max;

            let i = splits.iter().enumerate().filter(|(_, &rel)| percent >= rel).map(|(i, _)| i).max().unwrap();

            percents[i] * settings.quality.clamp(0.6, 5.0)
        }).collect::<Vec<f32>>();
        lod.insert(0, 1.0);

        // Custom octree heuristic
        let size = settings.size;
        let lod = Rc::new(RefCell::new(lod));
        let lod_cloned = lod.clone();
        let heuristic = math::OctreeHeuristic::Boxed(Box::new(move |target, node| {
            let div = (node.size() / size).next_power_of_two();

            let multiplier = lod.borrow()[node.depth() as usize];

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
            lod_multipliers: lod_cloned,
        }
    }
}

// Load the raw texels asynchronously using our asset system
fn load_raw_texels_handles<T: ImageTexel>(
    assets: &Assets,
    settings: &TerrainSettings,
    get_name_callback: impl Fn(&TerrainSubMaterial) -> &str,
) -> Option<Vec<AsyncHandle<RawTexels<T>>>> {
    let scale = TextureScale::Scale { scaling: 0.25 * 0.25 * 0.5, filter: TextureResizeFilter::Gaussian };
    let inputs = settings
        .sub_materials
        .as_ref()?
        .iter()
        .map(get_name_callback)
        .map(|n| (n, scale, ()))
        .collect::<Vec<_>>();
    Some(assets.async_load_from_iter::<RawTexels<T>>(inputs))
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
                mipmaps: SamplerMipMaps::AutoAniso,
                min_filter: SamplerFilter::Nearest,
                mag_filter: SamplerFilter::Nearest,
                mip_filter: SamplerFilter::Nearest,
                ..Default::default()
            }),
            TextureMipMaps::Manual { mips: &[] },
            &[TextureViewSettings::whole::<<LayeredTexture2D<T> as Texture>::Region>()],
            TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        )
        .unwrap()
    })
}
