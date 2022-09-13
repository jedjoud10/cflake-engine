use itertools::Itertools;
use math::{Cuboid, IcoSphere, UvSphere};

use crate::context::Context;

use super::{Mesh, MeshImportSettings, MeshUtils};

// A primitive generator that we can use to generate procedural shapes at runtime
// TODO: Finish writing this bozo
pub trait PrimitiveGenerator {
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh;
}

impl PrimitiveGenerator for Cuboid {
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh {
        let mut positions = Vec::<vek::Vec3<f32>>::with_capacity(24);
        let mut normals = settings
            .use_normals
            .then(|| Vec::<vek::Vec3<i8>>::with_capacity(24));
        let mut tex_coords = settings
            .use_tex_coords
            .then(|| Vec::<vek::Vec2<u8>>::with_capacity(24));

        // Create the rotation quaternions
        let rotations: [vek::Quaternion<f32>; 6] = [
            vek::Quaternion::identity(),
            vek::Quaternion::rotation_y(90.0f32.to_radians()),
            vek::Quaternion::rotation_y(180.0f32.to_radians()),
            vek::Quaternion::rotation_y(270.0f32.to_radians()),
            vek::Quaternion::rotation_x(90.0f32.to_radians()),
            vek::Quaternion::rotation_x(-90.0f32.to_radians()),
        ];

        // Create the 6 faces separartely
        for rot in rotations {
            let matrix = vek::Mat3::from(rot);

            // Generate the positions
            let local_positions = [
                MeshUtils::mul_position(matrix.into(), vek::Vec3::new(-0.5, -0.5, 0.5)),
                MeshUtils::mul_position(matrix.into(), vek::Vec3::new(-0.5, 0.5, 0.5)),
                MeshUtils::mul_position(matrix.into(), vek::Vec3::new(0.5, -0.5, 0.5)),
                MeshUtils::mul_position(matrix.into(), vek::Vec3::new(0.5, 0.5, 0.5)),
            ];

            // Generate the normals (optional)
            let local_normal = settings.use_normals.then(|| {
                MeshUtils::mul_normal(
                    matrix.into(),
                    vek::Vec3::new(0, 0, 127),
                    settings.invert_normals,
                )
            });

            // Generate the texture coordinates (optional)
            let mut local_tex_coords = settings.use_tex_coords.then(|| {
                [
                    vek::Vec2::new(0, 0),
                    vek::Vec2::new(0, 255),
                    vek::Vec2::new(255, 0),
                    vek::Vec2::new(255, 255),
                ]
            });

            // Add the positions into the vector
            positions.extend_from_slice(&local_positions);

            // Add the normals into the vector
            if let Some(normals) = &mut normals {
                normals.extend(std::iter::repeat(local_normal.unwrap()).take(4));
            }

            // Add the texture coordinates into the vector
            if let Some(tex_coords) = &mut tex_coords {
                MeshUtils::apply_settings_tex_coords(
                    local_tex_coords.as_mut().unwrap(),
                    settings.invert_horizontal_tex_coord,
                    settings.invert_vertical_tex_coord,
                );
                tex_coords.extend(local_tex_coords.unwrap());
            }
        }

        // Generate the indices
        let triangles = (0..6)
            .into_iter()
            .map(|face| {
                let offset = face * 4;
                let tri1 = [offset, 1 + offset, 2 + offset];
                let tri2 = [2 + offset, 1 + offset, 3 + offset];
                (tri1, tri2)
            })
            .flat_map(|(t1, t2)| [t1, t2])
            .collect_vec();

        // Generate the tangents if we want
        let tangents = if let (Some(normals), Some(tex_coords), true) = (&normals, &tex_coords, settings.use_tangents) {
            if let Some(mut tangents) = MeshUtils::compute_tangents(&positions, normals, tex_coords, &triangles) {
                if settings.invert_tangents {
                    for tangent in tangents.iter_mut() {
                        *tangent = MeshUtils::mul_tangent(vek::Mat4::identity(), *tangent, true)
                    }
                }

                Some(tangents)
            } else { None }
        } else { None };

        Mesh::from_vecs(
            ctx,
            settings.mode,
            positions,
            normals,
            tangents,
            None,
            tex_coords,
            triangles,
        )
        .unwrap()
    }
}

impl PrimitiveGenerator for UvSphere {
    // Generate a UV sphere mesh
    fn generate(self, _ctx: &mut Context, _settings: MeshImportSettings) -> Mesh {
        todo!()
    }
}

impl PrimitiveGenerator for IcoSphere {
    // Generate an IcoSphere mesh
    fn generate(self, _ctx: &mut Context, _settings: MeshImportSettings) -> Mesh {
        todo!()
    }
}
