

use crate::{Movable, Boundable};

// An axis aligned bounding box
#[derive(Clone, Copy)]
pub struct AABB {
    pub min: vek::Vec3<f32>,
    pub max: vek::Vec3<f32>,
}

impl AABB {
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

    // Check if the AABB is valid (it's max point is indeed bigger than min)
    pub fn is_valid(&self) -> bool {
        let mask = self.max.partial_cmpgt(&self.min);
        mask.x & mask.y & mask.z
    }
}

impl Movable for AABB {
    fn center(&self) -> vek::Vec3<f32> {
        (self.min + self.max) / 2.0
    }

    fn set_center(&mut self, new: vek::Vec3<f32>) {
        let diff = new - self.center();
        self.min += diff;
        self.max += diff;
    }
}

impl Boundable for AABB {
    fn bounds(&self) -> AABB {
        *self
    }

    fn scale_by(&mut self, _scale: f32) {
        todo!()
    }

    fn expand_by(&mut self, expand_units: f32) {
        self.min -= vek::Vec3::broadcast(expand_units / 2.0);
        self.max += vek::Vec3::broadcast(expand_units / 2.0);
    }
}