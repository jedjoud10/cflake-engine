pub mod rigidbody_system;
pub mod simulation_system;
use world::physics::rapier3d;

// Convert a Rapier3D vector to a position
pub fn vector_to_vec3(vec: rapier3d::prelude::Vector<f32>) -> veclib::Vector3<f32> {
    veclib::Vector3::<f32>::new(vec[0], vec[1], vec[2])
}
// Convert a position to a Rapier3D vector
pub fn vec3_to_vector(pos: veclib::Vector3<f32>) -> rapier3d::prelude::Vector<f32> {
    rapier3d::prelude::Vector::<f32>::new(pos.x, pos.y, pos.z)
}
// Convert a position to a Rapier3D translation
pub fn vec3_to_translation(pos: veclib::Vector3<f32>) -> rapier3d::prelude::Translation<f32> {
    rapier3d::prelude::Translation::<f32>::new(pos.x, pos.y, pos.z)
}
// Convert a quaternion to a Rapier3D rotation
pub fn quat_to_rotation(quat: veclib::Quaternion<f32>) -> rapier3d::prelude::Rotation<f32> {
    rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(quat[0], quat[1], quat[2], quat[3]))
}
// Convert a Rapier3D rotation to a quaternion
pub fn rotation_to_quat(quat: rapier3d::prelude::Rotation<f32>) -> veclib::Quaternion<f32> {
    let mut bruh = veclib::Quaternion::<f32>::IDENTITY;
    bruh[0] = quat[0];
    bruh[1] = quat[1];
    bruh[2] = quat[2];
    bruh[3] = quat[3];
    bruh
}
// Convert a translation and rotation to an Isometry
pub fn transform(pos: veclib::Vector3<f32>, rot: veclib::Quaternion<f32>) -> rapier3d::prelude::Isometry<f32> {
    rapier3d::prelude::Isometry::<f32> {
        translation: vec3_to_translation(pos),
        rotation: quat_to_rotation(rot),
    }
}
