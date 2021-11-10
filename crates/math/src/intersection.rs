use veclib::Swizzable;
use super::{bounds, shapes};

// Intersection tests
pub struct Intersection;

// The actual intersection tests
impl Intersection {
    /* #region AABB stuff */
    // Check if an AABB intersects another AABB
    pub fn aabb_aabb(aabb: &bounds::AABB, other: &bounds::AABB) -> bool {
        aabb.min.elem_lt(&other.max).all() && other.min.elem_lt(&aabb.max).all()
    }
    // Check if a point is inside an AABB
    pub fn point_aabb(point: &veclib::Vector3<f32>, aabb: &bounds::AABB) -> bool {
        aabb.min.elem_lt(point).all() && aabb.max.elem_gt(point).all()
    }
    // Check if an AABB is intersecting a sphere
    pub fn aabb_sphere(aabb: &bounds::AABB, sphere: &crate::shapes::Shape) -> bool {
        match sphere.internal_shape {
            shapes::ShapeType::Cube(_) => todo!() /* This is not a fucking sphere you dumbass*/,
            shapes::ShapeType::Sphere(_) => {
                let closest_point = aabb.get_nearest_point(&sphere.center);
                Self::point_sphere(&closest_point, sphere)
            },
            shapes::ShapeType::AxisPlane(axis) => todo!(),
        }
    }
    // Frustum and an aabb
    pub fn frustum_aabb(frustum: &crate::Frustum, aabb: &bounds::AABB) -> bool {
        // Project the corners of the AABB
        let center_point = frustum.matrix.mul_vector(&veclib::Vector4::new(aabb.center.x, aabb.center.y, aabb.center.z, 1.0));
        let center_point = center_point.get3([0, 1, 2]) / center_point.w;
        let coordinates: Vec<veclib::Vector3<f32>> = (0..8).collect::<Vec<u8>>().into_iter().map(|x| aabb.get_corner(x)).collect();
        let projected_points = coordinates
            .into_iter()
            .map(|x| {
                let point = &veclib::Vector4::new(x.x, x.y, x.z, 1.0);
                let point = frustum.matrix.mul_vector(point);
                point.get3([0, 1, 2]) / point.w
            })
            .collect::<Vec<veclib::Vector3<f32>>>();
        let _test2 = projected_points.iter().any(|x| x.z > 0.0 && x.z < 1.0);
        // Create a new AABB based on that
        let new_aabb = bounds::AABB::new_vertices(&projected_points);
        let intersect = Self::aabb_aabb(&bounds::AABB::ndc_forward(), &new_aabb);
        let _test = center_point.x > -1.0 && center_point.x < 1.0 && center_point.y > -1.0 && center_point.y < 1.0 && center_point.z < 1.0 && center_point.z > 0.0;
        intersect
    }
    // CSG shape and an abb
    pub fn csgshape_aabb(csgshape: &crate::csg::CSGShape, aabb: &bounds::AABB) -> bool {
        let center = csgshape.internal_shape.center;
        let intersection = match csgshape.internal_shape.internal_shape {
            shapes::ShapeType::Cube(half_extent) => {
                // Lol let's use the function that I already made kek
                let csg_aabb = crate::bounds::AABB::new_center_halfextent(center, half_extent);
                Self::aabb_aabb(aabb, &csg_aabb)
            },
            shapes::ShapeType::Sphere(radius) => {
                // Same stuff here
                Self::aabb_sphere(aabb, &csgshape.internal_shape)
            },
            shapes::ShapeType::AxisPlane(axis) => {
                // Sometimes you just want to kill yourlse
                todo!()
            }
        };
        intersection
    }
    // CSG tree and an aabb
    pub fn csgtree_aabb(csgtree: &crate::csg::CSGTree, aabb: &bounds::AABB) -> bool {
        // Loop through each node in the tree and keep track of the base intersection
        let mut base_intersection = false;
        for node in csgtree.nodes.iter() {
            // Calculate the new intersection
            let new_intersection = Self::csgshape_aabb(node, aabb);
            match node.csg_type {
                crate::constructive_solid_geometry::CSGType::Union => base_intersection |= new_intersection,
                crate::constructive_solid_geometry::CSGType::Difference => {},
                crate::constructive_solid_geometry::CSGType::Intersection => {},
            }
        }
        base_intersection
    }
    /* #endregion */ 
    /* #region Others */
    // Check if a point is inside a sphere
    pub fn point_sphere(point: &veclib::Vector3<f32>, sphere: &shapes::Shape) -> bool {
        match sphere.internal_shape {
            shapes::ShapeType::Cube(_) => todo!() /* Not a sphere */,
            shapes::ShapeType::Sphere(radius) => point.distance(sphere.center) < radius,
            shapes::ShapeType::AxisPlane(_) => todo!(),
        }        
    }
    /* #endregion */    
}
