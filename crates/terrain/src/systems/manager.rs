use crate::{
    Chunk, ChunkCoords, ChunkState, ChunkViewer, Terrain,
    TerrainMaterial, TerrainSettings, ChunkManager,
};
use ahash::AHashSet;
use assets::Assets;
use coords::{Position, Rotation, Scale};
use ecs::{Entity, Scene};
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

// Creates the appropriate components for creating a new chunk entity
fn create_chunk_components(
    viewer: &Position,
    coords: ChunkCoords,
    manager: &ChunkManager,
    settings: &TerrainSettings,
) -> (Position, Renderer, Surface<TerrainMaterial>, Chunk) {
    // Offset chunk coords to remove negatives
    let positive = coords
        .map(|x| (x + settings.chunk_render_distance as i32) as u32);
    let max =
        vek::Vec3::broadcast(settings.chunk_render_distance as u32 * 2 + 1);
    let index = (positive.x
        + positive.y * max.x
        + positive.z * max.x * max.y) as usize;

    // Get a mesh from the terrain mesh pool
    let mesh = &manager.indirect_meshes[index];

    // Create the surface for rendering
    let mut surface = Surface::indirect(
        mesh.clone(),
        manager.material.clone(),
        manager.id.clone(),
    );

    // Hide the surface at first
    surface.visible = false;

    // Create a renderer an a position component
    let renderer = Renderer::default();
    let position =
        Position::from(coords.as_::<f32>() * settings.size as f32);
    let allocation = index / settings.chunks_per_allocation;


    // Create the chunk component
    let chunk = Chunk {
        state: ChunkState::Pending,
        coords,
        allocation,
        index: index,
        priority: viewer.distance(*position),
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
    let Ok(mut _terrain) = terrain else {
        return;
    };

    // Get the terrain chunk manager and terrain settings
    let terrain = &mut *_terrain;
    let mut manager = &mut terrain.manager;
    let settings = &terrain.settings;

    // If we don't have a chunk viewer, don't do shit
    let Some((entity, _, position)) = viewer else {
        manager.viewer = None;
        return;
    };

    // Set the main viewer location and fetches the oldvalue
    let mut added = false;
    let new = (**position
        / vek::Vec3::broadcast(settings.size as f32))
    .round()
    .as_::<i32>();
    let old = if let Some((_, old)) = &mut manager.viewer {
        std::mem::replace(old, new)
    } else {
        manager.viewer = Some((*entity, new));
        added = true;
        new
    };

    // Keep a hashset of all the chunks around the viewer
    let mut chunks = AHashSet::<ChunkCoords>::new();

    // Check if it moved since last frame
    if added {
        // Generate the chunks around ze player
        let distance = settings.chunk_render_distance as i32;
        for x in -distance..distance {
            for y in -distance..distance {
                for z in -distance..distance {
                    let chunk = vek::Vec3::new(x, y, z);
                    let view = manager.viewer.unwrap().1;
                    chunks.insert(chunk + view);
                }
            }
        }

        // Detect the chunks that we should remove and remove them
        let removed = manager
            .chunks
            .difference(&chunks)
            .cloned()
            .collect::<Vec<_>>();
        for coords in removed {
            let entity = manager.entities.remove(&coords).unwrap();
            if scene.contains(entity) {
                scene.remove(entity);
            }
        }

        // Get the removed surfaces and add the mesh and indirect buffer handles back to the pool
        for surface in scene.removed::<Surface<TerrainMaterial>>() {}

        // Detect the chunks that we must generate and add them
        let added = chunks
            .difference(&manager.chunks)
            .cloned()
            .collect::<Vec<_>>();
        let entities =
            scene.extend_from_iter(added.iter().map(|coords| {
                create_chunk_components(position, *coords, &manager, &settings)
            }));

        // Add the new chunks into the chunk entities of the terrain
        for (coords, entity) in added.iter().zip(entities.iter()) {
            manager.entities.insert(*coords, *entity);
        }

        manager.chunks = chunks;
    }
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
