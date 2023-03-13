use crate::Renderer;

use world::{post_user, System, World};

// Update the global mesh matrices of objects that have been modified
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    use ecs::*;

    // Filter the objects that have changed only
    let f1 = modified::<Position>();
    let f2 = modified::<Rotation>();
    let f3 = modified::<Scale>();
    let f4 = added::<Renderer>();
    let filter = f1 | f2 | f3 | f4;
    let query = scene.query_mut_with::<(
        &mut Renderer,
        Option<&ecs::Position>,
        Option<&ecs::Rotation>,
        Option<&ecs::Scale>,
    )>(filter);

    // Update the matrices of objects that might contain location, rotation, or scale
    for (renderer, location, rotation, scale) in query {
        let mut matrix = vek::Mat4::<f32>::identity();
        matrix =
            location.map_or(matrix, |l| matrix * vek::Mat4::from(l));
        matrix *=
            rotation.map_or(matrix, |r| matrix * vek::Mat4::from(r));
        matrix *=
            scale.map_or(matrix, |s| matrix * vek::Mat4::from(s));
        renderer.matrix = matrix;
    }
}

// The matrix system will be responsible for updating the matrices of the renderer
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(super::rendering::system)
        .after(post_user);
}
