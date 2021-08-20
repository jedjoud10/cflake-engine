use crate::engine::core::defaults::components::components::Camera;


// An aabb bound
pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

// Default AABB, just a unit cube with a center at 0,0,0
impl Default for AABB {
    fn default() -> Self {
        Self {
            min: (glam::Vec3::ONE / 2.0) - 1.0,
            max: (glam::Vec3::ONE / 2.0),
        }
    }
}

// Intersection functions
impl AABB {
    // Get a specific corner from this AABB
    pub fn get_corner(&self, corner_index: u8) -> glam::Vec3 {
        match corner_index {            
            0 => glam::vec3(self.min.x, self.min.y, self.min.z), // -X, -Y, -Z            
            1 => glam::vec3(self.max.x, self.min.y, self.min.z), // X, -Y, -Z            
            2 => glam::vec3(self.max.x, self.min.y, self.max.z), // X, -Y, Z            
            3 => glam::vec3(self.min.x, self.min.y, self.max.z), // -X, -Y, Z           
            4 => glam::vec3(self.min.x, self.max.y, self.min.z), // -X, Y, -Z           
            5 => glam::vec3(self.max.x, self.max.y, self.min.z), // X, Y, -Z           
            6 => glam::vec3(self.max.x, self.max.y, self.max.z), // X, Y, Z           
            7 => glam::vec3(self.min.x, self.max.y, self.max.z), // -X, Y, Z

            // Other; not supported
            _ => { glam::Vec3::ZERO }
        }
    }

    // Check if this AABB intersects a sphere (or is inside of it)
    pub fn intersect_sphere(&self, _sphere_center: glam::Vec3, _sphere_radius: f32) -> bool {
        false
    }
    // Check if this AABB intersects another AABB (or is inside of it)
    pub fn intersect_other(&self, _other: Self) -> bool {
        false
    }
    // Check if this AABB intersects the camera's view frustum
    pub fn intersect_camera_view_frustum(&self, camera: &Camera) -> bool {
        // Create the clip space matrix
        let projection_matrix = camera.projection_matrix;
        // Get all the corners from this AABB and transform them by the matrix, then check if they fit inside the NDC
        for corner_index in 0..8 {
            let corner = self.get_corner(corner_index);
            let transformed_corner = projection_matrix.mul_vec4(glam::vec4(corner.x, corner.y, corner.z, 1.0));
            // Check if is inside the bounds of the NDC
            if transformed_corner.abs().cmplt(glam::Vec4::ONE).all() {
                // The AABB is inside the view frustum,.we can exit early
                return true;
            }
        }
        return false;
    }
}
