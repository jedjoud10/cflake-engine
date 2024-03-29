use std::{path::Path, sync::Arc};

use crate::{AlbedoMap, MaskMap, Mesh, NormalMap, PbrMaterial, Pipelines, SubSurface, Surface, CullResult};
use ahash::AHashMap;
use assets::{Asset, Data};
use base64::{
    alphabet,
    engine::{GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use bytemuck::Contiguous;
use coords::HierarchyManager;
use dashmap::DashMap;
use ecs::{Entity, Scene};
use gltf::json::accessor::{ComponentType, Type};
use graphics::{
    texture2d_from_raw, BufferMode, BufferUsage, Graphics, ImageTexel, Normalized, RawTexels,
    SamplerFilter, SamplerSettings, SamplerWrap, Texel, Texture, Texture2D, TextureImportSettings,
    TextureMipMaps, TextureScale, TextureUsage, TextureViewSettings, R, RGBA,
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use utils::{Handle, Storage};
use world::{Read, World, Write};

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
    pub pbr_materials: Write<'a, Storage<PbrMaterial>>,
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
        let pbr_materials = world.get_mut::<Storage<PbrMaterial>>()?;

        Ok(Self {
            graphics,
            scene,
            pipelines,
            meshes,
            albedo_maps,
            normal_maps,
            mask_maps,
            pbr_materials,
        })
    }
}

// These are the settings that must be given to the gltf importer so it can deserialize the scene
pub struct GltfSettings<'a> {
    // Default material that we should use when we don't have a material applied to objects
    pub fallback: PbrMaterial,

    // We can only load one scene at a time
    // If this is default, then it uses the default scene
    pub scene: Option<&'a str>,
}

impl<'a> Default for GltfSettings<'a> {
    fn default() -> Self {
        Self {
            scene: None,
            fallback: PbrMaterial {
                albedo_map: None,
                normal_map: None,
                mask_map: None,
                bumpiness_factor: 1.0,
                roughness_factor: 0.5,
                metallic_factor: 0.0,
                ambient_occlusion_factor: 0.0,
                tint: vek::Rgb::one(),
                scale: vek::Extent2::one(),
            },
        }
    }
}

// Marker type that implements asset
// Doesn't store anything on it's own; everything will be inserted into the world automatically
pub struct GltfScene;

// Can load in SINGLE .gltf JSON file
// Can load in MULTIPLE .gltf files (expects the user to have defined them as asset though)

impl Asset for GltfScene {
    type Context<'ctx> = GtlfContext<'ctx>;
    type Settings<'stg> = GltfSettings<'stg>;
    type Err = gltf::Error;

    // Gtlfs can be loaded from their binary or json formats
    fn extensions() -> &'static [&'static str] {
        &["gltf", "glb"]
    }

    // Load up the GTLF scene
    fn deserialize(
        data: assets::Data,
        mut context: Self::Context<'_>,
        settings: Self::Settings<'_>,
    ) -> Result<Self, Self::Err> {
        // Loads the GTLF file from the loaded up bytes
        let bytes = data.bytes();
        let reader = std::io::Cursor::new(bytes);

        // Second variable is the binary data in case we load from a GLB
        let (json, bin) = match data.extension() {
            // Load the scene from a GLTF file
            "gltf" => {
                let gltf = gltf::Gltf::from_reader(reader)?;
                let doc = gltf.document;
                (doc.into_json(), None)
            }

            // Load the scene from a GLB file
            "glb" => {
                let glb = gltf::Glb::from_reader(reader)?;
                let cursor = std::io::Cursor::new(&glb.json);
                let gltf = gltf::Gltf::from_reader(cursor)?;
                let doc = gltf.document;
                (doc.into_json(), glb.bin)
            }

            _ => panic!(),
        };

        // Decompose the JSON document
        let gltf::json::Root {
            accessors,
            buffers,
            buffer_views,
            scene,

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
        let base64 = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());

        // Map (raw) JSON buffers and store their raw byte values
        let mut offset = 0usize;
        let mapped_contents = buffers
            .iter()
            .map(|buffer| {
                // Handle reading buffers from URI or raw bytes directly
                let bytes = if let Some(uri) = buffer.uri.as_ref() {
                    const PREFIX: &str = "data:application/octet-stream;base64,";

                    if uri.starts_with(PREFIX) {
                        // Data is contained within the URI itself
                        let data = uri.strip_prefix(PREFIX).unwrap();

                        // Decode the raw base64 data
                        base64.decode(data).unwrap()
                    } else {
                        // URI references a file that must be loaded
                        let mut path = data.path().to_path_buf();
                        path.pop();
                        path.push(Path::new(uri));

                        // Load the file that contains the raw binary data
                        loader.load::<Vec<u8>>(path.to_str().unwrap()).unwrap()
                    }
                } else {
                    // Load the binary data oui oui
                    let bin = bin.as_ref().unwrap();
                    let bytes = bin[offset..(offset + buffer.byte_length.0 as usize)].to_vec();
                    offset += buffer.byte_length.0 as usize;
                    bytes
                };

                // Make sure we loaded the right amount of bytes
                assert_eq!(bytes.len(), buffer.byte_length.0 as usize);
                bytes
            })
            .collect::<Vec<_>>();
        log::debug!("Mapped {} glTF buffers", buffers.len());

        // Map buffer views as buffer slices
        let mapped_views = buffer_views
            .iter()
            .map(|view| {
                let buffer = &mapped_contents[view.buffer.value()];
                let len = view.byte_length.0 as usize;
                let offset = view.byte_offset.unwrap_or_default().0 as usize;
                assert!(view.byte_stride.is_none());
                &buffer[offset..(offset + len)]
            })
            .collect::<Vec<_>>();
        log::debug!("Mapped {} glTF buffer views", buffer_views.len());

        // Map images and store their raw byte values (and extension)
        let mapped_images = images
            .iter()
            .map(|image| {
                let view = &mapped_views[image.buffer_view.unwrap().value()];
                assert!(image.uri.is_none());
                let ext = &image.mime_type.as_ref().unwrap().0;
                (*view, ext.strip_prefix("image/").unwrap())
            })
            .collect::<Vec<_>>();
        log::debug!("Mapped {} glTF images", mapped_images.len());

        // Map buffer accessors and store their component values
        let mapped_accessors = accessors
            .iter()
            .map(|accessor| {
                let view = &mapped_views[accessor.buffer_view.unwrap().value()];
                assert!(accessor.sparse.is_none());
                let offset = accessor.byte_offset.unwrap_or_default().0 as usize;
                let _type = accessor.type_.as_ref().unwrap();
                let min = accessor.min.as_ref();
                let max = accessor.max.as_ref();

                let generic_component_type = accessor.component_type.as_ref().unwrap();
                (
                    &view[offset..],
                    (_type, &generic_component_type.0),
                    (min, max),
                )
            })
            .collect::<Vec<_>>();

        // TODO:
        // Convert the textures to hashmaps w the settings they should be created with
        // Create rayon scope that will take those hashmaps and convert their inner settings to act textures

        // Map PBR textures first in other threads
        let cached_albedo_maps = Arc::new(DashMap::<usize, AlbedoMap>::new());
        let cached_normal_maps = Arc::new(DashMap::<usize, NormalMap>::new());
        let cached_mask_maps = Arc::new(DashMap::<(Option<usize>, Option<usize>), MaskMap>::new());
        let graphics = context.graphics.clone();

        // Load the assets in other threads
        rayon::scope(|s| {
            for material in materials.iter() {
                // Decompose into Optional indices
                let pbr = material.pbr_metallic_roughness.clone();
                let albedo_map = pbr.base_color_texture.clone();
                let normal_map = material.normal_texture.clone();
                let metallic_roughness_map = pbr.metallic_roughness_texture.clone();
                let occlusion_map = material.occlusion_texture.clone();
                let textures = &textures;
                let mapped_images = &mapped_images;
                let samplers = &samplers;
                let cached_albedo_maps = cached_albedo_maps.clone();
                let cached_normal_maps = cached_normal_maps.clone();
                let cached_mask_maps = cached_mask_maps.clone();

                // Create or load a cached diffuse map texture
                if let Some(info) = albedo_map {
                    let graphics = graphics.clone();
                    s.spawn(move |_| {
                        cached_albedo_maps
                            .entry(info.index.value())
                            .or_insert_with(|| {
                                let texture = &textures[info.index.value()];
                                create_material_texture(graphics, texture, samplers, mapped_images)
                            });
                    });
                }

                // Create or load a cached normal map texture
                if let Some(tex) = normal_map {
                    let graphics = graphics.clone();
                    s.spawn(move |_| {
                        cached_normal_maps
                            .entry(tex.index.value())
                            .or_insert_with(|| {
                                let texture = &textures[tex.index.value()];
                                create_material_texture(graphics, texture, samplers, mapped_images)
                            });
                    });
                }

                // Create or load a cached mask map texture
                // r: ambient occlusion, g: roughness, b: metallic
                (metallic_roughness_map.is_some() || occlusion_map.is_some()).then(|| {
                    let graphics = graphics.clone();
                    s.spawn(move |_s| {
                        let metallic_roughness_map =
                            metallic_roughness_map.map(|x| x.index.value());
                        let occlusion_map = occlusion_map.map(|x| x.index.value());

                        cached_mask_maps
                            .entry((metallic_roughness_map, occlusion_map))
                            .or_insert_with(|| {
                                let metallic_roughness_map =
                                    metallic_roughness_map.map(|x| &textures[x]);
                                let occlusion_map = occlusion_map.map(|x| &textures[x]);

                                create_material_mask_texture(
                                    graphics,
                                    metallic_roughness_map,
                                    occlusion_map,
                                    samplers,
                                    mapped_images,
                                )
                            });
                    });
                });
            }
        });
        log::debug!("Mapping of {} glTF materials", materials.len());

        // Unwraps an Arc and panics if it returns an error
        let cached_albedo_maps = Arc::try_unwrap(cached_albedo_maps).map_err(|_| ()).unwrap();
        let cached_normal_maps = Arc::try_unwrap(cached_normal_maps).map_err(|_| ()).unwrap();
        let cached_mask_maps = Arc::try_unwrap(cached_mask_maps).map_err(|_| ()).unwrap();

        // Convert the textures to their appropriate handles
        let cached_albedo_maps = cached_albedo_maps
            .into_iter()
            .map(|(i, map)| (i, context.albedo_maps.insert(map)))
            .collect::<AHashMap<usize, Handle<AlbedoMap>>>();
        let cached_normal_maps = cached_normal_maps
            .into_iter()
            .map(|(i, map)| (i, context.normal_maps.insert(map)))
            .collect::<AHashMap<usize, Handle<NormalMap>>>();
        let cached_mask_maps = cached_mask_maps
            .into_iter()
            .map(|(i, map)| (i, context.mask_maps.insert(map)))
            .collect::<AHashMap<(Option<usize>, Option<usize>), Handle<MaskMap>>>();

        // Map PBR materials (map textures and their samplers as well)
        // TODO: Implement multiple texture coordinates for the mesh
        let mapped_materials = materials
            .iter()
            .map(|material| {
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

                // Get the handles NOW!!!
                let albedo_map = albedo_map.map(|x| cached_albedo_maps[&x.index.value()].clone());
                let normal_map = normal_map.map(|x| cached_normal_maps[&x.index.value()].clone());
                let mask = (
                    metallic_roughness_map.map(|x| x.index.value()),
                    occlusion_map.map(|x| x.index.value()),
                );
                let mask_map = cached_mask_maps.get(&mask).cloned();

                PbrMaterial {
                    albedo_map,
                    normal_map,
                    mask_map,
                    bumpiness_factor: bumpiness,
                    roughness_factor: roughness,
                    metallic_factor: metallic,
                    ambient_occlusion_factor: ambient_occlusion,
                    tint,
                    scale: vek::Extent2::one(),
                }
            })
            .map(|material| context.pbr_materials.insert(material))
            .collect::<Vec<Handle<PbrMaterial>>>();

        // Map meshes and create their handles
        type CachedMeshKey = (usize, Option<usize>, Option<usize>, Option<usize>, usize);
        let cached_meshes = DashMap::<CachedMeshKey, Mesh>::new();
        let graphics = context.graphics.clone();
        let mapped_meshes = meshes
            .par_iter()
            .map(|mesh| {
                let mut meshes = Vec::<CachedMeshKey>::new();

                for primitive in mesh.primitives.iter() {
                    // Get the accessor indices for the attributes used by this mesh
                    let mut key = (usize::MAX, None, None, None, usize::MAX);
                    for (semantic, attribute) in primitive.attributes.iter() {
                        let index = Some(attribute.value());
                        let semantic = semantic.as_ref().unwrap();

                        match semantic {
                            gltf::Semantic::Positions => key.0 = attribute.value(),
                            gltf::Semantic::Normals => key.1 = index,
                            gltf::Semantic::Tangents => key.2 = index,
                            gltf::Semantic::TexCoords(_) => key.3 = index,
                            _ => {}
                        }
                    }
                    key.4 = primitive.indices.unwrap().value();

                    // Create a new mesh if the accessors aren't cached
                    // TODO: Implement multi-buffer per allocation support to optimize this
                    cached_meshes.entry(key).or_insert_with(|| {
                        // Create buffers and AABB
                        let (mut positions, aabb) = create_positions_vec(&mapped_accessors[key.0]);

                        // TODO: Generate normals / tangents / uvs if missing
                        let mut normals = key
                            .1
                            .map(|index| create_normals_vec(&mapped_accessors[index]));
                        let mut tangents = key
                            .2
                            .map(|index| create_tangents_vec(&mapped_accessors[index]));
                        let mut tex_coords = Some(key
                            .3
                            .map(|index| create_tex_coords_vec(&mapped_accessors[index]))
                            .unwrap_or_else(|| vec![vek::Vec2::zero(); positions.len()]));
                        let mut triangles = create_triangles_vec(&mapped_accessors[key.4]);

                        // Optionally generate the tangents
                        if let (Some(normals), Some(tex_coords)) =
                            (normals.as_ref(), tex_coords.as_ref())
                        {
                            tangents = Some(
                                super::compute_tangents(
                                    &positions, normals, tex_coords, &triangles,
                                )
                                .unwrap(),
                            );
                        }

                        let mut temp_positions = Some(positions.as_mut_slice());
                        let mut temp_normals = normals.as_deref_mut();
                        let mut temp_tangents = tangents.as_deref_mut();
                        let mut temp_tex_coords = tex_coords.as_deref_mut();

                        // Optimize the mesh after we load it
                        super::optimize(
                            true,
                            true,
                            true,
                            &mut temp_positions,
                            &mut temp_normals,
                            &mut temp_tangents,
                            &mut temp_tex_coords,
                            &mut triangles,
                        );

                        // Create a new mesh for the accessors used
                        Mesh::from_slices(
                            &graphics.clone(),
                            BufferMode::Dynamic,
                            BufferUsage::COPY_DST,
                            Some(&positions),
                            normals.as_deref(),
                            tangents.as_deref(),
                            tex_coords.as_deref(),
                            &triangles,
                            aabb,
                        )
                        .unwrap()
                    });
                    meshes.push(key);
                }

                meshes
            })
            .collect::<Vec<Vec<CachedMeshKey>>>();

        // Convert mapped meshes into their actual handles
        let cached_meshes = cached_meshes
            .into_iter()
            .map(|(id, mesh)| (id, context.meshes.insert(mesh)))
            .collect::<AHashMap<_, _>>();
        let mapped_meshes = mapped_meshes
            .into_iter()
            .map(|vec| {
                vec.into_iter()
                    .map(|id| cached_meshes[&id].clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Get the scene that we will load
        let scene = settings
            .scene
            .map(|defined| {
                scenes
                    .iter()
                    .filter_map(|x| x.name.as_ref().map(|y| (x, y)))
                    .find(|(_, name)| defined == *name)
                    .map(|(scene, _)| scene)
                    .unwrap()
                    .clone()
            })
            .or(scene.as_ref().map(|i| scenes[i.value()].clone()))
            .unwrap();

        // Add a "default" PBR material
        let default = context.pbr_materials.insert(settings.fallback);

        // Keep track of the nodes we must evaluate
        let mut eval = scene
            .nodes
            .iter()
            .map(|node| (&nodes[node.value()], None))
            .collect::<Vec<(&gltf::json::Node, Option<Entity>)>>();

        // Number of entities we will add
        let mut count = 0;

        // PBR material id
        let id = context.pipelines.get::<PbrMaterial>().unwrap();

        // Iterate until there are no more nodes to pass through
        while let Some((node, parent)) = {
            if !eval.is_empty() {
                Some(eval.remove(0))
            } else {
                None
            }
        } {
            // Convert translation, rotation, and scale to proper components
            let position = node
                .translation
                .map(vek::Vec3::from)
                .unwrap_or(vek::Vec3::zero());
            let rotation = node
                .rotation
                .map(|quat| vek::Quaternion::from_vec4(vek::Vec4::from(quat.0)))
                .unwrap_or(vek::Quaternion::identity());
            let scale = node
                .scale
                .map(|scale| {
                    // Biggest scalar in the vector
                    let uniform = vek::Vec3::from_slice(&scale).reduce_partial_max();

                    // Non-uniform scale isn't supported in the engine
                    if scale.iter().any(|&x| x != uniform) {
                        log::warn!(
                            "Non-uniform scale is not supported in the engine. Given scale: {:?}",
                            scale
                        );
                    }

                    uniform
                })
                .unwrap_or(1.0f32);

            // For now, we only handle mesh entities and empty entities
            let name = node.name.as_ref().map(|x| x.as_str()).unwrap_or("Untitled Node");
            let entity = if let Some(mesh_index) = node.mesh {
                let mesh = &meshes[mesh_index.value()];
                let meshes = &mapped_meshes[mesh_index.value()];

                // Sub-Surfaces that we must render
                let mut subsurfaces = Vec::<SubSurface<PbrMaterial>>::new();

                // Create the sub-surfaces of the entity (mesh)
                for (submesh_index, primitive) in mesh.primitives.iter().enumerate() {
                    let mesh = &meshes[submesh_index];
                    let material = primitive
                        .material
                        .as_ref()
                        .map(|i| mapped_materials[i.value()].clone())
                        .unwrap_or(default.clone());
                    subsurfaces.push(SubSurface {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        culled: CullResult::Visible,
                        visible: true,
                        shadow_culled: 0,
                        shadow_caster: true,
                    });
                }

                // Create a proper surface
                let surface = Surface::from_iter(subsurfaces, id);

                match parent {
                    // Local coordinates if we have a parent
                    Some(_) => {
                        // Add the renderable entity
                        context.scene.insert((
                            coords::LocalPosition::from(position),
                            coords::LocalRotation::from(rotation),
                            coords::LocalScale::from(scale),
                            coords::Position::default(),
                            coords::Rotation::default(),
                            coords::Scale::default(),
                            surface,
                            crate::Renderer::default(),
                            ecs::Named(name.to_string()),
                        ))
                    }

                    // Global coordinates if we do not have a parent
                    None => {
                        // Add the renderable entity
                        context.scene.insert((
                            coords::Position::from(position),
                            coords::Rotation::from(rotation),
                            coords::Scale::from(scale),
                            surface,
                            crate::Renderer::default(),
                            ecs::Named(name.to_string()),
                        ))
                    }
                }
            } else {
                match parent {
                    // Local coordinates if we have a parent
                    Some(_) => {
                        // Add the renderable entity
                        context.scene.insert((
                            coords::LocalPosition::from(position),
                            coords::LocalRotation::from(rotation),
                            coords::LocalScale::from(scale),
                            coords::Position::default(),
                            coords::Rotation::default(),
                            coords::Scale::default(),
                            ecs::Named(name.to_string()),
                        ))
                    }

                    // Global coordinates if we do not have a parent
                    None => {
                        // Add the renderable entity
                        context.scene.insert((
                            coords::Position::from(position),
                            coords::Rotation::from(rotation),
                            coords::Scale::from(scale),
                            ecs::Named(name.to_string()),
                        ))
                    }
                }
            };

            // Attach this entity to its parent if it had any
            if let Some(parent) = parent {
                context.scene.attach(entity, parent).unwrap();
            }

            // Pass along the children nodes
            if let Some(children) = node.children.as_ref() {
                eval.extend(children.iter().map(|x| (&nodes[x.value()], Some(entity))));
            }

            count += 1;
        }

        log::debug!("Loaded {} entities into the world", count);
        log::debug!("Loaded {} unique meshes into the world", meshes.len());
        log::debug!(
            "Loaded {} unique material instances into the world",
            mapped_materials.len()
        );
        log::debug!(
            "Loaded {} albedo maps into the world",
            cached_albedo_maps.len()
        );
        log::debug!(
            "Loaded {} normal maps into the world",
            cached_normal_maps.len()
        );
        log::debug!("Loaded {} mask maps into the world", cached_mask_maps.len());

        Ok(GltfScene)
    }
}

type MinMax<'b> = (Option<&'b gltf::json::Value>, Option<&'b gltf::json::Value>);

type Value<'a, 'b> = &'a (&'b [u8], (&'b Type, &'b ComponentType), MinMax<'b>);

// Create the position vertices required by all meshes and also create an AABB
fn create_positions_vec(value: Value) -> (Vec<vek::Vec4<f32>>, Option<math::Aabb<f32>>) {
    let (bytes, (_type, _component), (min, max)) = value;
    assert_eq!(**_type, Type::Vec3);
    assert_eq!(**_component, ComponentType::F32);
    let data: &[vek::Vec3<f32>] = bytemuck::cast_slice(bytes);

    // Create the positions vector
    let vec = data
        .iter()
        .map(|vec3| vec3.with_w(0.0))
        .collect::<Vec<vek::Vec4<f32>>>();

    // Create the AABB using the position min and max given by the JSON
    let aabb: Option<math::Aabb<f32>> = if let (Some(min), Some(max)) = (min, max) {
        match (min, max) {
            (gltf::json::Value::Array(min), gltf::json::Value::Array(max)) => {
                assert_eq!(min.len(), 3);
                assert_eq!(max.len(), 3);

                fn deserialize_vec3_array(vec: &Vec<gltf::json::Value>) -> vek::Vec3<f32> {
                    let slice: &[gltf::json::Value; 3] = vec.as_slice().try_into().unwrap();

                    let slice = slice.clone().map(|value| match value {
                        gltf::json::Value::Number(number) => number.as_f64().unwrap() as f32,
                        _ => panic!(),
                    });

                    vek::Vec3::from_slice(&slice)
                }

                let min = deserialize_vec3_array(min);
                let max = deserialize_vec3_array(max);

                Some(math::Aabb::<f32> { min, max })
            }
            _ => None,
        }
    } else {
        None
    };

    (vec, aabb)
}

// Create the normal coordinates required by the PBR material
fn create_normals_vec(value: Value) -> Vec<vek::Vec4<i8>> {
    let (bytes, (_type, _component), _) = value;
    assert_eq!(**_type, Type::Vec3);
    assert_eq!(**_component, ComponentType::F32);
    let data: &[vek::Vec3<f32>] = bytemuck::cast_slice(bytes);
    data.iter()
        .map(|vec3| (vec3.with_w(1.0) * 127.0).as_::<i8>())
        .collect::<Vec<vek::Vec4<i8>>>()
}

// Create the UV coordinates required by the PBR material
fn create_tex_coords_vec(value: Value) -> Vec<vek::Vec2<f32>> {
    let (bytes, (_type, _component), _) = value;
    assert_eq!(**_type, Type::Vec2);
    assert_eq!(**_component, ComponentType::F32);
    let data: &[vek::Vec2<f32>] = bytemuck::cast_slice(bytes);
    data.to_vec()
}

// Create the tangents (if supplied) required by the PBR material
fn create_tangents_vec(value: Value) -> Vec<vek::Vec4<i8>> {
    let (bytes, (_type, _component), _) = value;
    assert_eq!(**_type, Type::Vec4);
    assert_eq!(**_component, ComponentType::F32);
    let data: &[vek::Vec4<f32>] = bytemuck::cast_slice(bytes);
    data.iter()
        .map(|vec4| (vec4 * 127.0).as_::<i8>())
        .collect::<Vec<vek::Vec4<i8>>>()
}

// Convert raw bytes to a triangle index buffer
fn create_triangles_vec(value: Value) -> Vec<[u32; 3]> {
    let (bytes, (_type, _component), _) = value;
    assert_eq!(**_type, Type::Scalar);

    match _component {
        ComponentType::U16 => {
            let data: &[u16] = bytemuck::cast_slice(bytes);
            data.chunks_exact(3)
                .map(|x| [x[0] as u32, x[1] as u32, x[2] as u32])
                .collect()
        }
        ComponentType::U32 => {
            let data: &[[u32; 3]] = bytemuck::cast_slice(bytes);
            data.to_vec()
        }
        _ => panic!("jed is dumb"),
    }
}

// Get sampling parameters from a sampling index of a texture
fn sampling(
    samplers: &[gltf::json::texture::Sampler],
    sampler: Option<gltf::json::Index<gltf::json::texture::Sampler>>,
) -> SamplerSettings {
    sampler
        .map(|index| {
            let sampler = &samplers[index.value()];

            let mag_filter = sampler
                .mag_filter
                .map(|x| match x.unwrap() {
                    gltf::texture::MagFilter::Nearest => SamplerFilter::Nearest,
                    gltf::texture::MagFilter::Linear => SamplerFilter::Linear,
                })
                .unwrap_or(SamplerFilter::Linear);

            let (min_filter, mip_filter) = sampler
                .min_filter
                .map(|x| match x.unwrap() {
                    gltf::texture::MinFilter::Nearest => {
                        (SamplerFilter::Nearest, SamplerFilter::Nearest)
                    }
                    gltf::texture::MinFilter::Linear => {
                        (SamplerFilter::Linear, SamplerFilter::Nearest)
                    }

                    gltf::texture::MinFilter::NearestMipmapNearest => {
                        (SamplerFilter::Nearest, SamplerFilter::Nearest)
                    }
                    gltf::texture::MinFilter::LinearMipmapNearest => {
                        (SamplerFilter::Linear, SamplerFilter::Nearest)
                    }
                    gltf::texture::MinFilter::NearestMipmapLinear => {
                        (SamplerFilter::Nearest, SamplerFilter::Linear)
                    }
                    gltf::texture::MinFilter::LinearMipmapLinear => {
                        (SamplerFilter::Linear, SamplerFilter::Linear)
                    }
                })
                .unwrap_or((SamplerFilter::Linear, SamplerFilter::Linear));

            let wrap_u = map_wrapping_mode(sampler.wrap_s.unwrap());
            let wrap_v = map_wrapping_mode(sampler.wrap_t.unwrap());

            SamplerSettings {
                mag_filter,
                min_filter,
                mip_filter,
                wrap_u,
                wrap_v,
                ..Default::default()
            }
        })
        .unwrap_or(SamplerSettings::default())
}

fn map_wrapping_mode(wrap: gltf::json::texture::WrappingMode) -> SamplerWrap {
    match wrap {
        gltf::texture::WrappingMode::ClampToEdge => SamplerWrap::ClampToEdge,
        gltf::texture::WrappingMode::MirroredRepeat => SamplerWrap::MirrorRepeat,
        gltf::texture::WrappingMode::Repeat => SamplerWrap::Repeat,
    }
}

// Create a texture used for a material used in the glTF scene
// This should be executed in multiple threads for maximum efficency
fn create_material_texture<T: Texel + ImageTexel>(
    graphics: Graphics,
    texture: &gltf::json::Texture,
    samplers: &[gltf::json::texture::Sampler],
    images: &[(&[u8], &str)],
) -> Texture2D<T> {
    let (bytes, extension) = &images[texture.source.value()];
    let name = texture.name.as_deref().unwrap_or("Untitled Texture");

    let data = Data::new(
        name,
        extension,
        Arc::from(bytes.to_vec()),
        Path::new(name),
        None,
    );

    let sampler = sampling(samplers, texture.sampler);

    Texture2D::<T>::deserialize(
        data,
        graphics,
        TextureImportSettings {
            sampling: Some(sampler),
            views: &[TextureViewSettings::whole::<
                <Texture2D<T> as Texture>::Region,
            >()],
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            scale: TextureScale::Default,
            mipmaps: TextureMipMaps::Manual { mips: &[] },
        },
    )
    .unwrap()
}

// r: ambient occlusion, g: roughness, b: metallic
fn create_material_mask_texture(
    graphics: Graphics,
    metallic_roughness: Option<&gltf::json::Texture>,
    occlusion: Option<&gltf::json::Texture>,
    samplers: &[gltf::json::texture::Sampler],
    images: &[(&[u8], &str)],
) -> MaskMap {
    assert!(metallic_roughness.is_some() || occlusion.is_some());

    let sampler = match (metallic_roughness, occlusion) {
        (None, Some(a)) | (Some(a), None) | (Some(a), Some(_)) => sampling(samplers, a.sampler),
        _ => panic!(),
    };

    let metallic_roughness = metallic_roughness.map(|i| (&images[i.source.value()], i));
    let occlusion = occlusion.map(|i| (&images[i.source.value()], i));

    let metallic_roughness = metallic_roughness.map(|((bytes, extension), texture)| {
        let name = texture.name.as_deref().unwrap_or("Untitled Texture");

        let data = Data::new(
            name,
            extension,
            Arc::from(bytes.to_vec()),
            Path::new(name),
            None,
        );

        RawTexels::<RGBA<Normalized<u8>>>::deserialize(data, (), TextureScale::Default).unwrap()
    });

    let occlusion = occlusion.map(|((bytes, extension), texture)| {
        let name = texture.name.as_deref().unwrap_or("Untitled Texture");

        let data = Data::new(
            name,
            extension,
            Arc::from(bytes.to_vec()),
            Path::new(name),
            None,
        );

        RawTexels::<R<Normalized<u8>>>::deserialize(data, (), TextureScale::Default).unwrap()
    });

    let mut extent = metallic_roughness.as_ref().map(|x| x.1);

    if let Some(extent2) = extent {
        if let Some(occlusion) = &occlusion {
            assert_eq!(occlusion.1, extent2);
            extent = Some(occlusion.1);
        }
    }

    // r: ambient occlusion, g: roughness, b: metallic
    let data: Vec<vek::Vec4<u8>> = match (metallic_roughness, occlusion) {
        (None, Some(raw)) => raw
            .0
            .into_iter()
            .map(|ao| vek::Vec4::new(ao, 255, 255, 0))
            .collect(),
        (Some(raw), None) => raw
            .0
            .into_iter()
            .map(|vec| vek::Vec4::new(255, vec.y, vec.z, 0))
            .collect(),
        (Some(raw0), Some(raw1)) => raw0
            .0
            .into_iter()
            .zip(raw1.0.into_iter())
            .map(|(vec, ao)| vek::Vec4::new(ao, vec.y, vec.z, 0))
            .collect(),
        _ => todo!(),
    };

    let raw = RawTexels(data, extent.unwrap());

    texture2d_from_raw(
        graphics,
        TextureImportSettings {
            sampling: Some(sampler),
            views: &[TextureViewSettings::whole::<<MaskMap as Texture>::Region>()],
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            scale: TextureScale::Default,
            mipmaps: TextureMipMaps::Manual { mips: &[] },
        },
        raw,
    )
    .unwrap()
}
