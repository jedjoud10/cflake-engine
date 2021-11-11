use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};

// Final density
pub type Shape = math::csg::CSGShape;

impl NodeInterpreter for Shape {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // I am dead inside
        let pos = getter.get(0, VarHashType::Vec3)?;
        let center = self.internal_shape.center;
        let center_string = format!("vec3({}, {}, {})", center.x, center.y, center.z);
        // Depends on the shape
        Ok(match self.internal_shape.internal_shape {
            math::shapes::ShapeType::Cube(half_extent) => format!("sdCube(({}+{}), {})", pos.get_name(), center_string, half_extent),
            math::shapes::ShapeType::Sphere(radius) => format!("sdSphere(({}+{}), {})", pos.get_name(), center_string, radius),
            math::shapes::ShapeType::AxisPlane(_) => todo!(),
        }.to_string())
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
    // Update the csg tree
    fn update_csgtree(&self, getter: &VarHashGetter, csgtree: &mut math::constructive_solid_geometry::CSGTree) {
        // Since we are a CSG shape ourselves, add it to the csgtree with "Union" csg type
        let mut shape = self.clone();
        shape.csg_type = math::csg::CSGType::Union;
        csgtree.add(shape); 
    }
}
