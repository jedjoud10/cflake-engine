// A simple entity in the world
pub struct Entity {
	pub name: String,
	pub components_id: u8;
}

// Default
impl Default for Entity {
	fn default() -> Self {
		Self {
			name: "Unnamed Entity",
			components_id: 0,
		}
	}
}