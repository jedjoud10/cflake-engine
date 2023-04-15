use std::time::Instant;

use ecs::Scene;
use math::ExplicitVertices;
use utils::{ThreadPool};
use world::World;

use crate::{
    DefaultMaterialResources, Material, RenderPath, Renderer,
    Surface, SubSurface,
};

// Check if an AABB intersects all the given frustum planes
// TODO: Use space partioning algorithms to make this faster (ex. Octree)
// TODO: Optimize this shit
// https://subscription.packtpub.com/book/game+development/9781787123663/9/ch09lvl1sec89/obb-to-plane
// https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
pub fn intersects_frustum(
    planes: &math::Frustum<f32>,
    aabb: math::Aabb<f32>,
    matrix: &vek::Mat4<f32>,
) -> bool {
    let corners =
        <math::Aabb<f32> as ExplicitVertices<f32>>::points(&aabb);
    let mut out: [vek::Vec4<f32>; 8] = [vek::Vec4::zero(); 8];

    for (input, output) in corners.iter().zip(out.iter_mut()) {
        let vec = matrix.mul_point(*input);
        *output = vec.with_w(0.0);
    }

    let aabb = crate::aabb_from_points(&out).unwrap();

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
    defaults: &mut DefaultMaterialResources<'r>,
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
            // A surface is culled *only* if all of it's sub-surface are not visible
            surface.culled = surface.subsurfaces.iter().all(|SubSurface { mesh, material }| {
                // Get the mesh and it's AABB
                let mesh = <M::RenderPath as RenderPath>::get(
                    defaults,
                    &mesh,
                );
                let aabb = mesh.vertices().aabb();

                // If we have a valid AABB, check if the surface is visible within the frustum
                if let Some(aabb) = aabb {
                    !intersects_frustum(
                        &defaults.camera_frustum,
                        aabb,
                        &renderer.matrix,
                    )
                } else {
                    false
                }
            });
        },
        1024,
    );
}
