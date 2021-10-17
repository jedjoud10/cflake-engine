use systems::{System, SystemData, SystemEventType};
use world_data::WorldData;

// Events
fn system_postfire(_system_data: &mut SystemData, data: &mut WorldData) {
    if data.debug.console.listen_command("toggle-vsync").is_some() {}
}

pub fn system(data: &mut WorldData) -> System {
    let mut system = System::new();
    // Create some default template commands
    let fullscreen_command = debug::Command::new("toggle-fullscreen", Vec::new());
    data.debug.console.register_template_command(fullscreen_command);
    let quit_command = debug::Command::new("quit", Vec::new());
    data.debug.console.register_template_command(quit_command);
    let template_command = debug::Command::new("toggle-vsync", Vec::new());
    data.debug.console.register_template_command(template_command);
    // Attach the events
    system.event(SystemEventType::SystemPostfire(system_postfire));

    system
}
