use std::{
    cell::Cell,
    mem::{size_of, MaybeUninit},
    ptr::null,
};

use arrayvec::ArrayVec;
use assets::Asset;
use math::AABB;
use obj::TexturedVertex;

use super::{
    AttributeBuffer, EnabledAttributes, TrianglesMut, TrianglesRef, MeshImportSettings, VerticesMut, VerticesRef,
};
use super::attributes::*;
use crate::{
    buffer::{ArrayBuffer, Buffer, BufferFormatAny, BufferMode, ElementBuffer, TriangleBuffer},
    context::Context,
    object::{Shared, ToGlName},
};

// A mesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct Mesh {
    // Enabled vertex attributes, and GL name
    pub(crate) vao: u32,
    pub(crate) enabled: EnabledAttributes,

    // Vertex attribute buffers
    pub(super) positions: AttributeBuffer<Position>,
    pub(super) normals: AttributeBuffer<Normal>,
    pub(super) tangents: AttributeBuffer<Tangent>,
    pub(super) colors: AttributeBuffer<Color>,
    pub(super) uvs: AttributeBuffer<TexCoord>,

    // The triangle buffer (triangles * 3)
    triangles: TriangleBuffer<u32>,
}

/*

        let mut arr = self
            .attributes_any()
            .into_iter()
            .map(|opt|
                opt.map(|(buf, _)| buf.len()
            )
        );

        let first = arr.find(|opt| opt.is_some()).flatten()?;
        let valid = arr.into_iter().flatten().all(|len| len == first);
        valid.then(|| first)
*/

impl Mesh {
    // Create a new mesh from the attribute buffers and the triangles
    pub fn from_buffers(
        positions: ArrayBuffer<VePosition>,
        normals: Option<ArrayBuffer<VeNormal>>,
        tangents: Option<ArrayBuffer<VeTangent>>,
        colors: Option<ArrayBuffer<VeColor>>,
        tex_coord: Option<ArrayBuffer<VeTexCoord>>,
        triangles: TriangleBuffer<u32>,
    ) -> Option<Self> {
        let mut mesh = Self { 
            vao: unsafe {
                let mut vao = 0;
                gl::CreateVertexArrays(1, &mut vao);
                vao
            },
            enabled: EnabledAttributes::empty(),
            positions: MaybeUninit::uninit(),
            normals: MaybeUninit::uninit(),
            tangents: MaybeUninit::uninit(),
            colors: MaybeUninit::uninit(),
            uvs: MaybeUninit::uninit(),
            triangles,
        };

        // Set the vertex buffers (including the position buffer)
        let mut vertices = mesh.vertices_mut();
        vertices.set_attribute::<Position>(Some(positions));
        vertices.set_attribute::<Normal>(normals);
        vertices.set_attribute::<Tangent>(tangents);
        vertices.set_attribute::<Color>(colors);
        vertices.set_attribute::<TexCoord>(tex_coord);
    
        // Bind the vertex buffers and check for valididity
        let valid = vertices.rebind(true);
        std::mem::forget(vertices);

        // Bind the triangle buffer
        mesh.triangles_mut().rebind(true);
        
        valid.then_some(mesh)
    }

    // Get a reference to the vertices immutably
    pub fn vertices(&self) -> VerticesRef {
        VerticesRef {
            positions: &self.positions,
            normals: &self.normals,
            tangents: &self.tangents,
            colors: &self.colors,
            uvs: &self.uvs,
            bitfield: self.enabled,
        }
    }

    // Get a reference to the vertices mutably
    pub fn vertices_mut(&mut self) -> VerticesMut {
        VerticesMut {
            vao: self.vao,
            positions: &mut self.positions,
            normals: &mut self.normals,
            tangents: &mut self.tangents,
            colors: &mut self.colors,
            uvs: &mut self.uvs,
            bitfield: &mut self.enabled,
            maybe_reassigned: EnabledAttributes::empty(),
        }
    }

    // Get a reference to the triangles immutably
    pub fn triangles(&self) -> TrianglesRef {
        TrianglesRef {
            buffer: &self.triangles,
        }
    }

    // Get a reference to the triangles mutably
    pub fn triangles_mut(&mut self) -> TrianglesMut {
        TrianglesMut {
            vao: self.vao,
            buffer: &mut self.triangles,
            maybe_reassigned: false,
        }
    }

    // Get the triangles and vertices both at the same time, immutably
    pub fn both(&self) -> (TrianglesRef, VerticesRef) {
        (TrianglesRef {
            buffer: &self.triangles,
        }, VerticesRef {
            positions: &self.positions,
            normals: &self.normals,
            tangents: &self.tangents,
            colors: &self.colors,
            uvs: &self.uvs,
            bitfield: self.enabled,
        })
    }
    
    // Get thr triangles and vertices both at the same time, mutably
    pub fn both_mut(&mut self) -> (TrianglesMut, VerticesMut) {
        (TrianglesMut {
            vao: self.vao,
            buffer: &mut self.triangles,
            maybe_reassigned: false,
        }, VerticesMut {
            vao: self.vao,
            positions: &mut self.positions,
            normals: &mut self.normals,
            tangents: &mut self.tangents,
            colors: &mut self.colors,
            uvs: &mut self.uvs,
            bitfield: &mut self.enabled,
            maybe_reassigned: EnabledAttributes::empty(),
        })
    }

    // Recalculate the vertex normals procedurally; based on position attribute
    pub fn compute_normals(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {
        let (triangles, mut vertices) = if self.vertices().is_enabled::<Position>() {
            self.both_mut()
        } else {
            return None;
        };

        // Fetch the buffers and map them
        let mapped_positions = vertices.attribute::<Position>().unwrap().map().unwrap();
        let positions = mapped_positions.as_slice();
        let mapped_triangles = triangles.data().map().unwrap();
        let triangles = mapped_triangles.as_slice();

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
        let buffer = Buffer::from_slice(ctx, normals.as_slice(), mode).unwrap();

        // Drop the buffers manually
        drop(mapped_positions);
        drop(mapped_triangles);

        // Insert the new buffer
        vertices.set_attribute::<Normal>(Some(buffer));
        Some(())
    }

    // Recalculate the tangents procedurally; based on normal, position, and texture coordinate attributes
    pub fn compute_tangents(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {       
        let valid_attributes = self.vertices().layout().contains(EnabledAttributes::POSITIONS | EnabledAttributes::NORMALS | EnabledAttributes::TEX_COORDS);
        let (triangles, mut vertices) = if valid_attributes {
            self.both_mut()
        } else {
            return None;
        };

        // Get positions slice
        let mapped_positions = vertices.attribute::<Position>().unwrap().map().unwrap();
        let positions = mapped_positions.as_slice();

        // Get normals slice
        let mapped_normals = vertices.attribute::<Normal>().unwrap().map().unwrap();
        let normals = mapped_normals.as_slice();

        // Get texture coordinate slice
        let mapped_tex_coords = vertices.attribute::<TexCoord>().unwrap().map().unwrap();
        let uvs = mapped_tex_coords.as_slice();

        // Get triangles slice
        let mapped_triangles = triangles.data().map().unwrap();
        let triangles = mapped_triangles.as_slice();

        // Local struct that will implement the Geometry trait from the tangent generation lib
        struct TangentGenerator<'a> {
            positions: &'a [vek::Vec3<f32>],
            triangles: &'a [[u32; 3]],
            normals: &'a [vek::Vec3<i8>],
            uvs: &'a [vek::Vec2<u8>],
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
                self.uvs[i].map(|x| x as f32 / 255.0).into_array()
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
            uvs,
            tangents: &mut tangents,
        };

        // Generate the procedural tangents and store them
        mikktspace::generate_tangents(&mut gen).then_some(())?;
        let buffer = Buffer::from_slice(ctx, tangents.as_slice(), mode).unwrap();

        // Drop the mapped buffers manually
        drop(mapped_positions);
        drop(mapped_normals);
        drop(mapped_tex_coords);
        drop(mapped_triangles);

        // Insert the new buffer
        vertices.set_attribute::<Tangent>(Some(buffer));
        Some(())
    }

    // Recalculate the AABB bounds of this mesh
    pub fn compute_bounds(&mut self) -> AABB {
        todo!()
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl<'a> Asset<'a> for Mesh {
    type Args = (&'a mut Context, MeshImportSettings);

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        let (ctx, settings) = args;

        // Load the .Obj mesh
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes()).unwrap();

        // Create temporary vectors containing the vertex attributes
        let capacity = parsed.vertices.len();
        let mut positions = Vec::with_capacity(capacity);
        let mut normals = Vec::with_capacity(capacity);
        let mut tex_coords_0 = Vec::with_capacity(capacity);
        let mut triangles = Vec::with_capacity(parsed.indices.len() / 3);
        let indices = parsed.indices;

        use vek::{Vec2, Vec3};

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            positions.push(Vec3::from_slice(&vertex.position) * settings.scale);
            normals.push(Vec3::from_slice(&vertex.normal).map(|f| (f * 127.0) as i8));
            tex_coords_0.push(Vec2::from_slice(&vertex.texture).map(|f| (f * 255.0) as u8));
        }

        // Convert the indices to triangles
        for triangle in indices.chunks_exact(3) {
            triangles.push(triangle.try_into().unwrap());
        }

        // Create the buffers
        let positions = Buffer::from_slice(ctx, &positions, settings.mode).unwrap();
        let normals = (!settings.generate_normals).then(|| Buffer::from_slice(ctx, &normals, settings.mode).unwrap());
        let tex_coord = Some(Buffer::from_slice(ctx, &tex_coords_0, settings.mode).unwrap());
        let triangles = Buffer::from_slice(ctx, &triangles, settings.mode).unwrap();

        // Create a new mesh
        let mut mesh = Mesh::from_buffers(positions, normals, None, None, tex_coord, triangles).unwrap();

        // Generate procedural normals if requested
        if settings.generate_normals {
            mesh.compute_normals(ctx, settings.mode).unwrap();
        }

        // Generate procedural tangents if requested
        if settings.generate_tangents {
            mesh.compute_tangents(ctx, settings.mode).unwrap();
        }
        mesh
    }
}