use std::f32::consts::PI;

use vek::num_traits::Pow;

use crate::{Boundable, Movable, Shape, SurfaceArea, Volume, AABB};

// A simple sphere that is represented by it's center and radius
#[derive(Clone, Copy)]
pub struct Sphere {
    // Center of the sphere
    pub center: vek::Vec3<f32>,
    
    // Radius of the sphere
    pub radius: f32,
}

impl Sphere {
    // Create a new sphere from a center and radius
    pub fn new(center: vek::Vec3<f32>, radius: f32) -> Self {
        Self {
            center,
            radius,
        }
    }
}

impl Movable for Sphere {
    fn center(&self) -> vek::Vec3<f32> {
        self.center
    }

    fn set_center(&mut self, new: vek::Vec3<f32>) {
        self.center = new;
    }
}

impl Boundable for Sphere {
    fn bounds(&self) -> AABB {
        AABB {
            min: self.center - self.radius,
            max: self.center + self.radius,
        }
    }

    fn scale_by(&mut self, scale: f32) {
        self.radius *= scale;
    }

    fn expand_by(&mut self, expand_units: f32) {
        self.radius += expand_units;
    }
}

impl Volume for Sphere {
    fn volume(&self) -> f32 {
        4.0 / 3.0 * PI * self.radius.pow(3.0)
    }
}

impl SurfaceArea for Sphere {
    fn surface_area(&self) -> f32 {
        4.0 * PI * self.radius.pow(2.0)
    }
}

impl Shape for Sphere {}

// TODO: Should this be stored within the rendering crate or math crate??
// A mesh UV sphere that has vertical and horizontal subdivisions
#[derive(Clone, Copy)]
pub struct UvSphere {
    pub center: vek::Vec3<f32>,
    pub radius: f32,
    pub horizontal_subdivions: u32,
    pub vertical_subdivisions: u32,
}

// A mesh ICO sphere that is built using multiple triangles
#[derive(Clone, Copy)]
pub struct IcoSphere {
    pub center: vek::Vec3<f32>,
    pub radius: f32,
    pub subdivisions: u32,
}
