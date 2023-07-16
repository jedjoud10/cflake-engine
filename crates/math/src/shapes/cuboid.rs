use crate::{Boundable, ExplicitVertices, Movable, SurfaceArea, Volume};

// A 3D cuboid that is defined by it's center and it's extent
#[derive(Clone, Copy)]
pub struct Cuboid<T> {
    // Center of the cuboid
    pub center: vek::Vec3<T>,

    // Half extent of the cubeoid
    pub half_extent: vek::Extent3<T>,
}

impl<T> Cuboid<T> {
    // Create a cuboid from a center and and half-extent
    pub fn new(center: vek::Vec3<T>, half_extent: vek::Extent3<T>) -> Self {
        Self {
            center,
            half_extent,
        }
    }

    // Create a cube from a center and half-extent
    pub fn cube(center: vek::Vec3<T>, half_extent: T) -> Self
    where
        T: Copy,
    {
        Self {
            center,
            half_extent: vek::Extent3::broadcast(half_extent),
        }
    }
}

macro_rules! impl_shape_traits {
    ($t:ty) => {
        impl From<crate::Aabb<$t>> for Cuboid<$t> {
            fn from(aabb: crate::Aabb<$t>) -> Self {
                Self {
                    center: (aabb.max + aabb.min) / 2.0,
                    half_extent: vek::Extent3::from(aabb.max - aabb.min) / 2.0,
                }
            }
        }

        impl Movable<$t> for Cuboid<$t> {
            fn center(&self) -> vek::Vec3<$t> {
                self.center
            }

            fn set_center(&mut self, new: vek::Vec3<$t>) {
                self.center = new
            }
        }

        impl Boundable<$t> for Cuboid<$t> {
            fn bounds(&self) -> crate::Aabb<$t> {
                crate::Aabb::<$t> {
                    min: self.center - self.half_extent,
                    max: self.center + self.half_extent,
                }
            }

            fn scale_by(&mut self, scale: $t) {
                self.half_extent *= scale;
            }

            fn expand_by(&mut self, expand_units: $t) {
                self.half_extent += vek::Extent3::broadcast(expand_units / 2.0);
            }
        }

        impl Volume<$t> for Cuboid<$t> {
            fn volume(&self) -> $t {
                (self.half_extent * 2.0).product()
            }
        }

        impl SurfaceArea<$t> for Cuboid<$t> {
            fn area(&self) -> $t {
                let extent = self.half_extent * 2.0;
                let front = extent.w * extent.h;
                let side = extent.d * extent.h;
                let top = extent.w * extent.d;
                front * 2.0 + side * 2.0 + top * 2.0
            }
        }

        impl ExplicitVertices<$t> for Cuboid<$t> {
            type Points = [vek::Vec3<$t>; 8];

            // http://paulbourke.net/geometry/polygonise/
            fn points(&self) -> Self::Points {
                let max = self.center + vek::Vec3::<$t>::from(self.half_extent);
                let min = self.center - vek::Vec3::<$t>::from(self.half_extent);

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
    };
}

impl_shape_traits!(f32);
impl_shape_traits!(f64);
