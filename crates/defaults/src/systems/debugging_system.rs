use world::{ecs::component::ComponentQuerySet, gui::egui, terrain, World, WorldState};

// The debugging system's update loop
fn run(world: &mut World, _data: ComponentQuerySet) {
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
        // Timings
        ui.separator();
        ui.heading("Timings");
        ui.label(format!("Time: {:.1}", world.time.elapsed));
        ui.label(format!("Delta: {:.3}", world.time.average_delta));
        ui.label(format!("FPS: {:.1}", 1.0 / world.time.average_delta));
        // ECS
        ui.separator();
        ui.heading("Entity Component Systems");
        ui.label(format!("Component: '{}'", world.ecs.components.len()));
        ui.label(format!("Entities: '{}'", world.ecs.entities.inner().len()));
        ui.label(format!("Systems: '{}'", world.ecs.systems.inner().borrow().len()));
        // Terrain
        let terrain = world.globals.get_mut::<crate::globals::Terrain>();
        if let Ok(terrain) = terrain {
            let octree = &terrain.manager.octree;
            ui.separator();
            ui.heading("Terrain");
            ui.label(format!("Chunk Size: [{a}x{a}x{a}]", a = terrain::CHUNK_SIZE));
            ui.label(format!("Terrain Octree Depth: '{}'", octree.inner.depth()));
            ui.label(format!("Terrain Octree Size: '[{a}x{a}x{a}]'", a = octree.inner.get_root_node().half_extent() * 2));
            ui.label(format!("Chunks: '{}'", terrain.manager.chunks.len()));
            ui.label(format!("Pending Generation: '{}'", terrain.manager.chunks_generating.len()));
            ui.label(format!("Voxel Data Buffer Length: '{}'", terrain.generator.buffer.len()));
            ui.label(format!("Active Mesh Tasks Count: '{}'", terrain.scheduler.active_mesh_tasks_count()));
            ui.label(format!("Pending Deletion: '{}'", terrain.manager.chunks_to_remove.len()));
        }
    });
}
// Create the debugging system
pub fn system(world: &mut World) {
    world.ecs.systems.builder().event(run).build();
}
