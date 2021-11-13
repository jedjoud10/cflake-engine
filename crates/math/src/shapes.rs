// The corners of a cube
pub const CUBE_CORNERS: [veclib::Vector3<f32>; 8] = [
    veclib::Vector3::<f32> { x: 0.0, y: 0.0, z: 0.0 },
    veclib::Vector3::<f32> { x: 1.0, y: 0.0, z: 0.0 },
    veclib::Vector3::<f32> { x: 1.0, y: 0.0, z: 1.0 },
    veclib::Vector3::<f32> { x: 0.0, y: 0.0, z: 1.0 },
    veclib::Vector3::<f32> { x: 0.0, y: 1.0, z: 0.0 },
    veclib::Vector3::<f32> { x: 1.0, y: 1.0, z: 0.0 },
    veclib::Vector3::<f32> { x: 1.0, y: 1.0, z: 1.0 },
    veclib::Vector3::<f32> { x: 0.0, y: 1.0, z: 1.0 },
];

pub const CUBE_EDGES: [Line; 12] = [
    // Bottom face
    Line {
        point: CUBE_CORNERS[0],
        point2: CUBE_CORNERS[1],
    },
    Line {
        point: CUBE_CORNERS[1],
        point2: CUBE_CORNERS[2],
    },
    Line {
        point: CUBE_CORNERS[2],
        point2: CUBE_CORNERS[3],
    },
    Line {
        point: CUBE_CORNERS[0],
        point2: CUBE_CORNERS[0],
    },
    // Top face
    Line {
        point: CUBE_CORNERS[4],
        point2: CUBE_CORNERS[5],
    },
    Line {
        point: CUBE_CORNERS[5],
        point2: CUBE_CORNERS[6],
    },
    Line {
        point: CUBE_CORNERS[6],
        point2: CUBE_CORNERS[7],
    },
    Line {
        point: CUBE_CORNERS[7],
        point2: CUBE_CORNERS[4],
    },
    // Side
    Line {
        point: CUBE_CORNERS[0],
        point2: CUBE_CORNERS[4],
    },
    Line {
        point: CUBE_CORNERS[1],
        point2: CUBE_CORNERS[5],
    },
    Line {
        point: CUBE_CORNERS[2],
        point2: CUBE_CORNERS[6],
    },
    Line {
        point: CUBE_CORNERS[3],
        point2: CUBE_CORNERS[7],
    },
];

// An infinite plane
#[derive(Default, Clone, Copy)]
pub struct Plane {
    pub distance: f32,
    pub normal: veclib::Vector3<f32>,
}
// A simple, finite line
#[derive(Default, Clone, Copy)]
pub struct Line {
    pub point: veclib::Vector3<f32>,
    pub point2: veclib::Vector3<f32>,
}
impl Line {
    // Construct a line from it's start position and dir
    pub fn dir_construct(start: veclib::Vector3<f32>, dir: veclib::Vector3<f32>) -> Self {
        Self {
            point: start,
            point2: start + dir,
        }
    }
    // Construct a line from two points
    pub fn construct(start: veclib::Vector3<f32>, end: veclib::Vector3<f32>) -> Self {
        Self { point: start, point2: end }
    }
}
// Shape type
#[derive(Clone, Copy, Debug)]
pub enum ShapeType {
    Cube(veclib::Vector3<f32>),
    Sphere(f32),
    AxisPlane(veclib::Vec3Axis),
}
// A main shape struct
#[derive(Clone, Debug)]
pub struct Shape {
    // The center of the shape
    pub center: veclib::Vector3<f32>,
    pub internal_shape: ShapeType,
}
impl Shape {
    // New cube
    pub fn new_cube(center: veclib::Vector3<f32>, half_extent: veclib::Vector3<f32>) -> Self {
        Self {
            center,
            internal_shape: ShapeType::Cube(half_extent),
        }
    }
    // New sphere
    pub fn new_sphere(center: veclib::Vector3<f32>, radius: f32) -> Self {
        Self {
            center,
            internal_shape: ShapeType::Sphere(radius),
        }
    }
}
