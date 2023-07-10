use crate::{Camera, CameraUniform, DeferredRenderer};

use coords::{Position, Rotation};
use ecs::Scene;
use graphics::Window;

use world::{System, World, post_user};

// Update event that will set/update the main perspective camera
fn update(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut renderer = world.get_mut::<DeferredRenderer>().unwrap();
    let window = world.get::<Window>().unwrap();

    // Fetch the main perspective camera from the scene renderer
    if let Some(entity) = renderer.main_camera {
        // Disable the entity in the resource if it got removed
        let mut entry = if let Some(entry) = ecs.entry_mut(entity) {
            entry
        } else {
            renderer.main_camera = None;
            return;
        };

        // Fetch it's components,and update them
        let (camera, position, rotation) = entry
            .as_query_mut::<(&mut Camera, &Position, &Rotation)>()
            .unwrap();
        let aspect = window.size().w as f32 / window.size().h as f32;
        camera.aspect_ratio = aspect;
        let projection = camera.projection_matrix();
        let view = camera.view_matrix(position, rotation);

        // Convert the camera to uniform data
        let projection = projection.cols;
        let view = view.cols;

        // Create the struct that contains the UBO data
        let data = CameraUniform {
            projection,
            view,
            position: (*position).with_w(0.0),
            forward: rotation.forward().with_w(0.0),
            right: rotation.right().with_w(0.0),
            up: rotation.up().with_w(0.0),
            near_far_hfov_: vek::Vec4::new(camera.near, camera.far, camera.hfov, 0.0),
            _a: vek::Vec4::zero(),
            _b: vek::Vec4::zero(),
            _c: vek::Vec4::zero(),
        };

        // Fill the camera UBO with the proper data
        renderer.camera_buffer.write(&[data], 0).unwrap();
    } else {
        // Set the main camera if we find one
        let next = ecs.find::<(&Camera, &Position, &Rotation, &ecs::Entity)>();
        if let Some((_, _, _, entity)) = next {
            renderer.main_camera = Some(*entity);
        }
    }
}

// The camera system will be responsible for updating the camera UBO and matrices
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(super::rendering::system)
        .after(post_user);
}
