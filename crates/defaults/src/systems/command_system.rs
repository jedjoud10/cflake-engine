use systems::{System, SystemData, SystemEventType};
use world_data::WorldData;

// Events
fn system_postfire(_system_data: &mut SystemData, data: &mut WorldData) {
    if data.debug.console.listen_command("toggle-vsync").is_some() {}
    // Crash the world lol
    if data.debug.console.listen_command("kill-me").is_some() {
        let x: Vec<usize> = data.entity_manager.entities.get_valids().iter().map(|x| x.entity_id).collect();
        for entity_id in x {
            data.entity_manager.remove_entity_s(entity_id).unwrap();
        }
    }
    // Toggle the UI
    if data.debug.console.listen_command("toggle-ui").is_some() {
        let r = data.ui_manager.get_default_root_mut();
        r.visible = !r.visible;
    }
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
    let template_command = debug::Command::new("kill-me", Vec::new());
    data.debug.console.register_template_command(template_command);
    let template_command = debug::Command::new("toggle-ui", Vec::new());
    data.debug.console.register_template_command(template_command);
    // Attach the events
    system.event(SystemEventType::SystemPostfire(system_postfire));

    system
}
