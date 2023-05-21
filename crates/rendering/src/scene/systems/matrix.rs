use crate::Renderer;
use coords::{Position, Rotation, Scale};
use rayon::prelude::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use world::{post_user, System, World};

// Update the global mesh matrices of objects that have been modified
// This will also handle frustum culling
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    use ecs::*;

    // Filter the objects that have changed only
    let f1 = modified::<&Position>();
    let f2 = modified::<&Rotation>();
    let f3 = modified::<&Scale>();
    let f4 = added::<&Renderer>();
    let filter = f1 | f2 | f3 | f4;
    let query = scene.query_mut_with::<(
        &mut Renderer,
        Option<&Position>,
        Option<&Rotation>,
        Option<&Scale>,
    )>(filter);
    let iter = query.into_iter().collect::<Vec<_>>();
    let iter = iter.into_par_iter();

    // Update the matrices of objects that might contain location, rotation, or scale
    iter.for_each(|(renderer, position, rotation, scale)| {
        let mut matrix = vek::Mat4::<f32>::identity();
        if let Some(position) = position {
            matrix *= vek::Mat4::from(*position);
        }

        if let Some(rotation) = rotation {
            matrix *= vek::Mat4::from(rotation);
        }

        if let Some(scale) = scale {
            matrix *= vek::Mat4::from(scale);
        }
        renderer.matrix = matrix;
    });
}

// The matrix system will be responsible for updating the matrices of the renderer
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(super::rendering::system)
        .after(post_user);
}
