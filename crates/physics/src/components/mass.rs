use ecs::Component;

// Depicts the mass of an object in kilograms
// Assumes the object has uniform mass throughout
#[derive(Component)]
pub struct Mass(pub f32);

impl Mass {
    /// Neue Massenkomponente (real)
    pub fn new(mass_kg: f32) -> Self {
        Self(mass_kg)
    }
}