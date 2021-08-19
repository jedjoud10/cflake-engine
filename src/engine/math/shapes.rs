// An infinite plane
#[derive(Default)]
pub struct Plane {
	pub position: glam::Vec3,
	pub normal: glam::Vec3
}

// Intersection tests
impl Plane {
	// Test intersection with a line
	pub fn intersect_line(&self, line: Line) -> bool {
		todo!()
	}
}

// A simple, finite line
#[derive(Default)]
pub struct Line {
	pub point: glam::Vec3,
	pub point2: glam::Vec3,
}