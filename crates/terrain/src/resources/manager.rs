use std::thread::Thread;

use ahash::{AHashMap, AHashSet};
use assets::{Assets, AsyncHandle};
use coords::Position;
use ecs::{Entity, Scene};
use graphics::{
    combine_into_layered, BufferMode, BufferUsage, DrawIndexedIndirect, DrawIndexedIndirectBuffer,
    GpuPod, Graphics, ImageTexel, LayeredTexture2D, RawTexels, SamplerFilter, SamplerMipMaps,
    SamplerSettings, SamplerWrap, Texel, TextureMipMaps, TextureMode, TextureUsage, Vertex, Buffer,
};
use math::{Octree, Node};
use rand::seq::SliceRandom;
use rendering::{AlbedoTexel, IndirectMesh, MaterialId, Pipelines, Renderer, Surface, MaskTexel, NormalTexel, MultiDrawIndirectMesh, SubSurface};
use utils::{Handle, Storage, BitSet};

use crate::{
    Chunk, ChunkState, LayeredAlbedoMap, LayeredMaskMap, LayeredNormalMap,
    MemoryManager, TerrainMaterial, TerrainSettings, TerrainSubMaterial, create_empty_buffer,
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
    pub(crate) entities: AHashMap<Node, Entity>,

    // Keep track of the number of generated and target children each parent has
    pub(crate) children_count: AHashMap<Node, (usize, usize)>,

    // Keeps track of the last chunk entity (and node) that we generated (last frame)
    // If we did not generate a chunk last frame this will be None
    pub(crate) last_chunk_generated: Option<Entity>,

    // Single entity that contains multiple meshes that represent the terrain
    pub(crate) global_draw_entity: Entity,

    // Buffer to store the position and scale of each chunk
    pub(crate) position_scaling_buffer: Buffer<vek::Vec4<f32>>,
    pub(crate) culled_position_scaling_buffer: Buffer<vek::Vec4<f32>>,

    // Buffer to store the visibility of each chunk
    // This is a bitwise buffer, so each element actually represents the visibility of 32 chunks at a time
    pub(crate) visibility_buffer: Buffer<u32>,

    // Temporary buffer that will store the visibility of each chunk as a bitwise 32 bit uint
    // Updated everytime the manager needs it to update
    pub(crate) visibilities: BitSet<u32>,

    // Viewer (camera) position
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
        let raw_albedo_texels = load_raw_texels_handles::<AlbedoTexel>(&assets, &settings, |x| &x.diffuse);
        let raw_normal_texels = load_raw_texels_handles::<NormalTexel>(&assets, &settings, |x| &x.normal);
        let raw_mask_texels = load_raw_texels_handles::<MaskTexel>(&assets, &settings, |x| &x.mask);

        // Wait till we load ALL the raw texels
        let raw_albedo_texels = raw_albedo_texels.map(|handles| assets.wait_from_iter(handles));
        let raw_normal_texels = raw_normal_texels.map(|handles| assets.wait_from_iter(handles));
        let raw_mask_texels = raw_mask_texels.map(|handles| assets.wait_from_iter(handles));

        // Get rid of ze errors
        let raw_albedo_texels = raw_albedo_texels.map(|x| x.into_iter().collect::<Result<Vec<_>, _>>().unwrap());
        let raw_normal_texels = raw_normal_texels.map(|x| x.into_iter().collect::<Result<Vec<_>, _>>().unwrap());
        let raw_mask_texels = raw_mask_texels.map(|x| x.into_iter().collect::<Result<Vec<_>, _>>().unwrap());

        rayon::scope(|scope| {
            // Create a layered texture 2D that contains the diffuse maps
            scope.spawn(|_| {         
                layered_albedo_map = load_layered_texture(
                    &graphics,
                    raw_albedo_texels,
                );
            });

            // Create a layered texture 2D that contains the normal maps
            scope.spawn(|_| { 
                layered_normal_map = load_layered_texture(
                    &graphics,
                    raw_normal_texels,
                );
            });

            // Create a layered texture 2D that contains the mask maps
            scope.spawn(|_| { 
                layered_mask_map = load_layered_texture(
                    &graphics,
                    raw_mask_texels
                );
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
        let heuristic = math::OctreeHeuristic::Boxed(Box::new(move |target, node| {
            if node.size() == size * 2 {
                // High resolution
                math::aabb_sphere(&node.aabb(), &math::Sphere {
                    center: *target,
                    radius: (size as f32 * 1.0),
                })
            } else if node.size() == size * 4 {
                // Medium resolution
                math::aabb_sphere(&node.aabb(), &math::Sphere {
                    center: *target,
                    radius: size as f32 * 2.0,
                })
            } else if node.size() == size * 8 {
                // Medium resolution
                math::aabb_sphere(&node.aabb(), &math::Sphere {
                    center: *target,
                    radius: size as f32 * 4.0,
                }) 
            } else {
                // Low resolution
                true
            }
        }));

        // Create an octree for LOD chunk generation
        let octree = Octree::new(settings.max_depth, settings.size, heuristic);

        // Create the chunk manager
        Self {
            last_chunk_generated: None,
            material,
            id,
            viewer: None,
            octree,
            entities: Default::default(),
            children_count: Default::default(),
            global_draw_entity,
            position_scaling_buffer: create_empty_buffer(graphics),
            culled_position_scaling_buffer: create_empty_buffer(graphics),
            visibility_buffer: create_empty_buffer(graphics),
            layered_albedo_map,
            layered_normal_map,
            layered_mask_map,
            visibilities: BitSet::new(),
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
        .map(|sub| get_name_callback(&sub))
        .collect::<Vec<_>>();
    Some(assets.async_load_from_iter::<RawTexels<T>>(paths))
}

// Load a 2D layered texture for the given texel type and the multitude of raw texels
fn load_layered_texture<T: ImageTexel>(
    graphics: &Graphics,
    raw: Option<Vec<RawTexels<T>>>,
) -> Option<LayeredTexture2D<T>> {
    raw.map(|raw| combine_into_layered(
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
    ).unwrap())
}
