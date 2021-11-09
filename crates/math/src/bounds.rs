use super::shapes;

// An aabb bound
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: veclib::Vector3<f32>,
    pub max: veclib::Vector3<f32>,
    pub center: veclib::Vector3<f32>,
}

// Default AABB, just a unit cube with a center at 0,0,0
impl Default for AABB {
    fn default() -> Self {
        Self {
            min: (veclib::Vector3::ONE / 2.0) - 1.0,
            max: (veclib::Vector3::ONE / 2.0),
            center: veclib::Vector3::ZERO,
        }
    }
}

// NDC
impl AABB {
    pub fn ndc() -> Self {
        Self {
            min: -veclib::Vector3::ONE,
            max: veclib::Vector3::ONE,
            center: veclib::Vector3::ZERO,
        }
    }
    pub fn ndc_forward() -> Self {
        Self {
            min: veclib::Vector3::new(-1.0, -1.0, 0.0),
            max: veclib::Vector3::ONE,
            center: veclib::Vector3::new(0.0, 0.0, 0.5),
        }
    }
}

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
        shapes::CUBE_EDGES[edge_index as usize]
    }
}

// Generation functions
impl AABB {
    // Generate the AABB from a set of points
    pub fn new_vertices(vertices: &Vec<veclib::Vector3<f32>>) -> Self {
        let mut aabb: Self = AABB {
            min: veclib::Vector3::ONE * 9999.0,
            max: -veclib::Vector3::ONE * 9999.0,
            center: veclib::Vector3::ZERO,
        };
        // Loop over the vertices
        for vertex in vertices.iter() {
            aabb.min = aabb.min.min(*vertex);
            aabb.max = aabb.max.max(*vertex);
        }
        aabb.center = (aabb.max + aabb.min) / 2.0;
        aabb
    }   
    // Generate the AABB from a center and some half extents
    pub fn new_center_halfextent(center: veclib::Vector3<f32>, half_extent: veclib::Vector3<f32>) -> Self {
        Self {
            min: center - half_extent,
            max: center + half_extent,
            center,
        }
    } 
}

// Transform functions
impl AABB {
    // Transform the AABB by a transform
    pub fn transform(&mut self, transform_matrix: &veclib::Matrix4x4<f32>) {
        // Transform the min and max by the transform's matrix
        let matrix = transform_matrix;
        self.min = matrix.mul_point(&self.min);
        self.max = matrix.mul_point(&self.max);
        self.center = (self.max + self.min) / 2.0;
    }
    // Get the closest point of the AABB from a specific point
    pub fn get_nearest_point(&self, point: &veclib::Vector3<f32>) -> veclib::Vector3<f32> {
        point.clamp(self.min, self.max)
    }
    // Expand the AABB by a number
    pub fn expand(&mut self, factor: f32) {
        // Expand the AABB
        self.min -= factor;
        self.max += factor;
        self.center = (self.max + self.min) / 2.0;
    }
    // Scale this AABB using a center point and a scaling vector
    pub fn scale(&mut self, center: veclib::Vector3<f32>, scale: veclib::Vector3<f32>) {
        self.min -= center;
        self.max -= center;
        // Scale
        self.min *= scale;
        self.max *= scale;
        // Reset again
        self.min += center;
        self.max += center;
    }
}
