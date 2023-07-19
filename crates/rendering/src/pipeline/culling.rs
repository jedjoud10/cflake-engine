use crate::{DefaultMaterialResources, Material, Pass, RenderPath, Renderer, SubSurface, Surface};
use ecs::Scene;
use math::ExplicitVertices;
use rayon::prelude::ParallelIterator;
use world::World;

// Results of culling against frustum/lightspace
#[derive(Debug, Clone, Copy)]
pub enum CullResult {
    // Object is one the bounding area
    Intersect,

    // Object is completely culled / outside the bounds
    Culled,

    // Object is completely visible and within the bounds
    Visible,
}

impl CullResult {
    // If the object must be completely culled
    pub fn culled(&self) -> bool {
        match self {
            CullResult::Culled => true,
            _ => false,
        }
    }

    // If the object is visible
    pub fn visible(&self) -> bool {
        !self.culled()
    }
}

// Cull an AABB with a specific matrix against a frustum
// TODO: Use space partioning algorithms to make this faster (ex. Octree)
// TODO: Optimize this shit
// https://subscription.packtpub.com/book/game+development/9781787123663/9/ch09lvl1sec89/obb-to-plane
// https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
pub(crate) fn cull_against_frustum(
    planes: &math::Frustum<f32>,
    aabb: math::Aabb<f32>,
    matrix: &vek::Mat4<f32>,
) -> CullResult {
    let corners = <math::Aabb<f32> as ExplicitVertices<f32>>::points(&aabb);
    let mut out: [vek::Vec4<f32>; 8] = [vek::Vec4::zero(); 8];

    for (input, output) in corners.iter().zip(out.iter_mut()) {
        let vec = matrix.mul_point(*input);
        *output = vec.with_w(0.0);
    }

    if let Some(aabb) = crate::aabb_from_points(&out) {
        let corners = [aabb.min, aabb.max];

        let res = planes.planes().map(|plane| {
            let mut furthest = vek::Vec3::zero();
            furthest.iter_mut().enumerate().for_each(|(i, e)| {
                *e = corners[(plane.normal[i] > 0.0) as usize][i];
            });
            let signed = furthest.dot(plane.normal) + plane.distance;

            signed > 0.0
        });

        let all = res.iter().all(|x| *x);
        let any = res.iter().any(|x| *x);

        match (all, any) {
            (true, true) | (true, false) => CullResult::Culled,
            (false, true) => CullResult::Intersect,
            (false, false) => CullResult::Visible,
        }
    } else {
        log::error!("Could not create an AABB for culling!");
        CullResult::Culled
    }
}

// Cull an AABB with a specific matrix against a lightspace matrix
pub(crate) fn cull_against_lightspace_matrix(
    lightspace: &vek::Mat4<f32>,
    aabb: math::Aabb<f32>,
    matrix: &vek::Mat4<f32>,
) -> CullResult {
    let corners = <math::Aabb<f32> as ExplicitVertices<f32>>::points(&aabb);

    let bools = corners.map(|vec| {
        let vec = matrix.mul_point(vec);
        let uv = lightspace.mul_point(vec);
        uv.x.abs() < 1.0 && uv.y.abs() < 1.0
    });

    // If all the nodes are outside the lightspace frustum
    let all_outside = bools.iter().all(|x| !*x);

    // If all the nodes are inside the lightspace frustum
    let all_inside = bools.iter().all(|x| *x);

    match (all_outside, all_inside) {
        (true, false) => CullResult::Culled,
        (false, true) => CullResult::Visible,
        _ => CullResult::Intersect,
    }
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

    // Iterate over the surfaces of this material and update their culled state
    for (surface, renderer) in query {
        if !renderer.visible || surface.subsurfaces.is_empty() {
            return;
        }

        // A surface is culled *only* if all of it's sub-surface are not visible
        P::set_cull_state(
            defaults,
            surface,
            surface.subsurfaces.iter().map(|SubSurface { mesh, .. }| {
                // Get the mesh and it's AABB
                let mesh = <M::RenderPath as RenderPath>::get(defaults, mesh);
                let aabb = mesh.vertices().aabb();

                // If we have a valid AABB, check if the surface is visible within the frustum
                if let Some(aabb) = aabb {
                    P::cull(defaults, aabb, &renderer.matrix)
                } else {
                    log::error!("No valid AABB, not culling");
                    CullResult::Culled
                }
            }).reduce(|a, b| match (a, b) {
                (CullResult::Culled, CullResult::Culled) => CullResult::Culled,
                (CullResult::Visible, CullResult::Visible) => CullResult::Visible,
                _ => CullResult::Intersect,
            }).unwrap(),
        );
    }
}
