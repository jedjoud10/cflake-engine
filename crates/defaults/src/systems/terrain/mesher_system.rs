use core::global::callbacks::CallbackType;
use std::collections::HashSet;

use ecs::SystemData;
use math::octrees::OctreeNode;
use others::callbacks::MutCallback;
use terrain::ChunkCoords;
use super::MesherSystem;
ecs::impl_systemdata!(MesherSystem);

// Loop though every entity, checking if one of them got their Voxel Data generated
fn entity_update(data: &mut SystemData<MesherSystem>, entity: &ecs::Entity) {
    // Get the chunk component and check for valid Voxel Data
    let chunk = core::global::ecs::component::<terrain::Chunk>(entity).unwrap();
    // If this chunk is not a pending chunk, then we cannot generate it's model
    if data.pending_chunks.contains(&chunk.coords) && chunk.generated {
        if let Option::Some(voxel_data) = &chunk.voxel_data {
            // If the voxel data is valid, create the model for this chunk
            let tmodel = terrain::mesher::generate_model(&voxel_data, chunk.coords, true);
            let model = rendering::Model::combine(tmodel.model, tmodel.skirts_model);
    
            // Create the model on the GPU
            let model_id = rendering::pipec::model(model);
            // Since each chunk starts without a renderer, we must manually add the renderer component
            let mut linkings = ecs::ComponentLinkingGroup::new();
            // Create a renderer with the correct model and materials
            
            let renderer = crate::components::Renderer::default().set_wireframe(true).set_model(model_id).set_material(data.material) ;
            linkings.link::<crate::components::Renderer>(renderer).unwrap();
            data.pending_chunks.remove(&chunk.coords);
            core::global::ecs::link_components(entity.entity_id, linkings);
        } else {
            // We generated the voxel data, but there was no surface, so no need to create the model
        }
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
