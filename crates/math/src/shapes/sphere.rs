use crate::{AABB, Volume, Area, Shape, Movable, Boundable};

// A simple sphere that is represented by it's center and radius
#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: vek::Vec3<f32>,
    pub radius: f32,
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
        todo!()
    }
}

impl Area for Sphere {
    fn area(&self) -> f32 {
        todo!()
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