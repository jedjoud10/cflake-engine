use num_traits::FloatConst;
use vek::num_traits::Pow;
use crate::{Boundable, Movable, Shape, SurfaceArea, Volume, Aabb};

// A simple sphere that is represented by it's center and radius
#[derive(Clone, Copy)]
pub struct Sphere<T> {
    // Center of the sphere
    pub center: vek::Vec3<T>,

    // Radius of the sphere
    pub radius: T,
}

impl<T> Sphere<T> {
    // Create a new sphere from a center and radius
    pub fn new(center: vek::Vec3<T>, radius: T) -> Self {
        Self { center, radius }
    }
}

macro_rules! impl_shape_traits {
    ($t:ty) => {       
        impl Movable<$t> for Sphere<$t> {
            fn center(&self) -> vek::Vec3<$t> {
                self.center
            }
        
            fn set_center(&mut self, new: vek::Vec3<$t>) {
                self.center = new;
            }
        }
        
        impl Boundable<$t> for Sphere<$t> {
            fn bounds(&self) -> crate::Aabb<$t> {
                crate::Aabb::<$t> {
                    min: self.center - self.radius,
                    max: self.center + self.radius,
                }
            }
        
            fn scale_by(&mut self, scale: $t) {
                self.radius *= scale;
            }
        
            fn expand_by(&mut self, units: $t) {
                self.radius += units;
            }
        }
        
        impl Volume<$t> for Sphere<$t> {
            fn volume(&self) -> $t {
                (4.0 / 3.0) * <$t>::PI() * self.radius.pow(3.0)
            }
        }
        
        impl SurfaceArea<$t> for Sphere<$t> {
            fn area(&self) -> $t {
                4.0 * <$t>::PI() * self.radius.pow(2.0)
            }
        }
        
        impl Shape<$t> for Sphere<$t> {} 
    }
}

impl_shape_traits!(f32);
impl_shape_traits!(f64);