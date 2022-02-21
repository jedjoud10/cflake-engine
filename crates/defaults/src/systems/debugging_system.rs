use main::{
    core::{World, WorldState},
    ecs::event::EventKey,
    gui::egui,
    math::{
        csg::CSGOperation,
        shapes::{BasicShapeType, Cube},
    },
    terrain::editing::Edit,
};

// The debugging system's update loop
fn run(world: &mut World, _data: EventKey) {
    // Check if we need to debug
    let gui = &world.gui.egui;
    let state = &mut world.state;
    let pipeline = world.pipeline.read();
    egui::Window::new("Debug Window").vscroll(false).hscroll(false).resizable(false).show(gui, |ui| {
        // Debug some world values
        // Main
        let data = world.globals.get_global::<crate::globals::GlobalWorldData>().unwrap();
        ui.heading("World");
        if ui.button("Quit game").clicked() {
            *state = WorldState::Exit;
        }
        ui.label(format!("Camera Pos: '{}'", (data.camera_pos * 10.0).round() / 10.0));
        ui.label(format!("Camera Dir: '{}'", (data.camera_forward * 10.0).round() / 10.0));
        // Timings
        ui.separator();
        ui.heading("Timings");
        ui.label(format!("Time: {:.1}", world.time.elapsed));
        ui.label(format!("Delta: {:.3}", world.time.smoothed_delta));
        ui.label(format!("FPS: {:.1}", 1.0 / world.time.smoothed_delta));
        // ECS
        ui.separator();
        ui.heading("Entity Component Systems");
        ui.label(format!("Component: '{}'", world.ecs.count_components()));
        ui.label(format!("Entities: '{}'", world.ecs.count_entities()));
        ui.label(format!("Systems: '{}'", world.ecs.count_systems()));
        // Terrain
        let terrain = world.globals.get_global_mut::<crate::globals::Terrain>();
        if let Ok(mut terrain) = terrain {
            ui.separator();
            ui.heading("Terrain");
            ui.label(format!("Chunk Size: [{a}x{a}x{a}]", a = main::terrain::CHUNK_SIZE));
            ui.label(format!("Terrain Octree Depth: '{}'", terrain.chunks_manager.octree.inner.depth));
            ui.label(format!(
                "Terrain Octree Size: '[{a}x{a}x{a}]'",
                a = terrain.chunks_manager.octree.inner.get_root_node().half_extent * 2
            ));
            ui.label(format!("Chunks: '{}'", terrain.chunks_manager.chunks.len()));
            ui.label(format!("Pending Generation: '{}'", terrain.chunks_manager.chunks_generating.len()));
            ui.label(format!("Pending Deletion: '{}'", terrain.chunks_manager.chunks_to_remove.len()));
            ui.label(format!("Total Edits: '{}'", terrain.editing_manager.edits.len()));
            if ui.button("Edit terrain").clicked() {
                terrain.edit(Edit::new(
                    BasicShapeType::Cube(Cube {
                        center: veclib::Vector3::<f32>::ZERO,
                        size: veclib::vec3(20.0, 20.0, 20.0),
                    }),
                    CSGOperation::Union,
                ));
            }
        }
        // Rendering
        ui.separator();
        ui.heading("Rendering");
        let debuginfo = pipeline.debugging.lock();
        ui.label(format!("Draw Calls: '{}'", debuginfo.draw_calls));
        ui.label(format!("Shadow Draw Calls: '{}'", debuginfo.shadow_draw_calls));
        ui.label(format!("Triangles: '{}k'", debuginfo.triangles / 1000));
        ui.label(format!("Vertices: '{}k'", debuginfo.vertices / 1000));
        ui.label(format!("Whole Frame Time: '{:.1}'", debuginfo.whole_frame));
        ui.label(format!("Render Frame Time: '{:.1}'", debuginfo.render_frame));
        ui.label(format!("EoF Callbacks Execution Time: '{:.1}'", debuginfo.eof_callbacks_execution));
        ui.label(format!("Swap Buffers Time: '{:.1}'", debuginfo.swap_buffers));
    });
}
// Create the debugging system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).build();
}
