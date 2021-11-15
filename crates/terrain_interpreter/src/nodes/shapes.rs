use crate::{
    error::InterpreterError,
    var_hash::{PassedData, VarHashType},
    var_hash_getter::VarHashGetter,
    NodeInterpreter,
};

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
            math::shapes::ShapeType::Cube(half_extent) => format!("sdBox({}, {})", position_string, format!("vec3({}, {}, {})", half_extent.x, half_extent.y, half_extent.z)),
            math::shapes::ShapeType::Sphere(radius) => format!("sdSphere({}, {})", position_string, radius),
            math::shapes::ShapeType::AxisPlane(axis, (offset_min, offset_max)) => match axis {
                veclib::Vec3Axis::X => format!("pos.x - {}", offset_min),
                veclib::Vec3Axis::Y => format!("pos.y - {}", offset_min),
                veclib::Vec3Axis::Z => format!("pos.z - {}", offset_min),
            },
        }
        .to_string())
    }
    fn get_output_type(&self, _getter: &VarHashGetter) -> VarHashType {
        VarHashType::Density
    }
    // Update the csg tree
    fn update_csgtree(&self, passed_data: &mut PassedData, getter: &VarHashGetter, csgtree: &mut math::constructive_solid_geometry::CSGTree, self_range: (f32, f32)) {
        // Since we are a CSG shape ourselves, add it to the csgtree with "Union" csg type
        let mut shape = self.clone();
        shape.csg_type = math::csg::CSGType::Union;
        // Create a custom shape indentifier
        passed_data.custom_shape_identifier = Some(math::csg::CSGCustomIdentifier {
            hash: getter.self_varhash.unwrap().index as u64,
        });
        csgtree.add_custom_identifier(passed_data.custom_shape_identifier.unwrap(), shape);
    }
}
