use cflake_engine::{
    assets,
    defaults,
    defaults::components::{Camera, Light, Transform, Renderer},
    rendering::basics::lights::LightType,
    vek, World,
};
// An example with multiple meshes in the same scene
fn main() {
    cflake_engine::start("cflake-examples/batch", init)
}
// Init the simple camera and multiple meshes
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");

    defaults::systems::flycam_system::system(world);

    world.events.insert(|world| {
        cflake_engine::gui::egui::Window::new("Debug Window").vscroll(false).hscroll(false).resizable(false).show(&mut world.gui.egui, |ui| {
            // Timings
            ui.separator();
            ui.heading("Timings");
            ui.label(format!("Time: {:.1}", world.time.elapsed()));
            ui.label(format!("Delta: {:.3}", world.time.average_delta()));
            ui.label(format!("FPS: {:.1}", 1.0 / world.time.average_delta()));
        });
    });

    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 2.0, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::new_directional(1.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-90f32.to_radians())).unwrap();
    });

    // Create multiple cubes
    let cube = world.pipeline.defaults().cube.clone();
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                world.ecs.insert(|_, linker| {
                    linker.insert(Renderer::from(cube.clone())).unwrap();
                    linker.insert(Transform::from((x as f32 * 2.0, y as f32 * 2.0, z as f32 * 2.0))).unwrap();
                });
            }
        }
    }    
}