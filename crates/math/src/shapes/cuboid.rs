use crate::{AABB, Movable, Boundable, Volume, Area};

// A 3D cuboid that is defined by it's center and it's extent
#[derive(Clone, Copy)]
pub struct Cuboid {
    pub center: vek::Vec3<f32>,
    pub extent: vek::Extent3<f32>,
}

impl Movable for Cuboid {
    fn center(&self) -> vek::Vec3<f32> {
        self.center
    }

    fn set_center(&mut self, new: vek::Vec3<f32>) {
        self.center = new
    }
}

impl Boundable for Cuboid {
    fn bounds(&self) -> AABB {
        let half_extent = vek::Vec3::<f32>::from(self.extent) / 2.0;
        AABB {
            min: self.center - half_extent,
            max: self.center + half_extent,
        }
    }

    fn scale_by(&mut self, scale: f32) {
        self.extent *= scale;
    }

    fn expand_by(&mut self, expand_units: f32) {
        self.extent += vek::Extent3::broadcast(expand_units); 
    }
}

impl Volume for Cuboid {
    fn volume(&self) -> f32 {
        self.extent.w * self.extent.h * self.extent.d
    }
}

impl Area for Cuboid {
    fn area(&self) -> f32 {
        let front = self.extent.w * self.extent.h;
        let side = self.extent.d * self.extent.h;
        let top = self.extent.w * self.extent.d;
        front * 2.0 + side * 2.0 + top * 2.0
    }
}
