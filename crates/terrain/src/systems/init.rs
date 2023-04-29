use crate::{
    ChunkManager, LayeredAlbedoMap, LayeredMaskMap, LayeredNormalMap, MemoryManager, MeshGenerator,
    Terrain, TerrainMaterial, TerrainSettings, Triangles, Vertices, VoxelGenerator,
};

use assets::Assets;

use ecs::Scene;
use graphics::{DrawIndexedIndirectBuffer, Graphics};
use rendering::{IndirectMesh, Pipelines};
use utils::{Storage};
use world::{post_user, System, World};

// Creates the terrain if there was terrain settings present
fn init(world: &mut World) {
    if let Some(mut settings) = world.remove::<TerrainSettings>() {
        // Add materials and fetch them
        world.insert(Storage::<TerrainMaterial>::default());
        world.insert(Storage::<LayeredAlbedoMap>::default());
        world.insert(Storage::<LayeredNormalMap>::default());
        world.insert(Storage::<LayeredMaskMap>::default());

        let mut layered_albedo_maps = world.get_mut::<Storage<LayeredAlbedoMap>>().unwrap();
        let mut layered_normal_maps = world.get_mut::<Storage<LayeredNormalMap>>().unwrap();
        let mut layered_mask_maps = world.get_mut::<Storage<LayeredMaskMap>>().unwrap();
        let mut materials = world.get_mut::<Storage<TerrainMaterial>>().unwrap();
        let mut scene = world.get_mut::<Scene>().unwrap();
        let mut pipelines = world.get_mut::<Pipelines>().unwrap();

        // Get graphics API and assets
        let graphics = world.get::<Graphics>().unwrap();
        let assets = world.get::<Assets>().unwrap();

        // Get indirect buffer storage
        let mut indirect_buffers = world
            .get_mut::<Storage<DrawIndexedIndirectBuffer>>()
            .unwrap();

        // Get indirect vertices and triangle buffers
        let mut vertices = world.get_mut::<Storage<Vertices>>().unwrap();
        let mut triangles = world.get_mut::<Storage<Triangles>>().unwrap();

        // Create a voxel generator
        let voxelizer = VoxelGenerator::new(&assets, &graphics, &mut settings);

        // Create a mesh generator
        let mesher = MeshGenerator::new(&assets, &graphics, &settings);

        // Create the memory manager
        let memory = MemoryManager::new(
            &assets,
            &graphics,
            &mut vertices,
            &mut triangles,
            &mut indirect_buffers,
            &settings
        );

        // Create the chunk manager
        let manager = ChunkManager::new(
            &assets,
            &graphics,
            &mut settings,
            &memory,
            &mut scene,
            &mut materials,
            &mut layered_albedo_maps,
            &mut layered_normal_maps,
            &mut layered_mask_maps,
            &mut pipelines,
        );

        // Combine all the terrain generator composites into the one terrain generator struct
        let terrain = Terrain {
            voxelizer,
            mesher,
            memory,
            manager,
            settings,
        };

        // Drop resources to be able to insert terrain into world
        drop(graphics);
        drop(assets);
        drop(indirect_buffers);
        drop(vertices);
        drop(triangles);
        drop(materials);
        drop(layered_albedo_maps);
        drop(layered_normal_maps);
        drop(layered_mask_maps);
        drop(pipelines);
        drop(scene);

        // Insert terrain
        world.insert(terrain);
    }
}

// Initializes the terrain
pub fn system(system: &mut System) {
    system.insert_init(init).after(post_user);
}
