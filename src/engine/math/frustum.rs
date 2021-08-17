use super::shapes::Plane;

// A frustum
pub struct Frustum {
	pub clip_planes: Vec<Plane>,
}

// Le kode
impl Frustum {
	// Test the intersection of a specific clip plane with an AABB
}