use ecs::SystemData;
use math::octrees::OctreeNode;
use terrain::ChunkCoords;
use super::MesherSystem;
ecs::impl_systemdata!(MesherSystem);

fn system_prefire(data: &mut SystemData<(MesherSystem)>) {

}

// Create the Mesher System
pub fn system(material: rendering::GPUObjectID) {    
    let mesher_system_data = MesherSystem {
        material,
        pending_chunks: Vec::new(),
    };
    core::global::ecs::add_system(mesher_system_data, || {
        // Create a system
        let mut system = ecs::System::new();
        // And link the events
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        // Return the newly made system
        system
    });
}
