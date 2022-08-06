// Mesh utils are simple helper functions that facilitate the construction of a mesh
pub struct MeshUtils;

impl MeshUtils {
    // Update a set of position attributes using a matrix
    pub fn apply_settings_positions(slice: &mut [vek::Vec3<f32>], matrix: vek::Mat4<f32>) {
        todo!()    
    }
    
    // Update a set of normal attributes using a matrix and a flip rule
    pub fn apply_settings_normals(slice: &mut [vek::Vec3<i8>], matrix: vek::Mat4<f32>, flip: bool) {
        todo!()
    }
    
    // Update a set of tangent attributes using a matrix and a flip rule
    pub fn apply_settings_tangents(slice: &mut [vek::Vec4<i8>], matrix: vek::Mat4<f32>, flip: bool) {
        todo!()
    }

    // Update a set of texture coordinate attributes using a flip horizontal/vertical rule
    pub fn apply_settings_tex_coords(slice: &mut [vek::Vec2<u8>], flip_horizontal: bool, flip_vertical: bool) {
        for uv in slice {
            if flip_horizontal {
                uv.x = 255 - uv.x;
            }

            if flip_vertical {
                uv.y = 255 - uv.y;
            }
        }
    }

    // Calculate the vertex normals procedurally and return them as a vector
    pub fn compute_normals(positions: &[vek::Vec3<f32>], triangles: &[[u32]; 3]) -> Option<Vec<vek::Vec3<f32>>> {
        todo!()
        // Create pre-allocated normal buffer
        let mut normals = vec![vek::Vec3::<f32>::zero(); positions.len()];

        // Normal calculations
        for i in 0..(triangles.len() / 3) {
            let tri = triangles[i];
            let i1 = tri[0] as usize;
            let i2 = tri[1] as usize;
            let i3 = tri[2] as usize;

            let a = positions[i1];
            let b = positions[i2];
            let c = positions[i3];

            let d1 = b - a;
            let d2 = c - a;
            let cross = vek::Vec3::<f32>::cross(d1, d2).normalized();

            normals[i1] += cross;
            normals[i2] += cross;
            normals[i3] += cross;
        }

        // Normalized + conversion to i8
        let normals: Vec<vek::Vec3<i8>> = normals
            .into_iter()
            .map(|n|
                n.normalized()
                .map(|e| (e * 127.0) as i8)
            ).collect::<_>();
    }

    // Calculate the vertex tangents procedurally and return them as a vector
    pub fn compute_tangents(positions: &[vek::Vec3<f32>], normals: &[vek::Vec3<i8>], tex_coords: &[vek::Vec2<u8>], triangles: &[[u32; 3]]) -> Option<Vec<vek::Vec4<i8>>> {
        // Check for incompatible lengths
        let len = positions.len();
        if len != normals.len() || len != tex_coords.len() {
            return None;
        }

        // Local struct that will implement the Geometry trait from the tangent generation lib
        struct TangentGenerator<'a> {
            positions: &'a [vek::Vec3<f32>],
            triangles: &'a [[u32; 3]],
            normals: &'a [vek::Vec3<i8>],
            tex_coords: &'a [vek::Vec2<u8>],
            tangents: &'a mut [vek::Vec4<i8>],
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
                self.normals[i].map(|x| x as f32 / 127.0).into_array()
            }

            fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
                let i = self.triangles[face][vert] as usize;
                self.tex_coords[i].map(|x| x as f32 / 255.0).into_array()
            }

            fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
                let i = self.triangles[face][vert] as usize;
                self.tangents[i] =
                    vek::Vec4::<f32>::from_slice(&tangent).map(|x| (x * 127.0) as i8);
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
}