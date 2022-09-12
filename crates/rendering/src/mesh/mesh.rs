use std::mem::MaybeUninit;

use assets::Asset;
use math::AABB;

use obj::TexturedVertex;

use super::attributes::*;
use super::{
    AttributeBuffer, EnabledAttributes, MeshImportSettings, MeshUtils, TrianglesMut, TrianglesRef,
    VerticesMut, VerticesRef,
};
use crate::context::ToGlName;
use crate::{
    buffer::{ArrayBuffer, BufferMode, Triangle, TriangleBuffer},
    context::Context,
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

    // The AABB bounding box for this mesh section
    aabb: Option<AABB>,

    // The triangle buffer (triangles * 3)
    triangles: TriangleBuffer<u32>,
}

impl Mesh {
    // Create a new mesh from the attribute vectors, context, and buffer mode
    pub fn from_vecs(
        ctx: &mut Context,
        mode: BufferMode,
        positions: Vec<VePosition>,
        normals: Option<Vec<VeNormal>>,
        tangents: Option<Vec<VeTangent>>,
        colors: Option<Vec<VeColor>>,
        tex_coords: Option<Vec<VeTexCoord>>,
        triangles: Vec<Triangle<u32>>,
    ) -> Option<Self> {
        let positions = ArrayBuffer::from_slice(ctx, &positions, mode).unwrap();
        let normals = normals.map(|vec| ArrayBuffer::from_slice(ctx, &vec, mode).unwrap());
        let tangents = tangents.map(|vec| ArrayBuffer::from_slice(ctx, &vec, mode).unwrap());
        let colors = colors.map(|vec| ArrayBuffer::from_slice(ctx, &vec, mode).unwrap());
        let tex_coords = tex_coords.map(|vec| ArrayBuffer::from_slice(ctx, &vec, mode).unwrap());
        let triangles = TriangleBuffer::from_slice(ctx, &triangles, mode).unwrap();
        Self::from_buffers(positions, normals, tangents, colors, tex_coords, triangles)
    }

    // Create a new mesh from the attribute buffers
    pub fn from_buffers(
        positions: ArrayBuffer<VePosition>,
        normals: Option<ArrayBuffer<VeNormal>>,
        tangents: Option<ArrayBuffer<VeTangent>>,
        colors: Option<ArrayBuffer<VeColor>>,
        tex_coords: Option<ArrayBuffer<VeTexCoord>>,
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
            aabb: None,
            triangles,
        };

        // Set the vertex buffers (including the position buffer)
        let mut vertices = mesh.vertices_mut();
        vertices.set_attribute::<Position>(Some(positions));
        vertices.set_attribute::<Normal>(normals);
        vertices.set_attribute::<Tangent>(tangents);
        vertices.set_attribute::<Color>(colors);
        vertices.set_attribute::<TexCoord>(tex_coords);
        vertices.compute_aabb();

        // Bind the vertex buffers and check for valididity
        let valid = vertices.rebind(true);
        std::mem::forget(vertices);
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
            aabb: &mut self.aabb,
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
            bound_buffer: self.triangles.name(),
            buffer: &mut self.triangles,
        }
    }

    // Get the triangles and vertices both at the same time, immutably
    pub fn both(&self) -> (TrianglesRef, VerticesRef) {
        (
            TrianglesRef {
                buffer: &self.triangles,
            },
            VerticesRef {
                positions: &self.positions,
                normals: &self.normals,
                tangents: &self.tangents,
                colors: &self.colors,
                uvs: &self.uvs,
                bitfield: self.enabled,
            },
        )
    }

    // Get thr triangles and vertices both at the same time, mutably
    pub fn both_mut(&mut self) -> (TrianglesMut, VerticesMut) {
        (
            TrianglesMut {
                vao: self.vao,
                bound_buffer: self.triangles.name(),
                buffer: &mut self.triangles,
            },
            VerticesMut {
                vao: self.vao,
                positions: &mut self.positions,
                normals: &mut self.normals,
                tangents: &mut self.tangents,
                colors: &mut self.colors,
                uvs: &mut self.uvs,
                bitfield: &mut self.enabled,
                aabb: &mut self.aabb,
                maybe_reassigned: EnabledAttributes::empty(),
            },
        )
    }

    // Recalculate the vertex normals procedurally; based on position attribute
    pub fn compute_normals(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {
        // Fetch the buffers and map them
        let (mut triangles, mut vertices) = self.both_mut();
        let viewed_positions = vertices.attribute_mut::<Position>()?.view()?;
        let positions = viewed_positions.as_slice();
        let viewed_triangles = triangles.data_mut().view().unwrap();
        let triangles = viewed_triangles.as_slice();

        // Mesh utils come to the rescue yet again
        let normals = MeshUtils::compute_normals(positions, triangles)?;

        // Return false if we were not able to generate the normals
        let buffer = ArrayBuffer::from_slice(ctx, normals.as_slice(), mode).unwrap();

        // Drop the buffers manually
        drop(viewed_positions);
        drop(viewed_triangles);

        // Insert the new buffer
        vertices.set_attribute::<Normal>(Some(buffer));
        Some(())
    }

    // Recalculate the tangents procedurally; based on normal, position, and texture coordinate attributes
    pub fn compute_tangents(&mut self, ctx: &mut Context, mode: BufferMode) -> Option<()> {
        let (triangles, mut vertices) = self.both_mut();

        // Get positions slice
        let viewed_positions = vertices.attribute::<Position>()?.view()?;
        let positions = viewed_positions.as_slice();

        // Get normals slice
        let viewed_normals = vertices.attribute::<Normal>()?.view()?;
        let normals = viewed_normals.as_slice();

        // Get texture coordinate slice
        let viewed_tex_coords = vertices.attribute::<TexCoord>()?.view()?;
        let tex_coords = viewed_tex_coords.as_slice();

        // Get triangles slice
        let viewed_triangles = triangles.data().view()?;
        let triangles = viewed_triangles.as_slice();

        // Generate the tangents using the mesh utils
        let tangents = MeshUtils::compute_tangents(positions, normals, tex_coords, triangles)?;

        // Return false if we were not able to generate the tangents
        let buffer = ArrayBuffer::from_slice(ctx, tangents.as_slice(), mode).unwrap();

        // Drop the viewed buffers manually
        drop(viewed_positions);
        drop(viewed_normals);
        drop(viewed_tex_coords);
        drop(viewed_triangles);

        // Insert the new buffer
        vertices.set_attribute::<Tangent>(Some(buffer));
        Some(())
    }

    // Update the AABB of the mesh using updated position vertices
    pub fn compute_aabb(&mut self) -> Option<()> {
        let vertices = self.vertices();
        let positions = vertices.attribute::<Position>()?;
        let view = positions.view()?;
        let slice = view.as_slice();
        let temp = MeshUtils::aabb_from_points(slice);
        drop(view);
        self.aabb = temp;

        Some(())
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
        let mut positions = Vec::<vek::Vec3<f32>>::with_capacity(capacity);
        let mut normals = settings
            .use_normals
            .then(|| Vec::<vek::Vec3<i8>>::with_capacity(capacity));
        let mut tex_coords = settings
            .use_tex_coords
            .then(|| Vec::<vek::Vec2<u8>>::with_capacity(capacity));
        let mut triangles = Vec::<[u32; 3]>::with_capacity(parsed.indices.len() / 3);
        let indices = parsed.indices;
        use vek::{Vec2, Vec3};

        // Convert the translation/rotation/scale settings to a unified matrix
        let translation: vek::Mat4<f32> = vek::Mat4::translation_3d(settings.translation);
        let rotation: vek::Mat4<f32> = vek::Mat4::from(settings.rotation);
        let scale: vek::Mat4<f32> = vek::Mat4::scaling_3d(settings.scale);
        let matrix = translation * rotation * scale;

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            // Read and add the position
            positions.push(vek::Vec3::from_slice(&vertex.position));

            // Read and add the normal
            if let Some(normals) = &mut normals {
                let read = Vec3::from_slice(&vertex.normal);
                let viewed = read.map(|f| (f * 127.0) as i8);
                normals.push(viewed);
            }

            // Read and add the texture coordinate
            if let Some(tex_coords) = &mut tex_coords {
                let read = Vec2::from_slice(&vertex.texture);
                let viewed = read.map(|f| (f * 255.0) as u8);
                tex_coords.push(viewed);
            }
        }

        // Convert the indices to triangles
        for triangle in indices.chunks_exact(3) {
            triangles.push(triangle.try_into().unwrap());
        }

        // Optionally generate the tangents
        let mut tangents = settings.use_tangents.then(|| {
            MeshUtils::compute_tangents(
                &positions,
                normals.as_ref().unwrap(),
                tex_coords.as_ref().unwrap(),
                &triangles,
            )
            .unwrap()
        });

        // Finally, create the mesh and generate it's new GPU data
        MeshUtils::apply_vec_settings(
            settings,
            matrix,
            &mut positions,
            &mut normals,
            &mut tangents,
            &mut tex_coords,
            &mut triangles,
        );
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
