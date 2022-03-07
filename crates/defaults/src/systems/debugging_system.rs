use world::{ecs::event::EventKey, gui::egui, World, WorldState};

// The debugging system's update loop
fn run(world: &mut World, _data: EventKey) {
    // Check if we need to debug
    let gui = &world.gui.egui;
    let state = &mut world.state;
    egui::Window::new("Debug Window").vscroll(false).hscroll(false).resizable(false).show(gui, |ui| {
        // Debug some world values
        // Main
        let _data = world.globals.get::<crate::globals::GlobalWorldData>().unwrap();
        ui.heading("World");
        if ui.button("Quit game").clicked() {
            *state = WorldState::Exit;
        }
        //ui.label(format!("Camera Pos: '{}'", (data.camera_pos * 10.0).round() / 10.0));
        //ui.label(format!("Camera Dir: '{}'", (data.camera_forward * 10.0).round() / 10.0));
        // Timings
        ui.separator();
        ui.heading("Timings");
        ui.label(format!("Time: {:.1}", world.time.elapsed));
        ui.label(format!("Delta: {:.3}", world.time.delta));
        ui.label(format!("FPS: {:.1}", 1.0 / world.time.delta));
        // ECS
        ui.separator();
        ui.heading("Entity Component Systems");
        ui.label(format!("Component: '{}'", world.ecs.components.len()));
        ui.label(format!("Entities: '{}'", world.ecs.entities.inner().len()));
        ui.label(format!("Systems: '{}'", world.ecs.systems.inner().borrow().len()));
        /*
         */
        /*
        // Terrain
        let terrain = world.globals.get_mut::<crate::globals::Terrain>();
        if let Ok(terrain) = terrain {
            let octree = terrain.chunks_manager.octree.lock();
            ui.separator();
            ui.heading("Terrain");
            ui.label(format!("Chunk Size: [{a}x{a}x{a}]", a = terrain::CHUNK_SIZE));
            ui.label(format!("Terrain Octree Depth: '{}'", octree.inner.depth));
            ui.label(format!("Terrain Octree Size: '[{a}x{a}x{a}]'", a = octree.inner.get_root_node().half_extent * 2));
            ui.label(format!("Chunks: '{}'", terrain.chunks_manager.chunks.len()));
            ui.label(format!("Pending Generation: '{}'", terrain.chunks_manager.chunks_generating.len()));
            ui.label(format!("Pending Deletion: '{}'", terrain.chunks_manager.chunks_to_remove.len()));
            ui.label(format!("Total Edits: '{}'", terrain.editing_manager.edits.len()));
        }
        */
        // Rendering
        ui.separator();
        ui.heading("Rendering");
    });
}
// Create the debugging system
pub fn system(world: &mut World) {
    world.ecs.systems.builder().with_run_event(run).build();
}
