use std::{io::BufReader, iter::repeat};

use assets::Asset;
use ecs::Scene;
use graphics::Graphics;
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
            asynchronous: true
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
        context: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        // Loads the GTLF file from the loaded up bytes
        let bytes = data.bytes();
        let reader = std::io::Cursor::new(bytes);
        let gltf = gltf::Gltf::from_reader(reader)?;
        let doc = gltf.document;
        let json = doc.into_json();

        // Get the asset loader for recrusive asset loading
        let loader = data.loader().unwrap();

        // Get the scene that we will load
        let scene = settings.scene.map(|defined| json
            .scenes
            .iter()
            .filter_map(|x| x.name.as_ref().map(|y| (x, y)))
            .find(|(_, name)| defined == *name)
            .map(|(scene, _)| scene)
            .unwrap()
            .clone()
        ).or(json.scene.as_ref().map(|i| json.scenes[i.value()].clone())).unwrap();

        // Gltf mesh -> renderable entity
        // Gltf mesh primitive -> actual mesh + corresponding surface

        // Create storages that will contain the *handles* that are themselves stored within storages   
        let mut meshes: Vec<Option<Handle<Mesh>>> = Vec::new();
        let mut albedo_maps: Vec<Option<Handle<AlbedoMap>>> = Vec::new();
        let mut normal_maps: Vec<Option<Handle<NormalMap>>> = Vec::new();
        let mut mask_maps: Vec<Option<Handle<MaskMap>>> = Vec::new();
        let mut pbr_materials: Vec<Option<Handle<PhysicallyBasedMaterial>>> = Vec::new();

        // Keep track of the renderable entities that we will add
        let mut entities: Vec<(coords::Position, coords::Rotation, crate::Surface<PhysicallyBasedMaterial>, crate::Renderer)> = Vec::new();

        // Iterate over the nodes now (or objects in the scene)
        for node in json.nodes {
            // Convert translation, rotation, and scale to proper components
            let position = node.translation.map(|slice |coords::Position::at_xyz_array(slice)).unwrap_or_default();
            let rotation = node.rotation.map(|quat| coords::Rotation::new_xyzw_array(quat.0)).unwrap_or_default();

            // Handle renderable entities
            if let Some(mesh) = node.mesh {
                let i = mesh.value();
                let primitives = &json.meshes[i].primitives;

                // Loop over the "surfaces" 
                for surface in primitive {

                }

                /*
                // Get the graphics mesh (if cached) and JSON mesh
                let mut out_mesh_handle = &mut meshes[i];
                let doc_mesh = &json.meshes[i];

                // Create a new graphics mesh if needed
                if out_mesh_handle.is_none() {
                    let primitive = doc_mesh
                    let (attribute, accessor) in 
                    for (semantic, accessor) in primitive.attributes.iter() {
                        let semantic = semantic.as_ref().unwrap();


                        match semantic {
                            gltf::Semantic::Positions => todo!(),
                            gltf::Semantic::Normals => todo!(),
                            gltf::Semantic::Tangents => todo!(),
                            gltf::Semantic::TexCoords(_) => todo!(),
                            _ => { log::warn!("GLTF semantic not supported") }
                        }
                    }
                }

                // Check if we have a mesh already created for this entity
                let handle = out_mesh_handle.as_ref().unwrap();

                // Primitives are basically surfaces
                for primitive in doc_mesh.primitives.iter() {
                    // Keep track of the mesh attributes that our mesh *must* support

                    /*
                    for (semantic, accessor) in primitive.attributes.iter() {
                        let semantic = semantic.as_ref().unwrap();


                        match semantic {
                            gltf::Semantic::Positions => todo!(),
                            gltf::Semantic::Normals => todo!(),
                            gltf::Semantic::Tangents => todo!(),
                            gltf::Semantic::TexCoords(_) => todo!(),
                            _ => { log::warn!("GLTF semantic not supported") }
                        }
                    }
                    */
                }
                */
            }
        }

        // Add the entities into the world
        context.scene.extend_from_iter(entities);
        
        todo!();
    }
}