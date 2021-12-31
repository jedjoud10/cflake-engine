use core::global::callbacks::CallbackType;

use ecs::SystemData;
use others::callbacks::{NullCallback, OwnedCallback};

// Some default events
fn system_prefire(data: &mut SystemData<()>) {
}
// Create the default system
pub fn system() {
    core::global::ecs::add_system((), || {
        // Create a system
        let mut system = ecs::System::new();
        // Link some components to the system
        // And link the events
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        // Return the newly made system
        system
    });
}
