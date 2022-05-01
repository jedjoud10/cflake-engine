// A singular triangle that is made up of 3 indices
pub struct Triangle {
    indices: [u32; 3]
}

impl From<(u32, u32, u32)> for Triangle {
    fn from(t: (u32, u32, u32)) -> Self {
        Self {
            indices: [t.0, t.1, t.2]
        }
    }
} 

impl From<[u32; 3]> for Triangle {
    fn from(arr: [u32; 3]) -> Self {
        Self { indices: arr }
    }
}

// Very simple, no need to overcomplicate it
pub type TriangleSet = Vec<Triangle>;