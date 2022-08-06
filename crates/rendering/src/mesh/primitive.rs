
use math::{UvSphere, Cuboid, IcoSphere};

use crate::context::Context;

use super::{Mesh, MeshImportSettings};

// A primitive generator that we can use to generate procedural shapes at runtime
pub trait PrimitiveGenerator {
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh;
}

impl PrimitiveGenerator for Cuboid {
    // Generate a cuboid mesh
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh {
        // Buffers we shall useth
        let positions = Vec::with_capacity(24);
        let normals = settings.use_normals.then(|| Vec::with_capacity(24));
        let tangents = settings.use_tangents.then(|| Vec::with_capacity(24));
        let tex_coords = settings.use_tex_coords.then(|| Vec::with_capacity(24));

        // Create the rotation quaternions
        let rotations: [vek::Quaternion<f32>; 6] = [
            vek::Quaternion::identity(), 
            vek::Quaternion::rotation_x(90.0f32.to_radians()),
            vek::Quaternion::rotation_x(180.0f32.to_radians()),
            vek::Quaternion::rotation_x(270.0f32.to_radians()),
            vek::Quaternion::rotation_z(90.0f32.to_radians()),
            vek::Quaternion::rotation_z(-90.0f32.to_radians())
        ];

        // Create the 6 faces separartely
        for f in 0..6 {
            let quat = rotations[f];
            let matrix = vek::Mat3::from(quat);

            // Generate the positions
            let p0 = matrix * vek::Vec3::new(-0.5, -0.5, 0.0);
            let p1 = matrix * vek::Vec3::new(-0.5, 0.5, 0.0);
            let p2 = matrix * vek::Vec3::new(0.5, -0.5, 0.0);
            let p3 = matrix * vek::Vec3::new(0.5, 0.5, 0.0);

            // Generate the normals (optional)
            let normal = settings.use_normals.then(|| matrix * vek::Vec3::new(0.0, 0.5, 0.0));
            
            // Generate the tangents (optional)
            
            // Generate the texture coordinates (optional)

        }
        
        // Generate the indices
        let indices = (0..24).into_iter().collect::<Vec<u32>>();

        todo!()
    }
}

impl PrimitiveGenerator for UvSphere {
    // Generate a UV sphere mesh
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh {
        todo!()
    }
}

impl PrimitiveGenerator for IcoSphere {
    // Generate an IcoSphere mesh
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh {
        todo!()
    }
}