use world::ecs::component::{ComponentQuerySet, ComponentQueryParams, ComponentQuery};
use world::math::shapes::ShapeType;
use world::rendering::advanced::storages::Buffer;
use world::World;
use world::terrain::editing::Edit;

use crate::components::{DynamicEdit, Transform};
use crate::globals;
// A system that will handle terrain edits
fn run(world: &mut World, data: ComponentQuerySet) {
    // Get the terrain global
    if let Ok(terrain) = world.globals.get_mut::<globals::Terrain>() {
        // Editing manager
        let terrain = &mut *terrain;

        // Handle the dynamic edits
        handle_dynamic_edits(data, terrain);

        let chunks_to_regenerate = terrain.editer.get_influenced_chunks(&terrain.manager.octree.inner);
        if !chunks_to_regenerate.is_empty() {
            // Regenerate the specified chunks
            for coords in chunks_to_regenerate {
                terrain.regenerate_chunk(coords);
            }
            // Also set the packed edits since we will need to update them on the GPU
            let packed = terrain.editer.convert();
            terrain.generator.ssbo_edits.storage_mut().write(&packed);
        }
        terrain.manager.update_priorities();
    }
}

// Updates the terrain edits to reflect on the modification applied onto the dynamic edits
fn handle_dynamic_edits(mut data: Vec<ComponentQuery>, terrain: &mut globals::Terrain) {
    // Debug
    let query = data.get_mut(0).unwrap();
    for (_, components) in query.all.iter_mut() {
        let transform = components.get_mut::<Transform>().unwrap();
        transform.position = transform.position + vek::Vec3::unit_y() * 0.01;
        let edit = components.get_mut::<DynamicEdit>().unwrap();
        edit.edit.params.color.r = edit.edit.params.color.r.wrapping_add(1);
        match &mut edit.edit.shape {
            ShapeType::Sphere(s) => s.radius += 0.01,
            _ => {}
        }
    }
    // Add the actual edits first
    for (_, components) in query.delta.added.iter_mut() {
        let dynamic_edit = components.get_mut::<DynamicEdit>().unwrap();
        // We are going to be mutating the edit anyways so just spawn the relative one
        dynamic_edit.key = terrain.editer.edit(dynamic_edit.edit.clone());
    }
    // Only update the edits if we finished generating the base terrain
    if !terrain.manager.chunks_generating.is_empty() || terrain.manager.must_update_octree || !terrain.manager.chunks_to_remove.is_empty() { return } 

    // Loop through every dynamic edit and update it's corresponding terrain edit
    for (_, components) in query.all.iter() {
        let transform = components.get::<Transform>().unwrap();
        let dynamic_edit = components.get::<DynamicEdit>().unwrap();
        if components.was_mutated::<DynamicEdit>().unwrap() || components.was_mutated::<Transform>().unwrap() {
            // Update
            let stored_edit = terrain.editer.get_mut(dynamic_edit.key).unwrap();
            // Get the shape type, but offset it with the transform's position
            let mut shape = dynamic_edit.edit.shape.clone();
            match &mut shape {
                ShapeType::Cuboid(cube) => cube.center += transform.position,
                ShapeType::Sphere(sphere) => sphere.center += transform.position,
                ShapeType::VerticalCapsule(capsule) => capsule.center += transform.position,
            };
        
            *stored_edit = Edit {
                shape,
                params: dynamic_edit.edit.params.clone(),
            }; 
        }
    }
}

// Create the system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder(&mut world.events.ecs)
        .query(ComponentQueryParams::default().link::<Transform>().link::<DynamicEdit>())
        .event(run)
        .build()
        .unwrap();
}
