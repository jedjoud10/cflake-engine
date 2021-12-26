use ecs::SystemData;
use rendering::GPUObjectID;
use terrain::{ChunkCoords, VoxelData};

use super::VoxelGenerationSystem;
ecs::impl_systemdata!(VoxelGenerationSystem);

// Get the first pending chunk, and tell the voxel generator to generate it's voxel data if it is allowed to
pub fn system_prefire(data: &mut SystemData<VoxelGenerationSystem>) {
    if data.is_free() {
        // We can run the voxel generation logic
        let chunk_coords = data.remove_first_chunk();
        data.generate_voxel_data(chunk_coords);
    }
}

// Check if the current Chunk has gotten it's voxel data generated
pub fn entity_update(data: &mut SystemData<VoxelGenerationSystem>, entity: &ecs::Entity) {
}

// When a chunk gets added, we tell the voxel generator to buffer the voxel generation for that chunk
pub fn entity_added(data: &mut SystemData<VoxelGenerationSystem>, entity: &ecs::Entity) {
    let chunk = core::global::ecs::component::<terrain::Chunk>(entity).unwrap();
    data.add_pending_chunk(chunk.coords.clone());
}

// When a chunk gets removed, we tell the voxel generator to stop generating the chunk's voxel data, if it is
pub fn entity_removed(data: &mut SystemData<VoxelGenerationSystem>, entity: &ecs::Entity) {
}

// Create the default system
pub fn system(interpreter_string: String) {
    // Create the system data
    core::global::ecs::add_system(VoxelGenerationSystem::new(interpreter_string) , || {
        // Create a system
        let mut system = ecs::System::new();
        // Link some components to the system
        system.link::<crate::components::Transform>();
        system.link::<terrain::Chunk>();
        // And link the events
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        system.event(ecs::SystemEventType::EntityAdded(entity_added));
        system.event(ecs::SystemEventType::EntityRemoved(entity_removed));
        // Return the newly made system
        system
    });
}
