use ecs::Component;

// A character controller component that can be added onto a kinematic body to simulate character physics
#[derive(Component)]
pub struct CharacterController {
    pub max_speed: f32,
    pub acceleration: f32,
    pub direction: vek::Vec3<f32>,
    pub air_control: f32,
    pub ground_control: f32,
    pub jump_force: f32,
    pub grounded: bool,
    pub jumping: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            max_speed: 10.0,
            acceleration: 10.0,
            direction: Default::default(),
            air_control: 0.7,
            ground_control: 1.0,
            jump_force: 8.0,
            grounded: false,
            jumping: false,
        }
    }
}

impl CharacterController {
    // Move the character controller in a specific direction
    pub fn move_with(&mut self, direction: vek::Vec3<f32>) {
        self.direction = direction;
    }

    // Make the character controller jump
    pub fn jump(&mut self) {
        self.jumping = true;
    }
}
