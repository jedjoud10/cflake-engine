use math::Scalar;
use crate::Component;

pub struct Forces {
    forces: Vec<Vec3>
}

impl Force {
    pub fn new() -> Self {
        Self(Vec3::new);
    }

    pub fn force_linear(&mut self, force: Vec3) {
        self::forces::push(force);
    }

    pub fn force(&mut self, force: Vec3, contact: Vec3, axis_pos: Vec3) {
        let d: f32;
        f32::sqrt(f32::pow(contact::x - axis_pos::x, 2) + f32::pow(contact::y - axis_pos::y, 2) + f32::pow(contact::z - axis_pos::z, 2));
        
    }

    pub fn apply(&mut self, &mut velocity: Vec3, mass: f32) {
        // Blasphemous code (real)
        for i in forces.iter_mut() {
            velocity += i / mass;
        }

        *forces.clear();
    }
}