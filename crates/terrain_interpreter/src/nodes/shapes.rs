use crate::{error::InterpreterError, var_hash::VarHashType, var_hash_getter::VarHashGetter, NodeInterpreter};

// Final density
pub type Shape = math::csg::CSGShape;

impl NodeInterpreter for Shape {
    fn get_node_string(&self, getter: &VarHashGetter) -> Result<String, InterpreterError> {
        // I am dead inside
        let pos = getter.get(0, VarHashType::Vec3)?;
        let center = self.internal_shape.center;
        let center_string = format!("vec3({}, {}, {})", center.x, center.y, center.z);
        let position_string = format!("({} - {})", pos.get_name(), center_string);
        // Depends on the shape
        Ok(match self.internal_shape.internal_shape {
            math::shapes::ShapeType::Cube(half_extent) => format!(
                "sdBox({}, {})",
                position_string,
                format!("vec3({}, {}, {})", half_extent.x, half_extent.y, half_extent.z)
            ),
            math::shapes::ShapeType::Sphere(radius) => format!("sdSphere({}, {})", position_string, radius),
            math::shapes::ShapeType::AxisPlane(_) => todo!(),
        }
        .to_string())
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
    // Update the csg tree
    fn update_csgtree(&self, getter: &VarHashGetter, csgtree: &mut math::constructive_solid_geometry::CSGTree, self_range: (f32, f32)) {
        // Since we are a CSG shape ourselves, add it to the csgtree with "Union" csg type
        let mut shape = self.clone();
        shape.csg_type = math::csg::CSGType::Union;
        let identifier = crate::var_hash::convert_csg_custom_identifier(&getter.self_varhash.unwrap());
        csgtree.add_custom_identifier(identifier, shape);
    }
}
