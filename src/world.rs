// The actual world
pub struct World {
	pub time_manager: Time,
}
impl World {
	pub fn start_world(&mut self) {
		
	}
	pub fn update_world(&mut self) {
		
	}
	pub fn stop_world(&mut self) {
		
	}
}

// Default values for world
impl Default for World {
	fn default() -> Self {
		Self {
			// Setup the time manager
			time_manager: Time {
				time_since_start: 0.0,
				delta_time: 0.0,
			},
		}
	}
}


// Static time variables
pub struct Time {
	pub time_since_start: f64,
	pub delta_time: f64,
}