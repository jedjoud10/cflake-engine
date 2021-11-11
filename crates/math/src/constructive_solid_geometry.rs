use crate::shapes::{Shape, ShapeType};

/* #region Some starter data types */
// CSG type
#[derive(Clone, Copy)]
pub enum CSGType {
    Union,
    Difference,
    Intersection,
}
// A main CSG shape struct
#[derive(Clone)]
pub struct CSGShape {
    pub internal_shape: Shape,
    pub csg_type: CSGType,
}
impl CSGShape {
    // New cube
    pub fn new_cube(center: veclib::Vector3<f32>, half_extent: veclib::Vector3<f32>, csg_type: CSGType) -> Self {
        Self {
            csg_type,
            internal_shape: Shape::new_cube(center, half_extent),
        }
    }
    // New sphere
    pub fn new_sphere(center: veclib::Vector3<f32>, radius: f32, csg_type: CSGType) -> Self {
        Self {
            csg_type,
            internal_shape: Shape::new_sphere(center, radius),
        }
    }
    // Expand this CSG shape using the specified method
    pub fn expand(&mut self, expand_method: ExpandMethod) {
        // Check the internal shape type first, some internal shape and expand method combinations might not work. Ex (ShapeTpe: Sphere and ExpandMethod: Vector)
        match &mut self.internal_shape.internal_shape {
            ShapeType::Cube(half_extents) => match expand_method {
                ExpandMethod::Factor(x) => *half_extents += veclib::Vector3::ONE * x,
                ExpandMethod::Vector(x) => *half_extents += x,
            },
            ShapeType::Sphere(radius) => match expand_method {
                ExpandMethod::Factor(x) => *radius += x,
                ExpandMethod::Vector(x) => todo!(),
            },
            ShapeType::AxisPlane(axis) => {
                todo!()
            }
        }
    }
}
// Expand method
pub enum ExpandMethod {
    Factor(f32),
    Vector(veclib::Vector3<f32>),
}
/* #endregion */
/* #region A simple CSG tree for easier usage */
#[derive(Default)]
pub struct CSGTree {
    // Nodes
    pub nodes: Vec<CSGShape>,
}

impl CSGTree {
    // Add a node to the tree
    pub fn add(&mut self, node: CSGShape) {
        self.nodes.push(node);
    }
    // Get a specific node
    pub fn get(&self, node_index: usize) -> &CSGShape {
        self.nodes.get(node_index).unwrap()
    }
    // Get a specific node mutably
    pub fn get_mut(&mut self, node_index: usize) -> &mut CSGShape {
        self.nodes.get_mut(node_index).unwrap()
    }
}

/* #endregion */
