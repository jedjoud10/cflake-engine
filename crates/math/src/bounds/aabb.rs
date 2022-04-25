use std::ops::Index;

// An axis aligned bounding box
#[derive(Default)]
pub struct AABB {
    // Minimum vertex in world space
    pub min: vek::Vec3<f32>,

    // Maximum vertex in world space
    pub max: vek::Vec3<f32>,
}

impl AABB {
    // Create an AABB at a specified position and half-width scale
    pub fn new(pos: vek::Vec3<f32>, hw: vek::Vec3<f32>) -> Self {
        Self {
            min: pos - hw,
            max: pos + hw,
        }
    }
    // Create all the points that belong to this AABB in arbitrary order cause I can't give a shit
    pub fn points(&self) -> [vek::Vec3<f32>; 8] {
        [self.min, 
        vek::Vec3::new(self.min.x, self.min.y, self.max.z),
        vek::Vec3::new(self.min.x, self.max.y, self.max.z),
        vek::Vec3::new(self.min.x, self.max.y, self.min.z),
        vek::Vec3::new(self.max.x, self.min.y, self.min.z),
        vek::Vec3::new(self.max.x, self.max.y, self.min.z),
        vek::Vec3::new(self.max.x, self.min.y, self.max.z),
        self.max]
    }
}

// Fetch the min/max vertices using an index
impl Index<usize> for AABB {
    type Output = vek::Vec3<f32>;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            &self.min
        } else if index == 1{
            &self.max
        } else {
            panic!("no")
        }
    }
}

// Trait to convert any shape to an AABB
pub trait ToAABB {
    fn aabb(&self) -> AABB;
}
