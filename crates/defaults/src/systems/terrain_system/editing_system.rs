use main::core::World;
use main::ecs::event::EventKey;
use main::gui::egui;
use main::math::shapes::{BasicShapeType, Cube, Sphere};
use main::terrain::editing::Edit;
// A system that will handle terrain edits
fn run(world: &mut World, _data: EventKey) {
    // Get the terrain global
    let data = world.globals.get_global::<crate::globals::GlobalWorldData>().unwrap();
    let pos = data.camera_pos + data.camera_forward * 400.0;
    if let Ok(mut terrain) = world.globals.get_global_mut::<crate::globals::Terrain>() {
        // Debug
        let gui = &world.gui.egui;
        egui::Window::new("Debug Terrain Editing Window")
            .vscroll(false)
            .hscroll(false)
            .resizable(false)
            .show(gui, |ui| {
                let mut arr = [terrain.color.x, terrain.color.y, terrain.color.z];
                ui.color_edit_button_rgb(&mut arr);
                terrain.color = veclib::vec3(arr[0], arr[1], arr[2]);
                ui.add(egui::DragValue::new(&mut terrain.size.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut terrain.size.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut terrain.size.z).speed(0.1));
                if ui.button("Edit").clicked() {
                    let size = terrain.size;
                    let color = terrain.color;
                    terrain.edit(
                        Edit::new(BasicShapeType::Cube(Cube { center: pos, size }), main::math::csg::CSGOperation::Union).with_color(veclib::Vector3::<u8>::from(color) * 255),
                    )
                }
            });
        // Editing manager
        let terrain = &mut *terrain;
        let chunks_to_regenerate = terrain.editing_manager.get_influenced_chunks(&terrain.chunks_manager.octree.inner);
        if !chunks_to_regenerate.is_empty() {
            // Regenerate the specified chunks
            for coords in chunks_to_regenerate {
                terrain.regenerate_chunk(coords);
            }
            // Also set the packed edits since we will need to update them on the GPU
            let packed = terrain.editing_manager.convert();
            terrain.voxel_generator.packed_edits_num = packed.len();
            terrain.voxel_generator.packed_edits_update = Some(packed);
        } else {
            terrain.voxel_generator.packed_edits_update = None;
        }
    }
}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).build();
}
