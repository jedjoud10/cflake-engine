mod angular_velocity;
mod character_controller;
mod colliders;
mod rigid_body;
mod velocity;
pub use character_controller::*;
pub use colliders::*;
pub use rigid_body::*;

pub use angular_velocity::AngularVelocity as UnmarkedAngularVelocity;
pub use velocity::Velocity as UnmarkedVelocity;

// Global coordinates updated frame to frame
pub type Velocity = velocity::Velocity<coords::Global<coords::FrameToFrame>>;
pub type AngularVelocity = angular_velocity::AngularVelocity<coords::Global<coords::FrameToFrame>>;

// Global coordinates updated during ticks for interpolation (first point)
pub type LastTickedVelocity = velocity::Velocity<coords::Global<coords::LastTick>>;
pub type LastTickedAngularVelocity =
    angular_velocity::AngularVelocity<coords::Global<coords::LastTick>>;

// Global coordinates updated during ticks for interpolation (last point)
pub type CurrentTickedVelocity = velocity::Velocity<coords::Global<coords::CurrentTick>>;
pub type CurrentTickedAngularVelocity =
    angular_velocity::AngularVelocity<coords::Global<coords::CurrentTick>>;
