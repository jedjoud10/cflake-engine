use crate::{
    Chunk, ChunkManager, ChunkState, ChunkViewer, Terrain, TerrainMaterial,
    TerrainSettings,
};
use ahash::{AHashMap, AHashSet};

use coords::{Position, Rotation, Scale};
use ecs::{Entity, Scene};

use graphics::{
    ActivePipeline, ComputePass, DrawIndexedIndirect, DrawIndexedIndirectBuffer, Graphics,
};
use math::OctreeDelta;
use rendering::{IndirectMesh, Renderer, Surface};
use utils::{Storage, Time, ThreadPool};
use world::{user, System, World};

// Dynamically generate the chunks based on camera position
fn update(world: &mut World) {
    // Tries to find a chunk viewer and the terrain generator
    let terrain = world.get_mut::<Terrain>();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let viewer = scene.find_mut::<(&Entity, &mut ChunkViewer, &Position, &Rotation)>();

    // If we don't have terrain, don't do shit
    let Ok(mut _terrain) = terrain else {
        return;
    };

    // Get the terrain chunk manager and terrain settings
    let terrain = &mut *_terrain;
    let mut manager = &mut terrain.manager;
    let settings = &terrain.settings;

    // If we don't have a chunk viewer, don't do shit
    let Some((entity, _, viewer_position, viewer_rotation)) = viewer else {
        manager.viewer = None;
        return;
    };

    // Set the main viewer location and fetches the oldvalue
    let mut added = false;
    let new = **viewer_position;
    let old = if let Some((_, old, _)) = &mut manager.viewer {
        std::mem::replace(old, new)
    } else {
        manager.viewer = Some((*entity, new, **viewer_rotation));
        added = true;
        new
    };

    // Check if it moved since last frame
    if added || new != old {
        // Regenerate the octree and detect diffs
        let OctreeDelta {
            added,
            removed
        } = manager.octree.compute(new, settings.radius);

        // Set the chunk state to "free" so we can reuse it
        for coord in removed {
            let entity = manager.entities.remove(&coord).unwrap();
            let mut entry = scene.entry_mut(entity).unwrap();
            let chunk = entry.get_mut::<Chunk>().unwrap();
            chunk.state = ChunkState::Free;
        }

        // Try to re-use "free" chunks first 
        let query = scene
            .query_mut::<(
                &mut Chunk,
                &mut Position,
                &mut Scale,
                &Entity,
                &mut Surface<TerrainMaterial>,
            )>()
            .into_iter()
            .filter(|(x, _, _, _, _)| x.state == ChunkState::Free);

        // Set the "dirty" state for newly added chukns
        /*
        for ((chunk, position, scale, entity, surface), node) in query.zip(added) {
            chunk.state = ChunkState::Dirty;
            chunk.node = node;
            surface.visible = false;
            **position = coords.as_::<f32>() * (terrain.settings.size as f32);
            chunk.priority = 0.0;
            manager.entities.insert(coords, *entity);
        }
        */
    }

    // Update priority for EACH chunk, even if the viewer did not move
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();

    scene.query_mut::<(&mut Chunk, &Position)>().for_each(
        &mut threadpool,
         |(chunk, position)| {
            chunk.priority = (1.0 / viewer_position.distance(**position).max(1.0)) * 10.0;
            chunk.priority *= viewer_rotation
                .forward()
                .dot((**position - viewer_position).normalized())
                * 5.0;
            chunk.priority = chunk.priority.clamp(0.0f32, 1000.0f32);
    }, 512);
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}
