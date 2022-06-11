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
    // Create an AABB at a specified center position and half-extent scale
    pub fn new(center: vek::Vec3<f32>, half_extent: vek::Extent3<f32>) -> Self {
        Self {
            min: center - vek::Vec3::from(half_extent),
            max: center + vek::Vec3::from(half_extent),
        }
    }

    // Get all the vertices of this AABB, in the order that is defined on this website
    // http://paulbourke.net/geometry/polygonise/
    pub fn points(&self) -> [vek::Vec3<f32>; 8] {
        [
            self.min,
            vek::Vec3::new(self.max.x, self.min.y, self.min.z),
            vek::Vec3::new(self.max.x, self.min.y, self.max.z),
            vek::Vec3::new(self.min.x, self.min.y, self.max.z),
            vek::Vec3::new(self.min.x, self.max.y, self.min.z),
            vek::Vec3::new(self.max.x, self.max.y, self.min.z),
            vek::Vec3::new(self.max.x, self.max.y, self.max.z),
            vek::Vec3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    // Calculate the center of the AABB
    pub fn get_center(&self) -> vek::Vec3<f32> {
        (self.min + self.max) / 2.0
    }

    // Calculate the full extent of the AABB
    pub fn get_extent(&self) -> vek::Extent3<f32> {
        vek::Extent3::from(self.max - self.min)
    }

    // Check if the AABB is valid (it's max point is indeed bigger than min)
    pub fn is_valid(&self) -> bool {
        let mask = self.max.partial_cmpgt(&self.min);
        mask.x & mask.y & mask.z
    }
}

// Fetch the min/max vertices using an index
impl Index<usize> for AABB {
    type Output = vek::Vec3<f32>;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            &self.min
        } else if index == 1 {
            &self.max
        } else {
            panic!()
        }
    }
}
