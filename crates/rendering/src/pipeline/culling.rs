use ecs::Scene;
use math::SharpVertices;
use utils::{Storage, ThreadPool};
use world::World;

use crate::{Material, DefaultMaterialResources, Mesh, Surface, Renderer};

// Check if an AABB intersects all the given frustum planes
// TODO: Use space partioning algorithms to make this faster (ex. Octree)
// TODO: Optimize this shit
// https://subscription.packtpub.com/book/game+development/9781787123663/9/ch09lvl1sec89/obb-to-plane
// https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
pub fn intersects_frustum(planes: &math::Frustum<f32>, aabb: math::Aabb<f32>, matrix: &vek::Mat4<f32>) -> bool {
    let mut corners = <math::Aabb<f32> as SharpVertices<f32>>::points(&aabb);

    for corner in corners.iter_mut() {
        *corner = matrix.mul_point(*corner);
    }

    let aabb = crate::aabb_from_points(&corners).unwrap();

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

// Update the "culled" paramter of each surface
pub(super) fn cull_surfaces<'r, M: Material>(
    world: &'r World,
    meshes: &'r Storage<Mesh>,
    default: &mut DefaultMaterialResources<'r>,
) {
    // Don't cull if there's no need
    if !M::frustum_culling() {
        return;
    }

    // Get all the entities that contain a visible surface
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();
    let query = scene.query_mut::<(&mut Surface<M>, &Renderer)>();

    // Iterate over the surfaces of this material and update their culled state
    query.for_each(
        &mut threadpool,
        |(surface, renderer)| {
        let mesh = meshes.get(&surface.mesh);
        if let Some(aabb) = mesh.vertices().aabb() {
            surface.culled = !intersects_frustum(&default.camera_frustum, aabb, &renderer.matrix)
        } else {
            surface.culled = false;
        }
    }, 256);
}
