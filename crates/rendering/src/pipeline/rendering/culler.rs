use arrayvec::ArrayVec;
use math::bounds::aabb::AABB;

use super::{RenderedModel, RenderingCamera};

// A single frustum plane
#[derive(Debug)]
struct Plane {
    normal: vek::Vec3<f32>,
    distance: f32,
}

impl Plane {
    // Create a plane from a matrix column
    fn new(column: vek::Vec4<f32>) -> Self {
        let mag = column.xyz().magnitude();
        Self {
            normal: column.xyz() / mag,
            distance: (column.w / mag),
        }
    }
}

// The 6 frustum planes from the camera
struct Frustum {
    planes: [Plane; 6]
}

// Calculate the view frustum from the camera
fn frustum(camera: &RenderingCamera) -> Frustum {
    let columns = camera.projm_viewm.clone().transposed().into_col_arrays();
    let columns = columns.into_iter().map(vek::Vec4::from).collect::<ArrayVec<vek::Vec4<f32>, 4>>();

    // Magic from https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
    // And also from https://gamedev.stackexchange.com/questions/156743/finding-the-normals-of-the-planes-of-a-view-frustum
    // YAY https://stackoverflow.com/questions/12836967/extracting-view-frustum-planes-gribb-hartmann-method
    let left = Plane::new(columns[3] + columns[0]);
    let right = Plane::new(columns[3] - columns[0]);
    let top = Plane::new(columns[3] - columns[1]);
    let bottom = Plane::new(columns[3] + columns[1]);
    let near = Plane::new(columns[3] + columns[2]);
    let far = Plane::new(columns[3] - columns[2]);

    Frustum {
        planes: [top, bottom, left, right, near, far]
    }
} 

// AABB - Frustum collision check
fn is_inside_frustum_aabb(frustum: &Frustum, aabb: &AABB) -> bool {
    // Totally not stolen from the internet or anything
    for plane in frustum.planes.iter() {
        // Furthest vertex down the plane normal
        let mut furthest = vek::Vec3::zero();

        // Update each value in each axis
        furthest.iter_mut().enumerate().for_each(|(i, e)| {            
            *e = aabb[(plane.normal[i] > 0.0) as usize][i];
        });

        // Calculate the signed distance of the point
        let signed = furthest.dot(plane.normal) + plane.distance;

        // Early return
        if signed < 0.0 {
            return false;
        }
    }
    true
}
// AABB frustum culling
// This will remove the objects that must be culled from "vec"
pub fn cull_frustum<'b>(camera: &RenderingCamera, mut vec: Vec<RenderedModel<'b>>) -> Vec<RenderedModel<'b>> {
    // Calculate the view frustum
    let frustum = frustum(camera);

    // Check if each object is inside the frustum or not
    let old = vec.len();
    let i = std::time::Instant::now();
    vec.retain(|model| is_inside_frustum_aabb(&frustum, model.aabb));
    let new = vec.len();
    println!("Culled '{}' models in '{:?}'", old-new, i.elapsed());
    vec
}