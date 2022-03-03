use std::{ffi::c_void, mem::size_of, ptr::null};
use crate::{object::Builder, pipeline::MeshKey};

use super::Mesh;

// Mesh builder
pub struct MeshBuilder {
    inner: Mesh
}

impl MeshBuilder {
    // Create a new mesh
    pub fn new(mesh: Mesh) -> Self {
        Self { inner: mesh }
    }
    // Combine a mesh with our internal one, and return the new mesh builder
    pub fn with_combined_mesh(mut self, other: Self) -> Self {
        let mut mesh = self.inner;
        let other = other.inner;
        let max_triangle_index: u32 = mesh.vertices.positions.len() as u32;
        mesh.indices.extend(other.indices.into_iter().map(|mut x| {
            x += max_triangle_index;
            x
        }));
        mesh.vertices.positions.extend(other.vertices.positions.into_iter());
        mesh.vertices.normals.extend(other.vertices.normals.into_iter());
        mesh.vertices.uvs.extend(other.vertices.uvs.into_iter());
        mesh.vertices.colors.extend(other.vertices.colors.into_iter());
        mesh.vertices.tangents.extend(other.vertices.tangents.into_iter());
        self
    }
    // Procedurally generate the normals for this mesh
    pub fn with_generated_normals(mut self) {
        // First, loop through every triangle and calculate it's face normal
        // Then loop through every vertex and average out the face normals of the adjacent triangles
        let mut mesh = self.inner;
        let mut vertex_normals: Vec<veclib::Vector3<f32>> = vec![veclib::Vector3::ZERO; mesh.vertices.positions.len()];
        for i in 0..(self.indices.len() / 3) {
            // Calculate the face normal
            let (i1, i2, i3) = (mesh.indices[i * 3], mesh.indices[i * 3 + 1], mesh.indices[i * 3 + 2]);
            // Get the actual vertices
            let a = mesh.vertices.positions.get(i1 as usize).unwrap();
            let b = mesh.vertices.positions.get(i2 as usize).unwrap();
            let c = mesh.vertices.positions.get(i3 as usize).unwrap();

            // Calculate
            let d1 = b - a;
            let d2 = c - a;
            let cross = veclib::Vector3::<f32>::cross(d1, d2).normalized();

            // Add the face normal to our local vertices
            vertex_normals[i1 as usize] += cross;
            vertex_normals[i2 as usize] += cross;
            vertex_normals[i3 as usize] += cross;
        }

        // Now we must normalize
        for vertex_normal in vertex_normals.iter_mut() {
            vertex_normal.normalize();
        }

        // Update our normals
        mesh.vertices.normals = vertex_normals.into_iter().map(|x| (x * 127.0).normalized().into()).collect::<Vec<_>>();
    }
}

impl Builder for MeshBuilder {
    type Element = Mesh;
    type Key = MeshKey;

    // Build the mesh
    fn build(self, slotmap: &mut slotmap::SlotMap<Self::Key, Self::Element>) -> Self::Key {
        // Create the OpenGL mesh
        let mesh = self.inner;
        if mesh.vertices.is_empty() { 
            // We simply don't have any vertices to render
            return Some(mesh);
        } 

        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut mesh.vertex_array_object);
            gl::BindVertexArray(mesh.vertex_array_object);

            // We can create all the buffers at once
            let mut buffers = [0_u32; 6];
            gl::GenBuffers(1, buffers.as_mut_ptr());

            // Create the EBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[0]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.indices.len() * size_of::<u32>()) as isize,
                mesh.indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::GenBuffers(5, buffers.as_mut_ptr().add(1));
            // Normal, fallback
            // Create the vertex buffer and populate it
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers[1]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.positions.len() * size_of::<f32>() * 3) as isize,
                self.vertices.positions.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Vertex attrib array
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Vertex normals attribute
            if !self.vertices.normals.is_empty() {
                // Vertex normals buffer
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[2]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.normals.len() * size_of::<i8>() * 3) as isize,
                    self.vertices.normals.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 3, gl::BYTE, gl::TRUE, 0, null());
            } else {
                gl::VertexAttrib4Nbv(1, [127, 127, 127, 0_i8].as_ptr());
            }

            if !self.vertices.tangents.is_empty() {
                // And it's brother, the tangent buffer
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[3]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.tangents.len() * size_of::<i8>() * 4) as isize,
                    self.vertices.tangents.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

                // Tangent attribute
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 4, gl::BYTE, gl::TRUE, 0, null());
            } else {
                gl::VertexAttrib4Nbv(2, [0, 0, 0, 127_i8].as_ptr());
            }

            if !self.vertices.uvs.is_empty() {
                // The texture coordinates buffer
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[4]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.uvs.len() * size_of::<u8>() * 2) as isize,
                    self.vertices.uvs.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

                // UV attribute
                gl::EnableVertexAttribArray(3);
                gl::VertexAttribPointer(3, 2, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
            } else {
                gl::VertexAttrib4Nub(3, 255, 255, 0, 0);
            }

            if !self.vertices.colors.is_empty() {
                // Vertex colors buffer
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[5]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.colors.len() * size_of::<u8>() * 3) as isize,
                    self.vertices.colors.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

                // Vertex colors attribute
                gl::EnableVertexAttribArray(4);
                gl::VertexAttribPointer(4, 3, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
            } else {
                gl::VertexAttrib4Nub(4, 255, 255, 255, 0);
            }
            // Unbind
            mesh.buffers = buffers;
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        slotmap.insert(mesh)
    }
}