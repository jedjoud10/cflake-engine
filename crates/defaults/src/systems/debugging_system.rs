use world::{gui::egui, terrain, World, WorldState};

use crate::globals::{Physics, Terrain};

// Smol gui
fn run(world: &mut World) {
    let gui = &world.gui.egui;
    egui::Window::new("Debug Window").vscroll(false).hscroll(false).resizable(false).show(gui, |ui| {
        // Debug some world values
        // Main
        let _data = world.globals.get::<crate::globals::GlobalWorldData>().unwrap();
        ui.heading("World");
        if ui.button("Quit game").clicked() {
            world.state = WorldState::Exit;
        }
        // Timings
        ui.separator();
        ui.heading("Timings");
        ui.label(format!("Time: {:.1}", world.time.elapsed()));
        ui.label(format!("Delta: {:.3}", world.time.average_delta()));
        ui.label(format!("FPS: {:.1}", 1.0 / world.time.average_delta()));
        // ECS
        ui.separator();
        ui.heading("Entity Component Systems");

        // Terrain
        let terrain = world.globals.get_mut::<Terrain>();
        if let Some(terrain) = terrain {
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
        // Physics
        let physics = world.globals.get_mut::<Physics>();
        if let Some(physics) = physics {
            ui.separator();
            ui.heading("Physics");
        }
    });
}
// Create the debugging system
pub fn system(world: &mut World) {
    world.events.insert(run);
}
