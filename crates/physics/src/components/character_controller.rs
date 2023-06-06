use ecs::Component;

use crate::Physics;

// A character controller component that can be added onto a kinematic body to simulate character physics
#[derive(Component)]
pub struct CharacterController {
    controller: rapier3d::control::KinematicCharacterController,
    
}

impl CharacterController {
    // Create a new character controller
    pub fn new(up: vek::Vec3<f32>, offset: f32,) -> Self {
        let controller = rapier3d::control::KinematicCharacterController {
            up: rapier3d::na::UnitVector3::new_normalize(crate::vek_vec_to_na_vec(up)),
            offset: rapier3d::control::CharacterLength::Absolute(offset),
            slide: false,
            autostep: None,
            max_slope_climb_angle: 45.0f32.to_radians(),
            min_slope_slide_angle: 25.0f32.to_radians(),
            snap_to_ground: None,
        };

        Self {
            controller,
        }
    }

    // Move the character controller in a specific direction
    pub fn translate(&mut self,) {

    }
}