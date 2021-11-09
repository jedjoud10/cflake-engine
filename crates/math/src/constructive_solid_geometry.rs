/* #region Some starter data types */
// CSG type
pub enum CSGType {
    Union,
    Difference,
    Intersection
}
// A main shape struct
pub struct CSGShape {
    // The center of the Constructive Solid Geometry shape
    pub center: veclib::Vector3<f32>,
    pub csg_type: CSGType,
    pub internal_shape: ShapeType
}
impl CSGShape {
    // New cube
    pub fn new_cube(center: veclib::Vector3<f32>, bounds: veclib::Vector3<f32>, csg_type: CSGType) -> Self {
        Self {
            center,
            csg_type,
            internal_shape: ShapeType::Cube(bounds),
        }
    }
    // New sphere
    pub fn new_sphere(center: veclib::Vector3<f32>, radius: f32, csg_type: CSGType) -> Self {
        Self {
            center,
            csg_type,
            internal_shape: ShapeType::Sphere(radius),
        }
    }
    // Expand this CSG shape using the specified method
    pub fn expand(&mut self, expand_method: ExpandMethod) {
        // Check the internal shape type first, some internal shape and expand method combinations might not work. Ex (ShapeTpe: Sphere and ExpandMethod: Vector)
        match &mut self.internal_shape {
            ShapeType::Cube(half_extents) => {
                match expand_method {
                    ExpandMethod::Factor(x) => *half_extents += veclib::Vector3::ONE * x,
                    ExpandMethod::Vector(x) => *half_extents += x,
                }
            },
            ShapeType::Sphere(radius) => {
                match expand_method {
                    ExpandMethod::Factor(x) => *radius += x,
                    ExpandMethod::Vector(x) => todo!(),
                }
            },
        }
    }
}
// Expand method
pub enum ExpandMethod {
    Factor(f32),
    Vector(veclib::Vector3<f32>)
}
// Shape type
pub enum ShapeType {
    Cube(veclib::Vector3<f32>),
    Sphere(f32),
}
/* #endregion */
/* #region A simple CSG tree for easier usage */
pub struct CSGTree {
    // Nodes
    pub nodes: Vec<CSGShape>,
}

impl CSGTree {
    // Add a node to the tree
    pub fn add(&mut self, node: CSGShape) {
        self.nodes.push(node);
    }
}
/* #endregion */