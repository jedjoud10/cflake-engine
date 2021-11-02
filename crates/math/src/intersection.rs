use veclib::Swizzable;

use super::{bounds, shapes};

// Intersection tests
pub struct Intersection {}

// The actual intersection tests
impl Intersection {
    // Check if an AABB intersects another AABB
    pub fn aabb_aabb(aabb: &bounds::AABB, other: &bounds::AABB) -> bool {
        return aabb.min.elem_lte(&other.max).all() && other.min.elem_lte(&aabb.max).all();
    }
    // Check if a point is inside a sphere
    pub fn point_sphere(point: &veclib::Vector3<f32>, sphere: &shapes::Sphere) -> bool {
        return point.distance(sphere.center) < sphere.radius;
    }
    // Check if a point is inside an AABB
    pub fn point_aabb(point: &veclib::Vector3<f32>, aabb: &bounds::AABB) -> bool {
        return aabb.min.elem_lt(point).all() && aabb.max.elem_gt(point).all();
    }
    // Check if an AABB is intersecting a sphere
    pub fn aabb_sphere(aabb: &bounds::AABB, sphere: &shapes::Sphere) -> bool {
        let closest_point = aabb.get_nearest_point(&sphere.center);
        return Self::point_sphere(&closest_point, sphere);
    }
    // Check if a square intersects another square
    pub fn square_square(square: &shapes::Square, other: &shapes::Square) -> bool {
        return square.min.elem_lte(&other.max).all() && other.min.elem_lte(&square.max).all();
    }
    // Check if a screen space point is inside the NDC
    pub fn ss_point_limits(point: &veclib::Vector2<f32>) -> bool {
        let min = (point).elem_lt(&veclib::Vector2::ONE).all();
        let max = (point).elem_gt(&-veclib::Vector2::ONE).all();
        min && max
    }
    // Check if a segment intersects an aabb
    pub fn edge_aabb(_segment: &shapes::Line, _aabb: &bounds::AABB) -> bool {
        todo!();
    }
    // Frustum and an aabb
    pub fn frustum_aabb(frustum: &crate::Frustum, aabb: &bounds::AABB) -> bool {
        // Project the corners of the AABB
        let coordinates: Vec<veclib::Vector3<f32>> = (0..8).collect::<Vec<u8>>().into_iter().map(|x| aabb.get_corner(x)).collect();
        let projected_points = coordinates.into_iter().map(|x| {
            let point = &veclib::Vector4::new(x.x, x.y, x.z, 1.0);
            let point = frustum.inverse_matrix.mul_vector(point);
            point.get3([0, 1, 2]) / point.w
        }).collect::<Vec<veclib::Vector3<f32>>>();
        // Create a new AABB based on that
        let new_aabb = bounds::AABB::from_vertices(&projected_points);
        // Intersect that AABB with the AABB of the NDC
        Self::aabb_aabb(&bounds::AABB::ndc(), &new_aabb)
    }
}
