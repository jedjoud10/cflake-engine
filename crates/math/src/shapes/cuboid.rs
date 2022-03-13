// A simple 3D cuboid
#[derive(Clone)]
pub struct Cuboid {
    // Common
    pub center: vek::Vec3<f32>,

    // This is the full size of the cuboid, not the half-extent
    pub size: vek::Vec3<f32>,
}

impl Default for Cuboid {
    fn default() -> Self {
        Self { center: Default::default(), size: vek::Vec3::one() }
    }
}