

use crate::{Movable, Boundable, SharpVertices, Volume, SurfaceArea};

// An axis aligned bounding box
#[derive(Clone, Copy)]
pub struct AABB {
    pub min: vek::Vec3<f32>,
    pub max: vek::Vec3<f32>,
}

impl AABB {
    // Create a new AABB from a list of points in 3D space
    pub fn from_points(points: &[vek::Vec3<f32>]) -> Option<Self> {
        if points.len() < 2 {
            return None;
        }

        // Initial values set to their inverse (since we have multiple iterations)
        let mut min = vek::Vec3::broadcast(f32::MAX);
        let mut max = vek::Vec3::broadcast(f32::MIN);

        for point in points {
            // Update the "max" bound element wise
            for (point_element, max_element) in point.as_slice().iter().zip(max.as_mut_slice().iter_mut()) {
                *max_element = f32::max(*max_element, *point_element)
            }

            // Update the "min" bound element wise
            for (point_element, min_element) in point.as_slice().iter().zip(min.as_mut_slice().iter_mut()) {
                *min_element = f32::max(*min_element, *point_element)
            }
        }

        // Check if the AABB would be valid
        (min != max).then_some(Self {
            min,
            max,
        })
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

    fn scale_by(&mut self, scale: f32) {
        let center = (self.min + self.max) / 2.0;
        let min = (self.min - center) * scale;
        let max = (self.max - center) * scale;
        self.min = min + center;
        self.max = max + center;
    }

    fn expand_by(&mut self, expand_units: f32) {
        self.min -= vek::Vec3::broadcast(expand_units / 2.0);
        self.max += vek::Vec3::broadcast(expand_units / 2.0);
    }
}

impl Volume for AABB {
    fn volume(&self) -> f32 {
        (self.max - self.min).product()
    }
}

impl SurfaceArea for AABB {
    fn surface_area(&self) -> f32 {
        let extent = vek::Extent3::<f32>::from(self.max - self.min);
        let front = extent.w * extent.h;
        let side = extent.d * extent.h;
        let top = extent.w * extent.d;
        front * 2.0 + side * 2.0 + top * 2.0
    }
}

impl SharpVertices for AABB {
    type Points = [vek::Vec3<f32>; 8];

    // http://paulbourke.net/geometry/polygonise/
    fn points(&self) -> Self::Points {
        [
            self.min,
            vek::Vec3::new(self.max.x, self.min.y, self.min.z),
            vek::Vec3::new(self.max.x, self.min.y, self.max.z),
            vek::Vec3::new(self.min.x, self.min.y, self.max.z),
            vek::Vec3::new(self.min.x, self.max.y, self.min.z),
            vek::Vec3::new(self.max.x, self.max.y, self.min.z),
            self.max,
            vek::Vec3::new(self.min.x, self.max.y, self.max.z),
        ]
    }
}