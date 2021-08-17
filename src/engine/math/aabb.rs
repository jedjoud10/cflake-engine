use super::frustum::Frustum;

// An aabb bound
pub struct AABB {
	pub min: glam::Vec3,
	pub max: glam::Vec3
}

// Intersection functions
impl AABB {
	// Check if this AABB intersects a sphere (or is inside of it)
	pub fn intersect_sphere(&self, sphere_center: glam::Vec3, sphere_radius: f32) -> bool {
		false
	}
	// Check if this AABB intersects another AABB (or is inside of it)
	pub fn intersect_other(&self, other: Self) -> bool {
		false
	}
	// Check if this AABB intersects a frustum (Like the camera's)
	pub fn intersect_frustum(&self, frustum: Frustum) -> bool {
		false
	}
}