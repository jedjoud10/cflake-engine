use crate::{
    Chunk, ChunkCoords, ChunkState, ChunkViewer, Terrain,
    TerrainMaterial,
};
use ahash::AHashSet;
use ecs::{Entity, Position, Rotation, Scale, Scene};
use graphics::{
    BufferMode, BufferUsage, DrawIndexedIndirect,
    DrawIndexedIndirectBuffer, Graphics, Normalized, TriangleBuffer,
    VertexBuffer, XYZW,
};
use rendering::{Mesh, Renderer, Surface};
use utils::{Storage, Time};
use world::{user, System, World};

// Creates the default material storage for the terrain material
fn init(world: &mut World) {
    world.insert(Storage::<TerrainMaterial>::default());
}

// Creates the appropriate components for creating a new chunk entity
fn create_chunk_components(
    terrain: &mut Terrain,
    coords: ChunkCoords,
) -> (Position, Renderer, Surface<TerrainMaterial>, Chunk) {
    // Get a mesh from the terrain mesh pool
    let (mesh, free) =
        terrain.meshes.iter_mut().find(|(_, free)| *free).unwrap();
    *free = false;

    // Get an indirect buffer from the terrain indirect buffer pool
    let (indirect, free) = terrain
        .indirect_buffers
        .iter_mut()
        .find(|(_, free)| *free)
        .unwrap();
    *free = false;

    // Create the surface for rendering
    let surface = Surface::indirect(
        mesh.clone(),
        terrain.material.clone(),
        indirect.clone(),
        terrain.id.clone(),
    );
    let renderer = Renderer::default();
    let position =
        Position::from(coords.as_::<f32>() * terrain.size as f32);

    // Create the chunk component
    let chunk = Chunk {
        state: ChunkState::Pending,
        coords,
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
    let new = (**position
        / vek::Vec3::broadcast(terrain.size as f32))
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
    if old != new || added {
        let distance = terrain.chunk_render_distance as i32;
        for x in -distance..distance {
            for y in -distance..distance {
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
            log::debug!("Remove terrain chunk with coords {coords}");
            let entity = terrain.entities.remove(&coords).unwrap();
            if scene.contains(entity) {
                scene.remove(entity);
            }
        }

        // Get the removed surfaces and add the mesh and indirect buffer handles back to the pool
        for surface in scene.removed::<Surface<TerrainMaterial>>() {
            let (_, free) = terrain
                .indirect_buffers
                .iter_mut()
                .find(|(handle, _)| {
                    *handle
                        == surface.indirect.as_ref().cloned().unwrap()
                })
                .unwrap();
            *free = true;

            let (_, free) = terrain
                .meshes
                .iter_mut()
                .find(|(handle, _)| *handle == surface.mesh)
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
                log::debug!("Add terrain chunk with coords {coords}");
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
    system.insert_init(init).before(user);
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
