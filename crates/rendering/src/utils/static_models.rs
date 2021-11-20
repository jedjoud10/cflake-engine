use crate::Model;
use veclib::consts::*;
// Some static models

// Static quad model used for the screen rendering
pub const QUAD: Model = Model {
    vertices: vec![
        vec3(1.0, -1.0, 0.0),
        vec3(-1.0, 1.0, 0.0),
        vec3(-1.0, -1.0, 0.0),
        vec3(1.0, 1.0, 0.0)
    ],
    normals: vec![veclib::Vector3::ZERO; 4],
    tangents: vec![veclib::Vector4::ZERO; 4],
    uvs: vec![
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 1.0)
    ],
    colors: vec![veclib::Vector3::ZERO; 4],
    triangles: vec![0, 1, 2, 0, 3, 1],
};