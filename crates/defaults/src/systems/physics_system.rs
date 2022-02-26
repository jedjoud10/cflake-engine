pub mod rigidbody_system;
pub mod simulation_system;

use world::physics::rapier3d;

// Convert a position to a rapier3d translation
pub fn vec3_to_translation(pos: veclib::Vector3<f32>) -> rapier3d::prelude::Translation<f32> {
    rapier3d::prelude::Translation::<f32>::new(pos.x, pos.y, pos.z)
}
// Same for rotation
pub fn quat_to_rotation(quat: veclib::Quaternion<f32>) -> rapier3d::prelude::Rotation<f32> {
    let rotation = rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(
        quat[0], quat[1], quat[2], quat[3],
    ));
    rotation
}
// Convert a translation and rotation to an Isometry
pub fn transform(
    pos: veclib::Vector3<f32>,
    rot: veclib::Quaternion<f32>,
) -> rapier3d::prelude::Isometry<f32> {
    rapier3d::prelude::Isometry::<f32> {
        translation: vec3_to_translation(pos),
        rotation: quat_to_rotation(rot),
    }
}
