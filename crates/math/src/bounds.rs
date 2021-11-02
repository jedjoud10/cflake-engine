use super::shapes;

// An aabb bound
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: veclib::Vector3<f32>,
    pub max: veclib::Vector3<f32>,
}

// Default AABB, just a unit cube with a center at 0,0,0
impl Default for AABB {
    fn default() -> Self {
        Self {
            min: (veclib::Vector3::ONE / 2.0) - 1.0,
            max: (veclib::Vector3::ONE / 2.0),
        }
    }
}

// Intersection functions
impl AABB {
    // Get a specific corner from this AABB
    pub fn get_corner(&self, corner_index: u8) -> veclib::Vector3<f32> {
        match corner_index {
            0 => veclib::Vector3::new(self.min.x, self.min.y, self.min.z), // -X, -Y, -Z
            1 => veclib::Vector3::new(self.max.x, self.min.y, self.min.z), // X, -Y, -Z
            2 => veclib::Vector3::new(self.max.x, self.min.y, self.max.z), // X, -Y, Z
            3 => veclib::Vector3::new(self.min.x, self.min.y, self.max.z), // -X, -Y, Z
            4 => veclib::Vector3::new(self.min.x, self.max.y, self.min.z), // -X, Y, -Z
            5 => veclib::Vector3::new(self.max.x, self.max.y, self.min.z), // X, Y, -Z
            6 => veclib::Vector3::new(self.max.x, self.max.y, self.max.z), // X, Y, Z
            7 => veclib::Vector3::new(self.min.x, self.max.y, self.max.z), // -X, Y, Z

            // Other; not supported
            _ => veclib::Vector3::ZERO,
        }
    }
    // Get a specific edge from this AABB
    pub fn get_edge(&self, edge_index: u8) -> shapes::Line {
        return shapes::CUBE_EDGES[edge_index as usize];
    }
}

// Generation functions
impl AABB {
    // Generate the AABB from a model; just loop over all the vertices and keep track of the min and max ones
    pub fn from_model(vertices: &Vec<veclib::Vector3<f32>>) -> Self {
        let mut aabb: Self = AABB {
            min: veclib::Vector3::ONE,
            max: -veclib::Vector3::ONE,
        };
        // Loop over the vertices
        for vertex in vertices.iter() {
            aabb.min = aabb.min.min(*vertex);
            aabb.max = aabb.max.max(*vertex);
        }
        aabb
    }
    // Transform the AABB by a transform
    pub fn transform(&mut self, transform_matrix: &veclib::Matrix4x4<f32>) {
        // Transform the min and max by the transform's matrix
        let matrix = transform_matrix;
        self.min = matrix.mul_point(&self.min);
        self.max = matrix.mul_point(&self.max);
    }
    // Get the closest point of the AABB from a specific point
    pub fn get_nearest_point(&self, point: &veclib::Vector3<f32>) -> veclib::Vector3<f32> {
        return point.clamp(self.min, self.max);
    }
    // Expand the AABB by a number
    pub fn expand(&mut self, factor: f32) {
        // Expand the AABB
        self.min -= factor;
        self.max += factor;
    }
}
