use crate::{Boundable, Movable, SharpVertices, SurfaceArea, Volume};

// An axis aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct Aabb<T> {
    pub min: vek::Vec3<T>,
    pub max: vek::Vec3<T>,
}

impl<T> Aabb<T> {
    // Check if the AABB is valid (it's max point is indeed bigger than min)
    pub fn is_valid(&self) -> bool where T: PartialOrd {
        let mask = self.max.partial_cmpgt(&self.min);
        mask.x & mask.y & mask.z
    }
}

macro_rules! impl_shape_traits {
    ($t:ty) => {
        impl Movable<$t> for Aabb<$t> {
            fn center(&self) -> vek::Vec3<$t> {
                (self.min + self.max) / 2.0
            }
        
            fn set_center(&mut self, new: vek::Vec3<$t>) {
                let diff = new - self.center();
                self.min += diff;
                self.max += diff;
            }
        }
        
        impl Boundable<$t> for Aabb<$t> {
            fn bounds(&self) -> Aabb<$t> {
                *self
            }
        
            fn scale_by(&mut self, scale: $t) {
                let center = (self.min + self.max) / 2.0;
                let min = (self.min - center) * scale;
                let max = (self.max - center) * scale;
                self.min = min + center;
                self.max = max + center;
            }
        
            fn expand_by(&mut self, units: $t) {
                self.min -= vek::Vec3::broadcast(units / 2.0);
                self.max += vek::Vec3::broadcast(units / 2.0);
            }
        }
        
        impl Volume<$t> for Aabb<$t> {
            fn volume(&self) -> $t {
                (self.max - self.min).product()
            }
        }
        
        impl SurfaceArea<$t> for Aabb<$t> {
            fn area(&self) -> $t {
                let extent = vek::Extent3::<$t>::from(self.max - self.min);
                let front = extent.w * extent.h;
                let side = extent.d * extent.h;
                let top = extent.w * extent.d;
                front * 2.0 + side * 2.0 + top * 2.0
            }
        }
        
        impl SharpVertices<$t> for Aabb<$t> {
            type Points = [vek::Vec3<$t>; 8];
        
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
    };
}

impl_shape_traits!(f32);
impl_shape_traits!(f64);