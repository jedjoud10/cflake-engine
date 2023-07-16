use crate::{DefaultMaterialResources, Material, Pass, RenderPath, Renderer, SubSurface, Surface};
use ecs::Scene;
use math::ExplicitVertices;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use world::World;

// Check if an AABB intersects all the given frustum planes
// TODO: Use space partioning algorithms to make this faster (ex. Octree)
// TODO: Optimize this shit
// https://subscription.packtpub.com/book/game+development/9781787123663/9/ch09lvl1sec89/obb-to-plane
// https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
pub(crate) fn intersects_frustum(
    planes: &math::Frustum<f32>,
    aabb: math::Aabb<f32>,
    matrix: &vek::Mat4<f32>,
) -> bool {
    let corners = <math::Aabb<f32> as ExplicitVertices<f32>>::points(&aabb);
    let mut out: [vek::Vec4<f32>; 8] = [vek::Vec4::zero(); 8];

    for (input, output) in corners.iter().zip(out.iter_mut()) {
        let vec = matrix.mul_point(*input);
        *output = vec.with_w(0.0);
    }

    if let Some(aabb) = crate::aabb_from_points(&out) {
        let corners = [aabb.min, aabb.max];

        planes.iter().all(|plane| {
            let mut furthest = vek::Vec3::zero();
            furthest.iter_mut().enumerate().for_each(|(i, e)| {
                *e = corners[(plane.normal[i] > 0.0) as usize][i];
            });
            let signed = furthest.dot(plane.normal) + plane.distance;

            signed > 0.0
        })
    } else {
        log::error!("Cannot create AABB for culling!");
        false
    }
}

// Check if an AABB intersects the shadow lightspace matrix
// TODO: Reimplement shadow culling
pub(crate) fn intersects_lightspace(
    lightspace: &vek::Mat4<f32>,
    aabb: math::Aabb<f32>,
    matrix: &vek::Mat4<f32>,
) -> bool {
    let corners = <math::Aabb<f32> as ExplicitVertices<f32>>::points(&aabb);

    for input in corners.iter() {
        let vec = matrix.mul_point(*input);
        let uv = lightspace.mul_point(vec);

        if uv.x.abs() < 1.0 && uv.y.abs() < 1.0 {
            return true;
        }
    }

    false
}

// Update the "culled" paramter of each surface
pub(super) fn cull_surfaces<'r, P: Pass, M: Material>(
    world: &'r World,
    defaults: &DefaultMaterialResources<'r>,
) {
    if !M::cull::<P>() {
        return;
    }

    // Get all the entities that contain a visible surface
    let mut scene = world.get_mut::<Scene>().unwrap();
    let query = scene.query_mut::<(&mut Surface<M>, &Renderer)>();
    //let iter = query.into_iter().collect::<Vec<_>>();
    //let iter = iter.into_par_iter();

    // Iterate over the surfaces of this material and update their culled state
    for (surface, renderer) in query {
        if !renderer.visible {
            return;
        }

        // A surface is culled *only* if all of it's sub-surface are not visible
        P::set_cull_state(
            surface,
            surface.subsurfaces.iter().all(|SubSurface { mesh, .. }| {
                // Get the mesh and it's AABB
                let mesh = <M::RenderPath as RenderPath>::get(defaults, mesh);
                let aabb = mesh.vertices().aabb();

                // If we have a valid AABB, check if the surface is visible within the frustum
                if let Some(aabb) = aabb {
                    P::cull(&defaults, aabb, &renderer.matrix)
                } else {
                    false
                }
            }),
        );
    }
    //iter.for_each(|(surface, renderer)| {

    //});
}
