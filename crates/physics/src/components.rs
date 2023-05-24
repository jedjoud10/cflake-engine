mod angular_velocity;
mod velocity;
mod rigid_body;
mod colliders;
pub use colliders::*;
pub use rigid_body::*;

// Global coordinates
pub type Velocity = velocity::Velocity<coords::Global>;
pub type AngularVelocity = angular_velocity::AngularVelocity<coords::Global>;

// Local coordinates
pub type LocalVelocity = velocity::Velocity<coords::Local>;
pub type LocalAngularVelocity = angular_velocity::AngularVelocity<coords::Local>;

