use std::collections::HashMap;

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
// A custom identifier so we can get CSG shapes without the need to get their index
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CSGCustomIdentifier {
    pub hash: u64,
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
    pub identifier_hashmap: HashMap<CSGCustomIdentifier, usize>,
}

impl CSGTree {
    // Add a node to the tree
    pub fn add(&mut self, node: CSGShape) {
        self.nodes.push(node);
    }
    // Add a node with a custom identifier to the tree
    pub fn add_custom_identifier(&mut self, identifier: CSGCustomIdentifier, node: CSGShape) {
        let id = self.nodes.len();
        self.add(node);
        self.identifier_hashmap.insert(identifier, id);
    }
    // Get a specific node using a custom identifier
    pub fn get_custom(&self, identifier: CSGCustomIdentifier) -> Option<&CSGShape> {
        let index = *self.identifier_hashmap.get(&identifier)?;
        return self.get(index);
    }
    // Get a specific node mutably, using a custom identifier
    pub fn get_custom_mut(&mut self, identifier: CSGCustomIdentifier) -> Option<&mut CSGShape> {
        let index = *self.identifier_hashmap.get(&identifier)?;
        return self.get_mut(index);
    }
    // Get a specific node
    pub fn get(&self, node_index: usize) -> Option<&CSGShape> {
        self.nodes.get(node_index)
    }
    // Get a specific node mutably
    pub fn get_mut(&mut self, node_index: usize) -> Option<&mut CSGShape> {
        self.nodes.get_mut(node_index)
    }
}

/* #endregion */
