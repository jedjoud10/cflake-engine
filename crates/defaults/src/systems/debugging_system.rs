use world::{gui::egui, terrain, World, WorldState};

use crate::resources::{Physics, Terrain};

// Smol gui
fn run(world: &mut World) {
    let gui = &world.gui.egui;
    egui::Window::new("Debug Window").vscroll(false).hscroll(false).resizable(false).show(gui, |ui| {
        // Debug some world values
        // Main
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
        ui.label(format!("Entity count: {}", world.ecs.entities().len()));
        ui.label(format!("Archetype count: {}", world.ecs.archetypes().len()));

        // Rendering
        ui.separator();
        ui.heading("Rendering");
        let stats = world.pipeline.stats().borrow();
        ui.label(format!("Models drawn: {}", stats.drawn));
        ui.label(format!("Models culled: {}", stats.culled));        
        ui.label(format!("Shadow-Models drawn: {}", stats.shadowed));

        // Terrain
        let terrain = world.resources.get_mut::<Terrain>();
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
    });
}
// Create the debugging system
pub fn system(world: &mut World) {
    world.events.insert(run);
}
