use std::ops::Neg;

use math::AABB;

use crate::buffer::Triangle;

use super::{MeshImportSettings, VeNormal, VePosition, VeTangent, VeTexCoord};

// Mesh utils are simple helper functions that facilitate the construction of a mesh
pub struct MeshUtils;

// Invert the given number if the boolean is true
fn inv<T: Neg<Output = T>>(num: T, flip: bool) -> T {
    if flip {
        -num
    } else {
        num
    }
}

// Convert i8 value to f32 (0 - 1 range)
fn to_f32(val: i8) -> f32 {
    val as f32 / 127.0
}

// Convert f32 value to i8 (-127 - 128 range)
fn to_i8(val: f32) -> i8 {
    (val * 127.0) as i8
}

impl MeshUtils {
    // Apply settings for mesh vectors (including triangles)
    pub fn apply_vec_settings(
        settings: MeshImportSettings,
        matrix: vek::Mat4<f32>,
        positions: &mut Vec<VePosition>,
        normals: &mut Option<Vec<VeNormal>>,
        tangents: &mut Option<Vec<VeTangent>>,
        tex_coords: &mut Option<Vec<VeTexCoord>>,
        triangles: &mut Vec<Triangle<u32>>,
    ) {
        MeshUtils::apply_settings_positions(positions, matrix);
        if let Some(normals) = normals {
            MeshUtils::apply_settings_normals(normals, matrix, settings.invert_normals);
        }
        if let Some(tangents) = tangents {
            MeshUtils::apply_settings_tangents(tangents, matrix, settings.invert_tangents);
        }
        if let Some(tex_coords) = tex_coords {
            MeshUtils::apply_settings_tex_coords(
                tex_coords,
                settings.invert_horizontal_tex_coord,
                settings.invert_vertical_tex_coord,
            );
        }
        if settings.invert_triangle_ordering {
            MeshUtils::invert_triangle_ordering(triangles);
        }
    }

    // Multiply a position by a matrix
    pub fn mul_position(matrix: vek::Mat4<f32>, position: VePosition) -> VePosition {
        matrix.mul_point(position)
    }

    // Multiply a normal by a matrix
    pub fn mul_normal(matrix: vek::Mat4<f32>, normal: VeNormal, flip: bool) -> VeNormal {
        let mapped = normal.map(to_f32);
        let new = inv(matrix.mul_direction(mapped), flip);
        new.map(to_i8)
    }

    // Multiply a tangent by a matrix
    pub fn mul_tangent(matrix: vek::Mat4<f32>, tangent: VeTangent, flip: bool) -> VeTangent {
        let mapped = tangent.map(|f| f as f32 / 127.0);
        let new = matrix.mul_direction(inv(mapped.xyz(), flip)).map(to_i8);

        vek::Vec4::new(new.x, new.y, new.z, to_i8(mapped.w))
    }

    // Update a texture coordinate by it's settings
    pub fn update_tex_coord(
        mut tex_coord: VeTexCoord,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) -> VeTexCoord {
        if flip_horizontal {
            tex_coord.x = 255 - tex_coord.x;
        }

        if flip_vertical {
            tex_coord.y = 255 - tex_coord.y;
        }

        tex_coord
    }

    // Update a set of position attributes using a matrix
    pub fn apply_settings_positions(positions: &mut [VePosition], matrix: vek::Mat4<f32>) {
        for position in positions {
            *position = Self::mul_position(matrix, *position);
        }
    }

    // Update a set of normal attributes using a matrix and a flip rule
    pub fn apply_settings_normals(normals: &mut [VeNormal], matrix: vek::Mat4<f32>, flip: bool) {
        for normal in normals {
            *normal = Self::mul_normal(matrix, *normal, flip);
        }
    }

    // Update a set of tangent attributes using a matrix and a flip rule
    pub fn apply_settings_tangents(tangents: &mut [VeTangent], matrix: vek::Mat4<f32>, flip: bool) {
        for tangent in tangents {
            *tangent = Self::mul_tangent(matrix, *tangent, flip);
        }
    }

    // Update a set of texture coordinate attributes using a flip horizontal/vertical rule
    pub fn apply_settings_tex_coords(
        tex_coords: &mut [VeTexCoord],
        flip_horizontal: bool,
        flip_vertical: bool,
    ) {
        for tex_coord in tex_coords {
            *tex_coord = Self::update_tex_coord(*tex_coord, flip_horizontal, flip_vertical);
        }
    }

    // Invert the triangle ordering of a mutable slice
    pub fn invert_triangle_ordering(triangles: &mut [[u32; 3]]) {
        for triangle in triangles {
            triangle.swap(0, 2);
        }
    }

    // Calculate the vertex normals procedurally and return them as a vector
    pub fn compute_normals(
        positions: &[VePosition],
        triangles: &[[u32; 3]],
    ) -> Option<Vec<VeNormal>> {
        let mut normals = vec![vek::Vec3::<f32>::zero(); positions.len()];
        for i in 0..(triangles.len() / 3) {
            // Get triangle indices
            let tri = triangles[i];
            let i1 = tri[0] as usize;
            let i2 = tri[1] as usize;
            let i3 = tri[2] as usize;

            // Get the three vertices that make up the triangle
            let a = positions[i1];
            let b = positions[i2];
            let c = positions[i3];

            // Create the cross product to find the normal face
            let d1 = b - a;
            let d2 = c - a;
            let cross = vek::Vec3::<f32>::cross(d1, d2).normalized();

            // Add the face normal to each vertex sum
            normals[i1] += cross;
            normals[i2] += cross;
            normals[i3] += cross;
        }

        // Normalized + conversion to i8
        Some(
            normals
                .into_iter()
                .map(|n| n.normalized().map(to_i8))
                .collect::<Vec<VeNormal>>(),
        )
    }

    // Calculate the vertex tangents procedurally and return them as a vector
    pub fn compute_tangents(
        positions: &[VePosition],
        normals: &[VeNormal],
        tex_coords: &[VeTexCoord],
        triangles: &[[u32; 3]],
    ) -> Option<Vec<VeTangent>> {
        // Check for incompatible lengths
        let len = positions.len();
        if len != normals.len() || len != tex_coords.len() {
            return None;
        }

        // Local struct that will implement the Geometry trait from the tangent generation lib
        struct TangentGenerator<'a> {
            positions: &'a [VePosition],
            triangles: &'a [[u32; 3]],
            normals: &'a [VeNormal],
            tex_coords: &'a [VeTexCoord],
            tangents: &'a mut [VeTangent],
        }

        impl<'a> mikktspace::Geometry for TangentGenerator<'a> {
            fn num_faces(&self) -> usize {
                self.triangles.len()
            }

            fn num_vertices_of_face(&self, _face: usize) -> usize {
                3
            }

            fn position(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.triangles[face][vert] as usize;
                self.positions[i].into_array()
            }

            fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.triangles[face][vert] as usize;
                self.normals[i].map(to_f32).into_array()
            }

            fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
                let i = self.triangles[face][vert] as usize;
                self.tex_coords[i].map(|x| x as f32 / 255.0).into_array()
            }

            fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
                let i = self.triangles[face][vert] as usize;
                self.tangents[i] = vek::Vec4::<f32>::from_slice(&tangent).map(to_i8);
            }
        }

        let mut tangents = vec![vek::Vec4::<i8>::zero(); positions.len()];
        let mut gen = TangentGenerator {
            positions,
            normals,
            triangles,
            tex_coords,
            tangents: &mut tangents,
        };

        // Generate the procedural tangents and store them
        mikktspace::generate_tangents(&mut gen).then_some(())?;
        Some(tangents)
    }

    // Create a new AABB from a list of vertices in 3D space
    pub fn aabb_from_points(points: &[vek::Vec3<f32>]) -> Option<AABB> {
        if points.len() < 2 {
            return None;
        }

        // Initial values set to their inverse (since we have multiple iterations)
        let mut min = vek::Vec3::broadcast(f32::MAX);
        let mut max = vek::Vec3::broadcast(f32::MIN);

        for point in points {
            // Update the "max" bound element wise
            for (point_element, max_element) in
                point.as_slice().iter().zip(max.as_mut_slice().iter_mut())
            {
                *max_element = f32::max(*max_element, *point_element)
            }

            // Update the "min" bound element wise
            for (point_element, min_element) in
                point.as_slice().iter().zip(min.as_mut_slice().iter_mut())
            {
                *min_element = f32::min(*min_element, *point_element)
            }
        }

        // Check if the AABB would be valid
        (min != max).then_some(AABB { min, max })
    }
}
