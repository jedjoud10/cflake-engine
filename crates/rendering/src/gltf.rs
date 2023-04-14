use std::io::BufReader;

use assets::Asset;
use graphics::Graphics;
use utils::{Storage, Handle};
use crate::Mesh;

// These are the context values that must be given to the GltfScene to load it
pub struct GtlfContext<'a> {
    pub graphics: &'a Graphics,

    // Storages that will contain the newly loaded GTLF data
    pub meshes: &'a mut Storage<Mesh>,

}

// These are the settings that must be given to the gltf importer so it can deserialize the scene
pub struct GltfSettings<'stg> {
    // We can only load one scene at a time
    pub scene_index: &'stg usize,
}

// This structure contains all data that will be filled when loading a glTF file
// This can either load a .gltf or .glb file, and it will deserialize the scene and store it in the struct
// At the moment, we will be able to load ONE scene only
// At the moment, only SINGLE file gltfs can be loaded
pub struct GltfScene {
    // Handles for the newly created meshes, textures, and materials
    pub meshes: Vec<(String, Handle<Mesh>)>,
}

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

        todo!()
    }
}