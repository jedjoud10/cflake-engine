// The corners of a cube
pub const CUBE_CORNERS: [glam::Vec3; 8] = [
    glam::const_vec3!([0.0, 0.0, 0.0]),
    glam::const_vec3!([1.0, 0.0, 0.0]),
    glam::const_vec3!([1.0, 0.0, 1.0]),
    glam::const_vec3!([0.0, 0.0, 1.0]),
    glam::const_vec3!([0.0, 1.0, 0.0]),
    glam::const_vec3!([1.0, 1.0, 0.0]),
    glam::const_vec3!([1.0, 1.0, 1.0]),
    glam::const_vec3!([0.0, 1.0, 1.0]),
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
    pub normal: glam::Vec3,
}
// A simple, finite line
#[derive(Default, Clone, Copy)]
pub struct Line {
    pub point: glam::Vec3,
    pub point2: glam::Vec3,
}
impl Line {
    // Construct a line from it's start position and dir
    pub fn dir_construct(start: glam::Vec3, dir: glam::Vec3) -> Self {
        Self {
            point: start,
            point2: start + dir,
        }
    }
    // Construct a line from two points
    pub fn construct(start: glam::Vec3, end: glam::Vec3) -> Self {
        Self { point: start, point2: end }
    }
}
// A simple cube
#[derive(Default, Clone, Copy)]
pub struct Cube {
    pub center: glam::Vec3,
    pub size: glam::Vec3,
}
// A simple sphere
#[derive(Default, Clone, Copy)]
pub struct Sphere {
    pub center: glam::Vec3,
    pub radius: f32,
}
// A simple square
#[derive(Default, Clone, Copy, Debug)]
pub struct Square {
    pub min: glam::Vec2,
    pub max: glam::Vec2,
}
