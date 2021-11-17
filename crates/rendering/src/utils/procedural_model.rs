use crate::basics::Model;

// Procedural model creation for spheres / boxes
pub struct ProceduralModelGenerator {}

impl ProceduralModelGenerator {
    // Generate a sphere with procedural parameters
    pub fn sphere(origin: veclib::Vector3<f32>, radius: f32, vs: i32, hs: i32) -> Model {
        todo!()
    }
    // Generate a cube with procedural parameters
    pub fn cube(origin: veclib::Vector3<f32>, size: veclib::Vector3<f32>) -> Model {
        // Create the procedural cube
        let vertices = vec![
            veclib::Vector3::<f32> { x: 0.0, y: 0.0, z: 0.0 },
            veclib::Vector3::<f32> { x: 0.0, y: 0.0, z: 1.0 },
            veclib::Vector3::<f32> { x: 1.0, y: 0.0, z: 1.0 },
            veclib::Vector3::<f32> { x: 1.0, y: 0.0, z: 0.0 },
            veclib::Vector3::<f32> { x: 0.0, y: 1.0, z: 0.0 },
            veclib::Vector3::<f32> { x: 0.0, y: 1.0, z: 1.0 },
            veclib::Vector3::<f32> { x: 1.0, y: 1.0, z: 1.0 },
            veclib::Vector3::<f32> { x: 1.0, y: 1.0, z: 0.0 },
        ]
        .iter()
        .map(|x| (*x * size) - (size / 2.0))
        .collect::<Vec<veclib::Vector3<f32>>>();

        return Model {
            vertices: vertices.iter().map(|x| *x - origin).collect(),
            normals: vertices.iter().map(|x| x.normalized()).collect(),
            tangents: todo!(),
            uvs: todo!(),
            colors: todo!(),
            triangles: todo!(),
        };
    }
}
