pub mod rigidbody_system;
pub mod simulation_system;

// Convert a Rapier3D vector to a position
pub fn vector_to_vec3(vec: rapier3d::prelude::Vector<f32>) -> vek::Vec3<f32> {
    vek::Vec3::<f32>::new(vec[0], vec[1], vec[2])
}
// Convert a position to a Rapier3D vector
pub fn vec3_to_vector(pos: vek::Vec3<f32>) -> rapier3d::prelude::Vector<f32> {
    rapier3d::prelude::Vector::<f32>::new(pos.x, pos.y, pos.z)
}
// Convert a position to a Rapier3D translation
pub fn vec3_to_translation(pos: vek::Vec3<f32>) -> rapier3d::prelude::Translation<f32> {
    rapier3d::prelude::Translation::<f32>::new(pos.x, pos.y, pos.z)
}
// Convert a position to a Rapier3D point
pub fn vec3_to_point(pos: vek::Vec3<f32>) -> rapier3d::prelude::Point<f32> {
    rapier3d::prelude::Point::<f32>::new(pos.x, pos.y, pos.z)
}
// Convert a quaternion to a Rapier3D rotation
pub fn quat_to_rotation(quat: vek::Quaternion<f32>) -> rapier3d::prelude::Rotation<f32> {
    panic!()
}
// Convert a Rapier3D rotation to a quaternion
pub fn rotation_to_quat(quat: rapier3d::prelude::Rotation<f32>) -> vek::Quaternion<f32> {
    panic!()
}
// Convert a translation and rotation to an Isometry
pub fn transform(pos: vek::Vec3<f32>, rot: vek::Quaternion<f32>) -> rapier3d::prelude::Isometry<f32> {
    rapier3d::prelude::Isometry::<f32> {
        translation: vec3_to_translation(pos),
        rotation: quat_to_rotation(rot),
    }
}
