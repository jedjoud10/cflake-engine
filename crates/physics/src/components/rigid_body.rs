use ecs::Component;
pub use rapier3d::prelude::LockedAxes;
pub use rapier3d::prelude::RigidBodyType;

// A rigidbody is an object that is affect by external forces and collisions
// It basically represents any physics object in the world scene that 
#[derive(Component)]
pub struct RigidBody {
    pub _type: RigidBodyType,
    pub(crate) interpolated: bool,
    pub(crate) sleeping: bool,
    pub(crate) locked: LockedAxes,
    pub(crate) handle: Option<rapier3d::dynamics::RigidBodyHandle>,
    pub(crate) impulses: Vec<vek::Vec3<f32>>,
    pub(crate) impulses_at_points: Vec<(vek::Vec3<f32>, vek::Vec3<f32>)>,
    pub(crate) torque_impulses: Vec<vek::Vec3<f32>>,
    pub(crate) forces: Vec<vek::Vec3<f32>>,
    pub(crate) forces_at_points: Vec<(vek::Vec3<f32>, vek::Vec3<f32>)>,
    pub(crate) torques: Vec<vek::Vec3<f32>>,
}

impl Clone for RigidBody {
    fn clone(&self) -> Self {
        Self {
            _type: self._type.clone(),
            sleeping: false,
            handle: None,
            interpolated: self.interpolated,
            locked: LockedAxes::empty(),
            impulses: self.impulses.clone(),
            impulses_at_points: self.impulses_at_points.clone(),
            torque_impulses: self.torque_impulses.clone(),
            forces: self.forces.clone(),
            forces_at_points: self.forces_at_points.clone(),
            torques: self.torques.clone(),
        }
    }
}

impl RigidBody {
    // Create a new RigidBody with a specific mass in kG
    pub fn new(_type: RigidBodyType, interpolated: bool, locked: LockedAxes) -> Self {
        Self { 
            _type,
            interpolated,
            handle: None,
            locked,
            sleeping: false,
            impulses: Default::default(),
            impulses_at_points: Default::default(),
            torque_impulses: Default::default(),
            forces: Default::default(),
            forces_at_points: Default::default(),
            torques: Default::default(),
        }
    }

    // Check if the rigidbody is being interpolated
    pub fn is_interpolated(&self) -> bool {
        self.interpolated
    }

    // Check if the RigidBody is currently sleeping
    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    // Apply an impulse on the rigid-body
    pub fn apply_impulse(&mut self, impulse: vek::Vec3<f32>) {
        self.impulses.push(impulse);
    }

    // Apply a torque impulse on the rigid-body
    pub fn apply_torque_impulse(&mut self, impulse: vek::Vec3<f32>) {
        self.torque_impulses.push(impulse);
    }

    // Apply an impulse at a specific point on the rigid-body
    pub fn apply_impulse_at_point(&mut self, impulse: vek::Vec3<f32>, point: vek::Vec3<f32>) {
        self.impulses_at_points.push((impulse, point));
    }

    // Add a force to the rigid-body
    pub fn apply_force(&mut self, force: vek::Vec3<f32>) {
        self.forces.push(force);
    }

    // Add a torque force on the rigid-body
    pub fn apply_torque(&mut self, torque: vek::Vec3<f32>) {
        self.torques.push(torque);
    }

    // Apply a force at a specific point on the rigid-body
    pub fn apply_force_at_point(&mut self, force: vek::Vec3<f32>, point: vek::Vec3<f32>) {
        self.forces_at_points.push((force, point));
    }
}