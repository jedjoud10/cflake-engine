// Convert a translation into a vec3
pub(crate) fn trans_to_vec(trans: rapier3d::na::Translation3<f32>) -> vek::Vec3<f32> {
    vek::Vec3::new(trans.x, trans.y, trans.z)
} 


// Convert a rotation into a quaternion
pub(crate) fn rot_to_quat(rot: rapier3d::na::UnitQuaternion<f32>) -> vek::Quaternion<f32> {
    vek::Quaternion {
        x: rot.i,
        y: rot.j,
        z: rot.k,
        w: rot.w,
    }
}


// Convert a vec3 into a translation
pub(crate) fn vec_to_trans(vec: vek::Vec3<f32>) -> rapier3d::na::Translation3<f32> {
    rapier3d::na::Translation3::<f32>::new(vec.x, vec.y, vec.z)
}

// Convert a vec3 into a vector 
pub(crate) fn vek_vec_to_na_vec(vec: vek::Vec3<f32>) -> rapier3d::na::Vector3<f32> {
    rapier3d::na::Vector3::<f32>::new(vec.x, vec.y, vec.z)
}

// Convert a vec3 into a point
pub(crate) fn vek_vec_to_na_point(vec: vek::Vec3<f32>) -> rapier3d::na::Point3<f32> {
    rapier3d::na::Point3::<f32>::new(vec.x, vec.y, vec.z)
}

// Convert a vector into a vec3 
pub(crate) fn na_vec_to_vek_vec(vec: rapier3d::na::Vector3<f32>) -> vek::Vec3<f32> {
    vek::Vec3::<f32>::new(vec.x, vec.y, vec.z)
}

// Convert a quaternion into a rotation
pub(crate) fn quat_to_rot(quat: vek::Quaternion<f32>) -> rapier3d::na::UnitQuaternion<f32> {
    rapier3d::na::UnitQuaternion::<f32>::from_quaternion(rapier3d::na::Quaternion::new(quat.w, quat.x, quat.y, quat.z))
}

// Convert an isometry into a translation and rotation
pub(crate) fn isometry_to_trans_rot(isometry: &rapier3d::na::Isometry3<f32>) -> (vek::Vec3<f32>, vek::Quaternion<f32>) {
    let translation = isometry.translation;
    let rotation = isometry.rotation;
    (trans_to_vec(translation), rot_to_quat(rotation))
}

// Convert a translation and rotation into an isometry
pub(crate) fn trans_rot_to_isometry(vec: vek::Vec3<f32>, quat: vek::Quaternion<f32>) -> rapier3d::na::Isometry3<f32> {
    rapier3d::na::Isometry3::<f32>::from_parts(vec_to_trans(vec), quat_to_rot(quat))
}