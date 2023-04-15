use std::{io::BufReader, iter::repeat, path::{PathBuf, Path}, sync::Arc};

use ahash::{AHashSet, AHashMap};
use assets::{Asset, Data};
use base64::{engine::{GeneralPurposeConfig, GeneralPurpose}, alphabet, Engine};
use ecs::Scene;
use graphics::{Graphics, Texture2D, Texture, SamplerSettings, SamplerFilter, SamplerWrap, SamplerMipMaps, TextureMode, TextureUsage, TextureMipMaps, TextureImportSettings, TextureScale, Texel, ImageTexel};
use utils::{Storage, Handle, ThreadPool};
use world::{World, Read, Write};
use crate::{Mesh, MaskMap, NormalMap, AlbedoMap, Pipelines, PhysicallyBasedMaterial};

// These are the context values that must be given to the GltfScene to load it
pub struct GtlfContext<'a> {
    // Needed resources
    pub graphics: Read<'a, Graphics>,
    pub scene: Write<'a, Scene>,
    pub pipelines: Write<'a, Pipelines>,

    // Storages that will contain the newly loaded GTLF data
    pub meshes: Write<'a, Storage<Mesh>>,
    pub albedo_maps: Write<'a, Storage<AlbedoMap>>,
    pub normal_maps: Write<'a, Storage<NormalMap>>,
    pub mask_maps: Write<'a, Storage<MaskMap>>,
    pub pbr_materials: Write<'a, Storage<PhysicallyBasedMaterial>>,
    pub threadpool: Write<'a, ThreadPool>,
}

impl<'a> GtlfContext<'a> {
    // Load all the necessary resources from the world LMFAO
    pub fn from_world(world: &'a World) -> Result<Self, world::WorldBorrowMutError> {
        let graphics = world.get::<Graphics>().unwrap();
        let scene = world.get_mut::<Scene>()?;
        let pipelines = world.get_mut::<Pipelines>()?;
        let meshes = world.get_mut::<Storage<Mesh>>()?;
        let albedo_maps = world.get_mut::<Storage<AlbedoMap>>()?;
        let normal_maps = world.get_mut::<Storage<NormalMap>>()?;
        let mask_maps = world.get_mut::<Storage<MaskMap>>()?;
        let pbr_materials = world.get_mut::<Storage<PhysicallyBasedMaterial>>()?;
        let threadpool = world.get_mut::<ThreadPool>()?;

        Ok(Self {
            graphics,
            scene,
            pipelines,
            meshes,
            albedo_maps,
            normal_maps,
            mask_maps,
            pbr_materials,
            threadpool,
        })
    }
}

// These are the settings that must be given to the gltf importer so it can deserialize the scene
pub struct GltfSettings<'a> {
    // Default material that we should use when we don't have a material applied to objects
    pub fallback_material: PhysicallyBasedMaterial,

    // We can only load one scene at a time
    // If this is default, then it uses the default scene
    pub scene: Option<&'a str>,

    // Should we use async asset loading to load in buffers and textures?
    pub asynchronous: bool,
}

impl<'a> Default for GltfSettings<'a> {
    fn default() -> Self {
        Self {
            scene: None,
            asynchronous: true,
            fallback_material: PhysicallyBasedMaterial {
                albedo_map: None,
                normal_map: None,
                mask_map: None,
                bumpiness: 1.0,
                roughness: 0.5,
                metallic: 0.0,
                ambient_occlusion: 0.0,
                tint: vek::Rgb::one()
            },
        }
    }
}


// Marker type that implements asset
// Doesn't store anything on it's own; everything will be inserted into the world automatically
pub struct GltfScene;

// Can load in SINGLE .gltf JSON file
// TODO: Can load in .glb binary glTF files
// Can load in MULTIPLE .gltf files (expects the user to have defined them as asset though)

impl Asset for GltfScene {
    type Context<'ctx> = GtlfContext<'ctx>;
    type Settings<'stg> = GltfSettings<'stg>;
    type Err = gltf::Error;

    // Gtlfs can be loaded from their binary or json formats
    fn extensions() -> &'static [&'static str] {
        &["gltf"]
    }

    // Load up the GTLF scene
    fn deserialize<'c, 's>(
        data: assets::Data,
        mut context: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        // Loads the GTLF file from the loaded up bytes
        let bytes = data.bytes();
        let reader = std::io::Cursor::new(bytes);
        let gltf = gltf::Gltf::from_reader(reader)?;
        let doc = gltf.document;
        let json = doc.into_json();

        // Decompose the JSON document
        let gltf::json::Root {
            accessors,
            buffers,
            buffer_views,
            scene,
            extensions,
            cameras,
            images,
            materials,
            meshes,
            nodes,
            samplers,
            scenes,
            textures,
            ..
        } = json;

        // Get the asset loader for recrusive asset loading
        let loader = data.loader().unwrap();

        // ----------- NOTES FOR GLTF -----------
        // Buffers are just raw containers of raw byte data
        // In the engine, each buffer accessor for primitives is actually a graphics buffer by itself 
        // Buffer -> BufferView -> BufferAccessor -> Attribute[] -> Mesh
        // Buffer -> BufferView -> Image -> Texture[] -> Material
        // TODO: Implement buffer allocation sharing in Graphics API to be able to combine bufferviews as a whole buffer by themselves

        // Create a base64 decoder
        let base64 = GeneralPurpose::new(
            &alphabet::STANDARD,
            GeneralPurposeConfig::default()
        );
        
        // Map (raw) JSON buffers and store their raw byte values
        let contents = buffers.iter().map(|buffer| {
            let string = buffer.uri.as_ref().unwrap();
            let bytes = if string.starts_with("data:application/octet-stream;base64,") {
                // Data is contained within the URI itself
                let data = string.strip_prefix("data:application/octet-stream;base64,").unwrap();

                // Decode the raw base64 data
                base64.decode(data).unwrap()
            } else {
                // URI references a file that must be loaded
                let mut path = data.path().to_path_buf();
                path.pop();
                path.push(Path::new(string));

                // Load the file that contains the raw binary data
                loader.load::<Vec<u8>>(path.to_str().unwrap()).unwrap()
            };

            // Make sure we loaded the right amount of bytes
            assert_eq!(bytes.len(), buffer.byte_length as usize);
            bytes
        }).collect::<Vec<_>>();
        log::debug!("Mapped {} glTF buffers", buffers.len());
        
        // Map buffer views as buffer slices
        let views = buffer_views.iter().map(|view| {
            let buffer = &contents[view.buffer.value()];
            let len = view.byte_length as usize;
            let offset = view.byte_offset.unwrap() as usize;
            assert!(view.byte_stride.is_none());
            &buffer[offset..(offset + len)]
        }).collect::<Vec<_>>();
        log::debug!("Mapped {} glTF buffer views", buffer_views.len());

        // Map images and store their raw byte values (and extension)
        let images = images.iter().map(|image| {
            let view = &views[image.buffer_view.unwrap().value()];
            assert!(image.uri.is_none());
            let ext = &image.mime_type.as_ref().unwrap().0;
            (*view, ext.strip_prefix("image/").unwrap())
        }).collect::<Vec<_>>();
        log::debug!("Mapped {} glTF images", images.len());

        // Map buffer accessors and store their component values
        let accessors = accessors.iter().map(|accessor| {
            let view = &views[accessor.buffer_view.unwrap().value()];
            assert!(accessor.sparse.is_none());
            let offset = accessor.byte_offset as usize;
            &view[offset..]
        }).collect::<Vec<_>>();

        // Map PBR materials (map textures and their samplers as well)
        // TODO: Implement multiple texture coordinates for the mesh
        let mut cached_albedo_maps = AHashMap::<usize, Handle<AlbedoMap>>::new();
        let mut cached_normal_maps = AHashMap::<usize, Handle<NormalMap>>::new();
        let mut cached_mask_maps = AHashMap::<(Option<usize>, Option<usize>), Handle<MaskMap>>::new(); 
        let materials = materials.iter().map(|material| {
            // Decompose into Optional indices
            let pbr = &material.pbr_metallic_roughness;
            let albedo_map = pbr.base_color_texture.as_ref();
            let normal_map = material.normal_texture.as_ref();
            let metallic_roughness_map = pbr.metallic_roughness_texture.as_ref();
            let occlusion_map = material.occlusion_texture.as_ref();

            // Get the texture map / tint factors
            let tint = vek::Rgb::from_slice(&pbr.base_color_factor.0[..3]);
            let roughness = pbr.roughness_factor.0;
            let metallic = pbr.metallic_factor.0;
            let ambient_occlusion = occlusion_map.as_ref().map(|x| x.strength.0).unwrap_or(1.0);
            let bumpiness = normal_map.as_ref().map(|x| x.scale).unwrap_or(1.0);
            
            // Create or load a cached diffuse map texture
            let albedo_map = albedo_map.map(|info| {
                cached_albedo_maps.entry(info.index.value()).or_insert_with(|| {
                    let texture = &textures[info.index.value()];
                    create_material_texture(
                        context.graphics.clone(),
                        texture,
                        &images,
                        loader,
                        &mut context.albedo_maps
                    )
                }).clone()
            });

            // Create or load a cached normal map texture
            let normal_map = normal_map.map(|tex| {
                cached_normal_maps.entry(tex.index.value()).or_insert_with(|| {
                    let texture = &textures[tex.index.value()];
                    create_material_texture(
                        context.graphics.clone(),
                        texture,
                        &images,
                        loader,
                        &mut context.normal_maps
                    )
                }).clone()
            });

            //let mask_map = (metallic_roughness_map.xor(optb), occlusion_map)

            PhysicallyBasedMaterial {
                albedo_map,
                normal_map,
                mask_map: None,
                bumpiness,
                roughness,
                metallic,
                ambient_occlusion,
                tint
            }
        }).map(|material| {
            context.pbr_materials.insert(material)
        }).collect::<Vec<Handle<PhysicallyBasedMaterial>>>();
        
        // Map entities (meshes). Returns array of meshes and materials
        let meshes = meshes.iter().map(|meshes| {
            todo!()
        }).collect::<Vec<_>>();

        // Get the scene that we will load
        let scene = settings.scene.map(|defined| scenes
            .iter()
            .filter_map(|x| x.name.as_ref().map(|y| (x, y)))
            .find(|(_, name)| defined == *name)
            .map(|(scene, _)| scene)
            .unwrap()
            .clone()
        ).or(scene.as_ref().map(|i| scenes[i.value()].clone())).unwrap();

        // Create storages that will contain the *handles* that are themselves stored within storages   
        let mut meshes: Vec<Handle<Mesh>> = Vec::new();
        let mut albedo_maps: Vec<Handle<AlbedoMap>> = Vec::new();
        let mut normal_maps: Vec<Handle<NormalMap>> = Vec::new();
        let mut mask_maps: Vec<Handle<MaskMap>> = Vec::new();
        let mut pbr_materials: Vec<Handle<PhysicallyBasedMaterial>> = Vec::new();

        // Keep track of the renderable entities that we will add
        let mut entities: Vec<(coords::Position, coords::Rotation, crate::Surface<PhysicallyBasedMaterial>, crate::Renderer)> = Vec::new();

        // Iterate over the nodes now (or objects in the scene)
        for node in nodes {
            // Convert translation, rotation, and scale to proper components
            let position = node.translation.map(|slice |coords::Position::at_xyz_array(slice)).unwrap_or_default();
            let rotation = node.rotation.map(|quat| coords::Rotation::new_xyzw_array(quat.0)).unwrap_or_default();

            // Handle renderable entities
            if let Some(mesh) = node.mesh {
            }
        }

        // Add the entities into the world
        context.scene.extend_from_iter(entities);
        
        todo!();
    }
}

fn create_material_texture<T: Texel + ImageTexel>(
    graphics: Graphics,
    texture: &gltf::json::Texture,
    images: &[(&[u8], &str)],
    loader: &assets::Assets,
    storage: &mut Storage<Texture2D<T>>,
) -> Handle<Texture2D<T>> {
    let (bytes, extension) = &images[texture.source.value()];
                    
    let data = Data::new(
        texture.name.as_ref().map(|x| &**x).unwrap_or("Untitled Texture"),
        extension,
        Arc::from(bytes.to_vec()),
        Path::new(""),
        Some(loader)
    );

    let texture = Texture2D::<T>::deserialize(
        data,
        graphics,
        TextureImportSettings {
            sampling: Some(SamplerSettings {
                filter: SamplerFilter::Nearest,
                wrap: SamplerWrap::Repeat,
                mipmaps: SamplerMipMaps::Auto,
            }),
            mode: TextureMode::Dynamic,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            scale: TextureScale::Default,
            mipmaps: TextureMipMaps::Disabled,
        }
    ).unwrap();

    storage.insert(texture)
}