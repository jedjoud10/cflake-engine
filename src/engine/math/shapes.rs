use super::aabb::AABB;
// An infinite plane
#[derive(Default)]
pub struct Plane {
	pub distance: f32,
	pub normal: glam::Vec3
}

// Intersection tests
impl Plane {
	// Test intersection with an AABB
	// https://gdbooks.gitbooks.io/3dcollisions/content/Chapter2/static_aabb_plane.html
	pub fn interect_aabb(&self, aabb: AABB) -> bool {
		// Convert AABB to center-extents representation
		let c = (aabb.max + aabb.min) * 0.5; // Compute AABB center
		let e = aabb.max - c; // Compute positive extents
	
		// Compute the projection interval radius of b onto L(t) = b.c + t * p.n
		let r = (self.normal.abs() * e);
		let r = r.x + r.y + r.z;
	
		// Compute distance of box center from plane
		let s = self.normal.dot(c) - self.distance;
	
		// Intersection occurs when distance s falls within [-r,+r] interval
		return s.abs() <= r;		
	}
}

// A simple, finite line
#[derive(Default)]
pub struct Line {
	pub point: glam::Vec3,
	pub point2: glam::Vec3,
}