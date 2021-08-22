use glam::Vec4Swizzles;

use crate::engine::{core::defaults::components::components::Camera, rendering::model::Model};


// An aabb bound
#[derive(Clone, Copy)]
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
    pub fn intersect_camera_view_frustum(&self, camera_frustum_matrix: &glam::Mat4) -> bool {
        // Create the clip space matrix
        let matrix = camera_frustum_matrix;
        // An multiplication factor just to debug the frustum culling
        const factor: f32 = 1.3;
        // Get all the corners from this AABB and transform them by the matrix, then check if they fit inside the NDC
        for corner_index in 0..8 {
            let corner = self.get_corner(corner_index);
            let transformed_corner = matrix.mul_vec4(glam::vec4(corner.x, corner.y, corner.z, 1.0));
            // You have to divide by the W scalar first to get the screenspace NDC
            let transformed_corner_screen_space = transformed_corner.xy() / transformed_corner.w;
            // Only the objects in front of us
            if transformed_corner.z > 0.0 {
                // Check if is inside the bounds of the 2D screenspace NDC
                let min = (transformed_corner_screen_space * factor).cmplt(glam::Vec2::ONE).all();
                let max = (transformed_corner_screen_space * factor).cmpgt(-glam::Vec2::ONE).all();
                if min && max {
                    // The AABB is inside the view frustum,.we can exit early
                    return true;
                }
            } else {
                // The projected corner was behind us
                continue;
            }
        }
        return false;
    }
}

// Generation functions
impl AABB {
    // Generate the AABB from a model; just loop over all the vertices and keep track of the min and max ones
    pub fn from_model(model: &Model) -> Self {
        let mut aabb: Self = AABB::default();
        // Loop over the vertices
        for vertex in model.vertices.iter() {
            aabb.min = aabb.min.min(*vertex);
            aabb.max = aabb.max.max(*vertex);
        }
        return aabb;
    }
    // Offset the AABB using a position
    pub fn offset(&mut self, position: glam::Vec3) {
        // Offset the AABB by offsetting the min and max
        self.min += position;
        self.max += position;
    }
    // Scale the AABB using a scalar value
    pub fn scale(&mut self, scale: f32) {
        // Scale the AABB by scaling the min and max
        self.min *= scale;
        self.max *= scale;
    }
}