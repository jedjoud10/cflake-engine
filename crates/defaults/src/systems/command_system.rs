use system_event_data::WorldData;
use systems::{System, SystemEventType};

// System added events
fn system_added(data: &mut SystemData, world_data: &mut WorldData) {
    println!("Command system added!");
}

fn create_command_system() -> System {
    let mut system = System::new("Default Commands System");
    // Link the components
    system.lin
    // Attach the events
    system.event(SystemEventType::SystemAdded(system_added));

    return system;
}