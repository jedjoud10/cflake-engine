use ecs::Component;

use crate::Physics;

// A character controller component that can be added onto a kinematic body to simulate character physics
#[derive(Component)]
pub struct CharacterController {
    pub velocity: vek::Vec3<f32>,
    pub air_control: f32,
    pub ground_control: f32,
    pub(crate) grounded: bool, 
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            velocity: vek::Vec3::zero(),
            air_control: 0.7,
            ground_control: 1.0,
            grounded: false,
        }
    }
}

impl CharacterController {
}