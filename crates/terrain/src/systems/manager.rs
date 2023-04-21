use crate::{
    Chunk, ChunkCoords, ChunkState, ChunkViewer, Terrain,
    TerrainMaterial, TerrainSettings, ChunkManager,
};
use ahash::{AHashSet, AHashMap};

use coords::{Position, Rotation};
use ecs::{Entity, Scene};

use graphics::{ComputePass, Graphics, DrawIndexedIndirectBuffer, DrawIndexedIndirect, ActivePipeline};
use rendering::{
    Renderer, Surface, IndirectMesh,
};
use utils::{Time, Storage};
use world::{user, System, World};

// Dynamically generate the chunks based on camera position
fn update(world: &mut World) {
    // Tries to find a chunk viewer and the terrain generator
    let terrain = world.get_mut::<Terrain>();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let viewer =
        scene.find_mut::<(&Entity, &mut ChunkViewer, &Position, &Rotation)>();

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
    let new = (**viewer_position
        / vek::Vec3::broadcast(settings.size as f32))
    .round()
    .as_::<i32>();
    let old = if let Some((_, old, _)) = &mut manager.viewer {
        std::mem::replace(old, new)
    } else {
        manager.viewer = Some((*entity, new, **viewer_rotation));
        added = true;
        new
    };
    
    // Check if it moved since last frame
    if added || new != old {
        // Keep a hashset of all the chunks around the viewer
        let mut chunks = AHashSet::<ChunkCoords>::new();

        // Generate the chunks around ze player
        let distance = settings.chunk_render_distance as i32;
        let vertical = ((distance as f32) / 3.0).ceil() as i32;
        for x in -distance..=distance {
            for y in -vertical..=vertical {
                for z in -distance..=distance {
                    let chunk = vek::Vec3::new(x, y, z);
                    let view = manager.viewer.unwrap().1;
                    chunks.insert(chunk + view);
                }
            }
        }

        // Detect the chunks that we should remove and "remove" them
        let removed = manager
            .chunks
            .difference(&chunks)
            .cloned();

        // Detect the chunks that we must generate and add them
        let added = chunks
            .difference(&manager.chunks)
            .cloned();
        
        // Set the chunk state to "free" and reset the value of the indirect buffers
        for coord in removed {
            let entity = manager.entities.remove(&coord).unwrap();
            let mut entry = scene.entry_mut(entity).unwrap();
            let chunk = entry.get_mut::<Chunk>().unwrap();
            chunk.state = ChunkState::Free;
        }
        
        // We won't actually create new entities, only update the old ones
        let query = scene.query_mut::<(&mut Chunk, &mut Position, &Entity, &mut Surface<TerrainMaterial>)>()
            .into_iter()
            .filter(|(x, _, _, _)| 
                x.state == ChunkState::Free
            );

        // Set the "dirty" state for newly added chukns
        for ((chunk, position, entity, surface), coords) in query.zip(added) {
            chunk.state = ChunkState::Dirty;
            chunk.coords = coords;
            surface.visible = false;
            **position = coords.as_::<f32>() * (terrain.settings.size as f32);
            chunk.priority = 0.0;
            manager.entities.insert(coords, *entity);
        }

        manager.chunks = chunks;
    }

    // Update priority for EACH chunk, even if the viewer did not move
    for (chunk, position) in scene.query_mut::<(&mut Chunk, &Position)>() {
        chunk.priority = (1.0 / viewer_position.distance(**position).max(1.0)) * 10.0;
        chunk.priority *= viewer_rotation.forward().dot((**position - viewer_position).normalized()) * 5.0;
        chunk.priority = chunk.priority.clamp(0.0f32, 1000.0f32);
    }
}

// Adds/removes the chunk entities from the world
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system)
        .after(user);
}