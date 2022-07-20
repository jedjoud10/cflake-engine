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
    mesh::MeshImportMode,
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
        triangles: ElementBuffer<u32>,
    ) -> Option<Self> {
        todo!()
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

    // Specify the buffer bindings for all the enabled vertex attributes and EBO
    // This will only re-bind the buffer that are marked as "maybe reassigned" since they might be unlinked
    /*
    pub(crate) unsafe fn bind_buffers_lazy(&self) {
        // Check if we even need to rebind the buffers in the first place
        let copy = self.maybe_reassigned.get();
        if copy.is_empty() {
           return;
        }

        // Bind all the active attribute buffers at the start (create the binding triangles)
        let iter = self.attributes_any().into_iter().filter_map(|s| s).enumerate();
        for (i, (buffer, attrib)) in iter {
            if copy.contains(attrib.layout()) {
                gl::VertexArrayVertexBuffer(self.vao, i as u32, buffer.name(), 0, buffer.stride() as i32);
                gl::VertexArrayAttribBinding(self.vao, attrib.attribute_index(), i as u32)
            }
        }

        // Rebind the EBO to the VAO
        if copy.contains(MeshBuffers::INDICES) {
            gl::VertexArrayElementBuffer(self.vao, self.triangles.assume_init_ref().name());
        }

        // Reset
        self.maybe_reassigned.set(MeshBuffers::empty());
    }
    */

    /*
    // Recalculate the vertex normals procedurally; based on position attribute
    pub fn compute_normals(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {
        assert!(self.is_attribute_active::<Position>(), "Position attribute is not enabled");

        // Get positions buffer and mapping
        let mapped_positions = self.attribute::<Position>().unwrap().map();
        let positions = mapped_positions.as_slice();

        // Get index buffer and mapping
        let mapped_indices = self.triangles().map();
        let triangles = mapped_indices.as_slice();
        assert!(triangles.len() % 3 == 0, "Index count is not multiple of 3");

        // Create pre-allocated normal buffer
        let mut normals = vec![vek::Vec3::<f32>::zero(); positions.len()];

        // Normal calculations
        for i in 0..(triangles.len() / 3) {
            let i1 = triangles[i * 3] as usize;
            let i2 = triangles[i * 3 + 1] as usize;
            let i3 = triangles[i * 3 + 2] as usize;

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
        let buffer = Some(Buffer::from_slice(ctx, normals.as_slice(), mode));

        // Drop the buffers manually
        drop(mapped_positions);
        drop(mapped_indices);

        // Insert the new buffer
        self.set_attribute::<Normal>(buffer);
        Some(())
    }

    // Recalculate the tangents procedurally; based on normal, position, and texture coordinate attributes
    pub fn compute_tangents(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {
        assert!(self.is_attribute_active::<Position>(), "Position attribute is not enabled");

        // Get positions slice
        let mapped_positions = self.attribute::<Position>().unwrap().map();
        let positions = mapped_positions.as_slice();

        // Get normals slice
        let mapped_normals = self.attribute::<Normal>().unwrap().map();
        let normals = mapped_normals.as_slice();

        // Get texture coordinate slice
        let mapped_tex_coords = self.attribute::<TexCoord>().unwrap().map();
        let uvs = mapped_tex_coords.as_slice();

        // Get index slice
        let mapped_indices = self.triangles().map();
        let triangles = mapped_indices.as_slice();
        assert!(triangles.len() % 3 == 0, "Index count is not multiple of 3");

        // Local struct that will implement the Geometry trait from the tangent generation lib
        struct TangentGenerator<'a> {
            positions: &'a [vek::Vec3<f32>],
            triangles: &'a [u32],
            normals: &'a [vek::Vec3<i8>],
            uvs: &'a [vek::Vec2<u8>],
            tangents: &'a mut [vek::Vec4<i8>],
        }

        impl<'a> mikktspace::Geometry for TangentGenerator<'a> {
            fn num_faces(&self) -> usize {
                self.triangles.len() / 3
            }

            fn num_vertices_of_face(&self, _face: usize) -> usize {
                3
            }

            fn position(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.triangles[face * 3 + vert] as usize;
                self.positions[i].into_array()
            }

            fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.triangles[face * 3 + vert] as usize;
                self.normals[i].map(|x| x as f32 / 127.0).into_array()
            }

            fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
                let i = self.triangles[face * 3 + vert] as usize;
                self.uvs[i].map(|x| x as f32 / 255.0).into_array()
            }

            fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
                let i = self.triangles[face * 3 + vert] as usize;
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
        let buffer = Some(Buffer::from_slice(ctx, tangents.as_slice(), mode));

        // Drop the mapped buffers manually
        drop(mapped_positions);
        drop(mapped_normals);
        drop(mapped_tex_coords);
        drop(mapped_indices);

        // Insert the new buffer
        self.set_attribute::<Tangent>(buffer);
        Some(())
    }

    // Clear the underlying mesh, making it invisible and dispose of the buffers
    pub fn clear(&mut self, ctx: &mut Context) {
        let mode = BufferMode::Static;
        *self.indices_mut() = Buffer::empty(ctx, mode);
        self.attribute_mut::<Position>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<Normal>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<Tangent>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<Color>().map(|buf| *buf = Buffer::empty(ctx, mode));
        self.attribute_mut::<TexCoord>().map(|buf| *buf = Buffer::empty(ctx, mode));
    }

    // Recalculate the AABB bounds of this mesh
    pub fn compute_bounds(&mut self) -> AABB {
        todo!()
    }
    */
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

/*
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
        let triangles = parsed.triangles;

        use vek::{Vec2, Vec3};

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            positions.push(Vec3::from_slice(&vertex.position) * settings.scale);
            normals.push(Vec3::from_slice(&vertex.normal).map(|f| (f * 127.0) as i8));
            tex_coords_0.push(Vec2::from_slice(&vertex.texture).map(|f| (f * 255.0) as u8));
        }

        // Convert the mesh mode into the valid buffer modes
        let mode = match settings.mode {
            MeshImportMode::Static => BufferMode::Static,
            MeshImportMode::Dynamic => BufferMode::Dynamic,
            MeshImportMode::Procedural => BufferMode::Resizable,
        };

        // Create the buffers
        let positions = Buffer::from_slice(ctx, &positions, mode);
        let normals = Buffer::from_slice(ctx, &normals, mode);
        let tex_coord = Buffer::from_slice(ctx, &tex_coords_0, mode);
        let triangles = Buffer::from_slice(ctx, &triangles, mode);

        // Create a new mesh
        let mut mesh = Mesh::from_buffers(positions, Some(normals), None, None, Some(tex_coord), triangles).unwrap();

        // Generate procedural tangents if requested
        if settings.generate_tangents {
            //mesh.compute_tangents(ctx, mode).unwrap();
        }
        mesh
    }
}
*/
