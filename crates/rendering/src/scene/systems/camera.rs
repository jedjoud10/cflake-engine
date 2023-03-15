use crate::{Camera, CameraUniform, ForwardRenderer};

use ecs::{Position, Rotation, Scene};
use graphics::Window;

use world::{System, World};

// Update event that will set/update the main perspective camera
fn update(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut renderer = world.get_mut::<ForwardRenderer>().unwrap();
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
        let (camera, location, rotation) = entry
            .as_query_mut::<(&mut Camera, &Position, &Rotation)>()
            .unwrap();
        let aspect = window.size().w as f32 / window.size().h as f32;
        camera.set_aspect_ratio(aspect);
        camera.update(location, rotation);

        // Convert the camera to uniform data
        let projection = camera.projection_matrix().cols;
        let view = camera.view_matrix().cols;
        let inverse_projection =
            (camera.projection_matrix().inverted()).cols;
        let inverse_view = (camera.view_matrix().inverted()).cols;

        // Create the struct that contains the UBO data
        let data = CameraUniform {
            projection,
            inverse_projection,
            view,
            inverse_view,
            position: (*location).with_w(0.0),
            forward: rotation.forward().with_w(0.0),
            right: rotation.right().with_w(0.0),
            up: rotation.up().with_w(0.0),
        };

        // Fill the camera UBO with the proper data
        renderer.camera_buffer.write(&[data], 0).unwrap();
    } else {
        // Set the main camera if we find one
        let next = ecs
            .find::<(&Camera, &Position, &Rotation, &ecs::Entity)>();
        if let Some((_, _, _, entity)) = next {
            renderer.main_camera = Some(*entity);
        }
    }
}

// The camera system will be responsible for updating the camera UBO and matrices
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(super::rendering::system);
}
