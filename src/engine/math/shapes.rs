// A simple plane consisting of 4 vertices
#[derive(Default)]
pub struct Plane {
	pub x: glam::Vec3,
	pub y: glam::Vec3,
	pub x1: glam::Vec3,
	pub y1: glam::Vec3,
}

// Intersection tests
impl Plane {
	// Test intersection with another plane
	pub fn intersect_other(&self, _other: Self) -> bool {
		todo!()
	}
}