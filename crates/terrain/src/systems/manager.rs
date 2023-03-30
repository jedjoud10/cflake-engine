use crate::{
    Chunk, ChunkCoords, ChunkState, ChunkViewer, Terrain,
    TerrainMaterial, TerrainSettings,
};
use ahash::AHashSet;
use assets::Assets;
use ecs::{Entity, Position, Rotation, Scale, Scene};
use graphics::{
    BufferMode, BufferUsage, DrawIndexedIndirect,
    DrawIndexedIndirectBuffer, Graphics, Normalized, TriangleBuffer,
    VertexBuffer, XYZW,
};
use rendering::{Mesh, Renderer, Surface, IndirectMesh, AttributeBuffer, attributes, Pipelines};
use utils::{Storage, Time};
use world::{user, System, World, post_user};

// Creates the terrain if there was terrain settings present
fn init(world: &mut World) {
    if let Some(settings) = world.remove::<TerrainSettings>() {
        world.insert(Storage::<TerrainMaterial>::default());
        let graphics = world.get::<Graphics>().unwrap();
        let assets = world.get::<Assets>().unwrap();
        let mut indirect_meshes = world.get_mut::<Storage<IndirectMesh>>().unwrap();
        let mut indirect_buffers = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
        let mut vertices = world.get_mut::<Storage<AttributeBuffer<attributes::Position>>>().unwrap();
        let mut triangles = world.get_mut::<Storage<TriangleBuffer<u32>>>().unwrap();
        let mut materials = world.get_mut::<Storage<TerrainMaterial>>().unwrap();
        let mut pipelines = world.get_mut::<Pipelines>().unwrap();

        let terrain = Terrain::new(
            &graphics, 
            &assets,
            settings,
            &mut indirect_meshes,
            &mut indirect_buffers,
            &mut vertices,
            &mut triangles,
            &mut materials,
            &mut pipelines,
        );
        
        drop(graphics);
        drop(assets);
        drop(indirect_meshes);
        drop(indirect_buffers);
        drop(vertices);
        drop(triangles);
        drop(materials);
        drop(pipelines);

        world.insert(terrain);
    }
}

// Creates the appropriate components for creating a new chunk entity
fn create_chunk_components(
    terrain: &mut Terrain,
    coords: ChunkCoords,
) -> (Position, Renderer, Surface<TerrainMaterial>, Chunk) {
    // Get a mesh from the terrain mesh pool
    let (mesh, free) =
        terrain.indirect_meshes.iter_mut().find(|(_, free)| *free).unwrap();
    *free = false;

    // Create the surface for rendering
    let mut surface = Surface::indirect(
        mesh.clone(),
        terrain.material.clone(),
        terrain.id.clone(),
    );

    // Hide the surface at first
    surface.visible = false;

    // Create a renderer an a position component
    let renderer = Renderer::default();
    let position =
        Position::from(coords.as_::<f32>() * terrain.size as f32);

    // Offset chunk coords to remove negatives
    let positive  = coords.map(|x| (x + terrain.chunk_render_distance as i32) as u32);
    let max = vek::Vec3::broadcast(terrain.chunk_render_distance * 2 + 1);

    let index = (positive.x + positive.y * max.x + positive.z * max.x * max.y) as usize;
    let allocation = index / terrain.chunks_per_allocation;
    log::trace!("terrain manager: using allocation {allocation} for chunk {coords}");

    // Create the chunk component
    let chunk = Chunk {
        state: ChunkState::Pending,
        coords,
        allocation,
    };

    // Return the components of the new chunk
    (position, renderer, surface, chunk)
}

// Dynamically generate the chunks based on camera position
fn update(world: &mut World) {
    // Tries to find a chunk viewer and the terrain generator
    let time = world.get::<Time>().unwrap();
    let terrain = world.get_mut::<Terrain>();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let viewer =
        scene.find_mut::<(&Entity, &mut ChunkViewer, &Position)>();

    // If we don't have terrain, don't do shit
    let Ok(mut terrain) = terrain else {
        return;
    };

    // If we don't have a chunk viewer, don't do shit
    let Some((entity, _, position)) = viewer else {
        terrain.viewer = None;
        return;
    };

    // Set the main viewer location and fetches the oldvalue
    let mut added = false;
    let new = (**position / vek::Vec3::broadcast(terrain.size as f32))
        .round()
        .as_::<i32>();
    let old = if let Some((_, old)) = &mut terrain.viewer {
        std::mem::replace(old, new)
    } else {
        terrain.viewer = Some((*entity, new));
        added = true;
        new
    };

    // Keep a hashset of all the chunks around the viewer
    let mut chunks = AHashSet::<ChunkCoords>::new();

    // Check if it moved since last frame
    if  added {
        let distance = terrain.chunk_render_distance as i32;
        for x in -distance..distance {
            for y in -1..1 {
                for z in -distance..distance {
                    let chunk = vek::Vec3::new(x, y, z);
                    let view = terrain.viewer.unwrap().1;
                    chunks.insert(chunk + view);
                }
            }
        }

        // Detect the chunks that we should remove and remove them
        let removed = terrain
            .chunks
            .difference(&chunks)
            .cloned()
            .collect::<Vec<_>>();
        for coords in removed {
            log::trace!("terrain manager: remove chunk with coords {coords}");
            let entity = terrain.entities.remove(&coords).unwrap();
            if scene.contains(entity) {
                scene.remove(entity);
            }
        }

        // Get the removed surfaces and add the mesh and indirect buffer handles back to the pool
        for surface in scene.removed::<Surface<TerrainMaterial>>() {
            let (_, free) = terrain
                .indirect_meshes
                .iter_mut()
                .find(|(handle, _)| {
                    *handle == surface.mesh.clone()
                })
                .unwrap();
            *free = true;
        }

        // Detect the chunks that we must generate and add them
        let added = chunks
            .difference(&terrain.chunks)
            .cloned()
            .collect::<Vec<_>>();
        let entities =
            scene.extend_from_iter(added.iter().map(|coords| {
                log::trace!("terrain manager: add chunk with coords {coords}");
                create_chunk_components(&mut terrain, *coords)
            }));

        // Add the new chunks into the chunk entities of the terrain
        for (coords, entity) in added.iter().zip(entities.iter()) {
            terrain.entities.insert(*coords, *entity);
        }

        terrain.chunks = chunks;
    }
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system.insert_init(init).after(post_user);
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
