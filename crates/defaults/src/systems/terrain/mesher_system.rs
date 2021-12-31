use core::global::callbacks::CallbackType;
use std::collections::HashSet;

use super::MesherSystem;
use ecs::SystemData;
use math::octrees::OctreeNode;
use others::callbacks::{MutCallback, OwnedCallback};
use rendering::ShaderUniformsGroup;
use terrain::ChunkCoords;
ecs::impl_systemdata!(MesherSystem);

// Loop though every entity, checking if one of them got their Voxel Data generated
fn entity_update(data: &mut SystemData<MesherSystem>, entity: &ecs::Entity) {
    // Get the chunk component and check for valid Voxel Data
    let entity_id = entity.id.clone();
    let chunk = core::global::ecs::component::<terrain::Chunk>(entity).unwrap();
    // If this chunk is not a pending chunk, then we cannot generate it's model
    let i = std::time::Instant::now();
    if data.pending_chunks.contains(&chunk.coords) && chunk.voxel_data.is_some() {
        if let Option::Some(voxel_data) = &chunk.voxel_data.as_ref().unwrap() {
            // If the voxel data is valid, create the model for this chunk
            let tmodel = terrain::mesher::generate_model(&voxel_data, chunk.coords, true);
            let material = data.material;
            let model = rendering::Model::combine(tmodel.model, tmodel.skirts_model);          
            let coords = chunk.coords;  
            // The actual creation of the GPU model is asynchronous
            rendering::pipec::task(rendering::RenderTask::ModelCreate(model)).with_callback(CallbackType::RenderingGPUObjectCallback(OwnedCallback::new(move |(_, model_id)| {
                // Create the model on the GPU
                let mut group = ShaderUniformsGroup::new();
                group.set_i32("node_depth", coords.depth as i32);
                // Since each chunk starts without a renderer, we must manually add the renderer component
                let mut linkings = ecs::ComponentLinkingGroup::new();
                // Create a renderer with the correct model and materials
                let renderer = crate::components::Renderer::default()
                    .set_wireframe(true)
                    .set_fading_animation(true)
                    .set_model(model_id)
                    .set_material(material)
                    .set_shader_uniforms(group);
                linkings.link::<crate::components::Renderer>(renderer).unwrap();
                core::global::batch::batch_add(0, true, core::global::ecs::link_components(entity_id, linkings));                
            })).create());            
        } else {
            // We generated the voxel data, but there was no surface, so no need to create the model
        };
        data.pending_chunks.remove(&chunk.coords);
        //println!("{}", i.elapsed().as_millis());
    }
}

// When a Chunk gets added to the mesher system, we will constantly wait for it's Voxel Data generation
fn entity_added(data: &mut SystemData<MesherSystem>, entity: &ecs::Entity) {
    let chunk_coords = core::global::ecs::component::<terrain::Chunk>(entity).unwrap().coords.clone();
    data.pending_chunks.insert(chunk_coords);
}

// When a Chunk gets removed from the mesher system, we will not create it's mesh
fn entity_removed(data: &mut SystemData<MesherSystem>, entity: &ecs::Entity) {
    let chunk_coords = &core::global::ecs::component::<terrain::Chunk>(entity).unwrap().coords;
    data.pending_chunks.remove(chunk_coords);
}

// Create the Mesher System
pub fn system(material: rendering::GPUObjectID) {
    let mesher_system_data = MesherSystem {
        material,
        pending_chunks: HashSet::new(),
    };
    core::global::ecs::add_system(mesher_system_data, || {
        // Create a system
        let mut system = ecs::System::new();
        // Link some components to the system
        system.link::<crate::components::Transform>();
        system.link::<terrain::Chunk>();
        // And link the events
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        system.event(ecs::SystemEventType::EntityAdded(entity_added));
        system.event(ecs::SystemEventType::EntityRemoved(entity_removed));
        // Return the newly made system
        system
    });
}
