use crate::{
    Boundable, Movable, SharpVertices, SurfaceArea, Volume, AABB,
};

// A 3D cuboid that is defined by it's center and it's extent
#[derive(Clone, Copy)]
pub struct Cuboid {
    // Center of the cuboid
    pub center: vek::Vec3<f32>,

    // Full extent of the cubeoid
    pub extent: vek::Extent3<f32>,
}

impl Cuboid {
    // Create a cube from a center and an extent
    pub fn cube(center: vek::Vec3<f32>, extent: f32) -> Self {
        Self {
            center,
            extent: vek::Extent3::broadcast(extent),
        }
    }
}

impl From<AABB> for Cuboid {
    fn from(aabb: AABB) -> Self {
        Self {
            center: (aabb.max + aabb.min) / 2.0,
            extent: vek::Extent3::from(aabb.max - aabb.min),
        }
    }
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
        AABB {
            min: self.center - self.extent / 2.0,
            max: self.center + self.extent / 2.0,
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
        self.extent.product()
    }
}

impl SurfaceArea for Cuboid {
    fn surface_area(&self) -> f32 {
        let front = self.extent.w * self.extent.h;
        let side = self.extent.d * self.extent.h;
        let top = self.extent.w * self.extent.d;
        front * 2.0 + side * 2.0 + top * 2.0
    }
}

impl SharpVertices for Cuboid {
    type Points = [vek::Vec3<f32>; 8];

    // http://paulbourke.net/geometry/polygonise/
    fn points(&self) -> Self::Points {
        let max =
            self.center + vek::Vec3::<f32>::from(self.extent / 2.0);
        let min =
            self.center - vek::Vec3::<f32>::from(self.extent / 2.0);

        [
            min,
            vek::Vec3::new(max.x, min.y, min.z),
            vek::Vec3::new(max.x, min.y, max.z),
            vek::Vec3::new(min.x, min.y, max.z),
            vek::Vec3::new(min.x, max.y, min.z),
            vek::Vec3::new(max.x, max.y, min.z),
            max,
            vek::Vec3::new(min.x, max.y, max.z),
        ]
    }
}
