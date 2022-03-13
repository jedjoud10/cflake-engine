use std::time::Instant;

use crate::{
    components::{Chunk, Collider, ColliderGeometry, Renderer, RigidBody, Transform},
    globals::ChunkGenerationState,
};
use rapier3d::prelude::{ColliderMaterial, RigidBodyType};
use world::{
    ecs::{
        component::{ComponentQueryParameters, ComponentQuerySet},
        entity::{ComponentLinkingGroup, ComponentUnlinkGroup},
    },
    rendering::{
        basics::{material::Material, mesh::Mesh},
        pipeline::Handle,
    },
    terrain::mesher::{Mesher, MesherSettings},
    World,
};

// The mesher systems' update loop
fn run(world: &mut World, mut data: ComponentQuerySet) {
    let query = &mut data.get_mut(0).unwrap().all;
    let terrain = world.globals.get_mut::<crate::globals::Terrain>();
    if Instant::now().saturating_duration_since(world.time.current.begin_instant).as_millis() > 1 {
        return;
    }
    if let Ok(mut terrain) = terrain {
        // We can only create the mesh of a single chunk per frame
        if let ChunkGenerationState::EndVoxelDataGeneration(key, true) = terrain.manager.current_chunk_state {
            // Get the chunk component from the specific chunk
            let linked = query.get_mut(&key).unwrap();
            let coords = linked.get_mut::<Chunk>().unwrap().coords;
            // Either way, we're going to be updating/generating the mesh so might as well make the mesher now
            let mesher = Mesher::new(
                coords,
                &terrain.generator.stored,
                MesherSettings {
                    interpolation: true,
                    skirts: true,
                },
            );
            // Generate the mesh and add it to the chunk entity
            let mesh = mesher.build();
            let mesh = world.pipeline.meshes.insert(mesh);
            let _cloned = mesh.clone();

            if !linked.is_linked::<Renderer>() {
                // Generate the new component and link it
                let group = create_chunk_renderer_linking_group(mesh, terrain.manager.material.clone(), terrain.manager.physics);
                world.ecs.link(key, group).unwrap();
            } else {
                // The renderer is already linked, we just need to update the mesh
                // Valid renderer
                let renderer = linked.get_mut::<Renderer>().unwrap();
                renderer.mesh = mesh;
            }

            // We have created voxel data for this chunk, and it is valid (it contains a surface)
            terrain.manager.chunks_generating.remove(&coords);
            // Switch states
            terrain.manager.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
        } else if let ChunkGenerationState::EndVoxelDataGeneration(key, false) = terrain.manager.current_chunk_state {
            // Get the chunk component from the specific chunk
            let linked = query.get_mut(&key).unwrap();
            let _chunk = linked.get_mut::<Chunk>().unwrap();
            // Remove the chunk's renderer if it had one
            if world.ecs.entities.get(key).unwrap().is_linked::<crate::components::Renderer>() {
                let mut unlink_group = ComponentUnlinkGroup::default();
                unlink_group.unlink::<crate::components::Renderer>().unwrap();
                world.ecs.unlink_components(key, unlink_group).unwrap();
            }

            // The chunk ID is the same, but we do not have a surface
            // We still gotta update the current chunk state though
            terrain.manager.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
        }
    }
}

// Create a new linking group that contains a renderer with a specific mesh
fn create_chunk_renderer_linking_group(mesh: Handle<Mesh>, material: Handle<Material>, physics: bool) -> ComponentLinkingGroup {
    // First time we link the renderer
    let mut group = ComponentLinkingGroup::default();

    // Add the renderer
    let renderer = Renderer {
        mesh: mesh.clone(),
        material,
        ..Default::default()
    };
    group.link(renderer).unwrap();

    if physics {
        // Add the collider
        let collider = Collider::new(ColliderGeometry::mesh(mesh, 100.0), ColliderMaterial::new(100.0, 0.0));
        group.link(collider).unwrap();
        
        // Add the static rigidbody
        let rigidbody = RigidBody::new(RigidBodyType::Static);
        group.link(rigidbody).unwrap();
    }

    group
}
// Create a mesher system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .query(ComponentQueryParameters::default().link::<Transform>().link::<Chunk>())
        .query(ComponentQueryParameters::default().link::<Transform>().link::<Renderer>().link::<Chunk>())
        .event(run)
        .build()
}
