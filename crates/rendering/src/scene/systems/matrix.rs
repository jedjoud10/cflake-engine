use crate::Renderer;
use world::{post_user, System, World};

// Check if an AABB intersects all the given frustum planes
// TODO: Use space partioning algorithms to make this faster (ex. Octree)
// TODO: Use multithreading to make it faster as well
// https://subscription.packtpub.com/book/game+development/9781787123663/9/ch09lvl1sec89/obb-to-plane
// https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
pub fn intersects_frustum(planes: &vek::FrustumPlanes<f32>, aabb: vek::Aabb<f32>, matrix: &vek::Mat4<f32>) -> bool {
    let mut corners = aabb.points();

    for corner in corners.iter_mut() {
        *corner = matrix.mul_point(*corner);
    }

    let aabb = MeshUtils::aabb_from_points(&corners).unwrap();

    let corners = [aabb.min, aabb.max];

    planes.iter().all(|plane| {
        let mut furthest = vek::Vec3::zero();
        furthest.iter_mut().enumerate().for_each(|(i, e)| {
            *e = corners[(plane.normal[i] > 0.0) as usize][i];
        });
        let signed = furthest.dot(plane.normal) + plane.distance;

        signed > 0.0
    })
}

// Update the global mesh matrices of objects that have been modified
// This will also handle frustum culling 
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

    // Handle frustum culling of the scene objects
    let query = scene.query_mut::<(&mut Culler, &Renderer)>()
}

// The matrix system will be responsible for updating the matrices of the renderer
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(super::rendering::system)
        .after(post_user);
}
