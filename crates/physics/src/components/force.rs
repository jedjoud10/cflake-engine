


use std::ops::Div;

pub fn impulse_real(
    velocity: &mut vek::Vec3<f32>,
    angular_velocity: &mut vek::Quaternion<f32>,
    force: &vek::Vec3<f32>,
    collision_point: &vek::Vec3<f32>,
    position: &vek::Vec3<f32>,
    dt: &f32,
    mass: &f32,
) {
    // LINEAR FORCE RELATED

    let acceleration = force / mass;
    *velocity += acceleration * *dt;

    // TORQUE RELATED

    // Moment arm for torque calculation
    let moment_arm = collision_point - position;

    // Fuck moment of inertia all my homies hate realistic angular velocity calc
    let ixx = (mass / 12.0) * (2.0);
    let iyy = (mass / 12.0) * (2.0);
    let izz = (mass / 12.0) * (2.0);
    let moment_of_inertia = vek::Vec3::new(ixx, iyy, izz);

    // Calculate the torque and linear acceleration exerted on the object
    let torque: vek::Vec3<f32> = moment_arm.cross(*force);

    // Update the object's angular velocity based on the torque and the object's moment of inertia
    let angular_acceleration = torque.div(&moment_of_inertia);

    // Integrate the angular velocity to update the rotation quaternion
    let mut delta_rotation: vek::Quaternion<f32> = vek::Quaternion::<f32>::identity();
    delta_rotation.rotate_x(angular_acceleration.x * *dt / 2.0);
    delta_rotation.rotate_y(angular_acceleration.y * *dt / 2.0);
    delta_rotation.rotate_z(angular_acceleration.z * *dt / 2.0);

    // Integrate the velocity to update the position
    angular_velocity.rotate_x(delta_rotation.x);
    angular_velocity.rotate_y(delta_rotation.y);
    angular_velocity.rotate_z(delta_rotation.z);
}
