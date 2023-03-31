use crate::Component;

pub struct Mass(f32);

impl Mass {
    /// Neue Massenkomponente (real)
    pub fn new(mass_kg: f32) -> Self {
        Self(mass_kg)
    }
}