use crate::{
    Chunk, ChunkCoords, ChunkState, ChunkViewer, Terrain,
    TerrainMaterial, TerrainSettings, Vertices, Triangles, VoxelGenerator, MeshGenerator, MemoryManager, ChunkManager,
};
use ahash::AHashSet;
use assets::Assets;
use ecs::{Entity, Position, Rotation, Scale, Scene};
use graphics::{
    BufferMode, BufferUsage, DrawIndexedIndirect,
    DrawIndexedIndirectBuffer, Graphics, Normalized, TriangleBuffer,
    VertexBuffer, XYZW,
};
use rendering::{
    attributes, AttributeBuffer, IndirectMesh, Mesh, Pipelines,
    Renderer, Surface,
};
use utils::{Storage, Time};
use world::{post_user, user, System, World};

// Creates the terrain if there was terrain settings present
fn init(world: &mut World) {
    if let Some(settings) = world.remove::<TerrainSettings>() {
        // Add materials and fetch them
        world.insert(Storage::<TerrainMaterial>::default());
        let mut materials = world.get_mut::<Storage<TerrainMaterial>>().unwrap();
        let mut pipelines = world.get_mut::<Pipelines>().unwrap();
        
        // Get graphics API and assets
        let graphics = world.get::<Graphics>().unwrap();
        let assets = world.get::<Assets>().unwrap();
        
        // Get indirect mesh storage
        let mut indirect_meshes =
            world.get_mut::<Storage<IndirectMesh>>().unwrap();
        
        // Get indirect buffer storage
        let mut indirect_buffers = world
            .get_mut::<Storage<DrawIndexedIndirectBuffer>>()
            .unwrap();
        
        // Get indirect vertices and triangle buffers
        let mut vertices = world.get_mut::<Storage<Vertices>>().unwrap();
        let mut triangles = world.get_mut::<Storage<Triangles>>().unwrap();

        // Create a voxel generator
        let voxel = VoxelGenerator::new(
            &assets,
            &graphics,
            &settings
        );

        // Create a mesh generator
        let mesher = MeshGenerator::new(
            &assets,
            &graphics,
            &settings
        );

        // Create the memory manager
        let memory = MemoryManager::new(
            &assets,
            &graphics,
            &mut vertices,
            &mut triangles,
            &settings
        );

        // Create the chunk manager
        let manager = ChunkManager::new(
            &assets,
            &graphics,
            &settings,
            &memory,
            &mut indirect_meshes,
            &mut indirect_buffers,
            &mut materials,
            &mut pipelines,
        );

        // Combine all the terrain generator composites into the one terrain generator struct
        let terrain = Terrain {
            voxelizer: todo!(),
            mesher,
            memory,
            manager,
            settings,
        };

        // Drop resources to be able to insert terrain into world
        drop(graphics);
        drop(assets);
        drop(indirect_meshes);
        drop(indirect_buffers);
        drop(vertices);
        drop(triangles);
        drop(materials);
        drop(pipelines);

        // Insert terrain
        world.insert(terrain);
    }
}

// Initializes the terrain
pub fn system(system: &mut System) {
    system.insert_init(init).after(post_user);
}
