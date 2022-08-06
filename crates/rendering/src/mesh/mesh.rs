use std::{
    mem::{MaybeUninit},
};


use assets::Asset;
use math::AABB;
use num::{Zero, One};
use obj::TexturedVertex;

use super::{
    AttributeBuffer, EnabledAttributes, TrianglesMut, TrianglesRef, MeshImportSettings, VerticesMut, VerticesRef, MeshUtils,
};
use super::attributes::*;
use crate::{
    buffer::{ArrayBuffer, Buffer, BufferMode, TriangleBuffer},
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

    // The triangle buffer (triangles * 3)
    triangles: TriangleBuffer<u32>,
}

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

        
        let buffer = Buffer::from_slice(ctx, normals.as_slice(), mode).unwrap();

        // Drop the buffers manually
        drop(mapped_positions);
        drop(mapped_triangles);

        // Insert the new buffer
        vertices.set_attribute::<Normal>(Some(buffer));
        Some(())
    }

    // Recalculate the tangents procedurally; based on normal, position, and texture coordinate attributes
    pub fn compute_tangents(&mut self, ctx: &mut Context, mode: BufferMode) -> bool {       
        let valid_attributes = self.vertices().layout().contains(EnabledAttributes::POSITIONS | EnabledAttributes::NORMALS | EnabledAttributes::TEX_COORDS);
        let (triangles, mut vertices) = if valid_attributes {
            self.both_mut()
        } else {
            return false;
        };

        // Get positions slice
        let mapped_positions = vertices.attribute::<Position>().unwrap().map().unwrap();
        let positions = mapped_positions.as_slice();

        // Get normals slice
        let mapped_normals = vertices.attribute::<Normal>().unwrap().map().unwrap();
        let normals = mapped_normals.as_slice();

        // Get texture coordinate slice
        let mapped_tex_coords = vertices.attribute::<TexCoord>().unwrap().map().unwrap();
        let tex_coords = mapped_tex_coords.as_slice();

        // Get triangles slice
        let mapped_triangles = triangles.data().map().unwrap();
        let triangles = mapped_triangles.as_slice();

        // Generate the tangents using the mesh utils 
        let tangents = MeshUtils::compute_tangents(positions, normals, tex_coords, triangles);

        // Return false if we were not able to generate the tangents
        let buffer = if tangents.is_none() {
            return false;
        } else {
            Buffer::from_slice(ctx, tangents.unwrap().as_slice(), mode).unwrap()
        };

        // Drop the mapped buffers manually
        drop(mapped_positions);
        drop(mapped_normals);
        drop(mapped_tex_coords);
        drop(mapped_triangles);

        // Insert the new buffer
        vertices.set_attribute::<Tangent>(Some(buffer));
        true
    }

    // Recalculate the AABB bounds of this mesh
    pub fn compute_bounds(&mut self) -> Option<AABB> {
        let vertices = self.vertices();
        let positions = vertices.attribute::<Position>().unwrap();
        let mapped = positions.map().unwrap();
        AABB::from_points(mapped.as_slice())
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
        let mut normals = settings.use_normals.then(|| Vec::<vek::Vec3<i8>>::with_capacity(capacity));
        let mut tex_coords = settings.use_tex_coords.then(|| Vec::<vek::Vec2<u8>>::with_capacity(capacity));
        let mut triangles = Vec::<[u32; 3]>::with_capacity(parsed.indices.len() / 3);
        let indices = parsed.indices;
        use vek::{Vec2, Vec3};

        // Convert the translation/rotation/scale settings to a unified matrix
        let translation: vek::Mat4<f32> =  vek::Mat4::translation_3d(settings.translation);
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
                let mapped = read.map(|f| (f * 127.0) as i8);
                normals.push(mapped);
            }

            // Read and add the texture coordinate
            if let Some(tex_coords) = &mut tex_coords {
                let read = Vec2::from_slice(&vertex.texture);
                let mapped = read.map(|f| (f * 255.0) as u8);
                tex_coords.push(mapped);
            }
        }

        // Convert the indices to triangles
        for triangle in indices.chunks_exact(3) {
            triangles.push(triangle.try_into().unwrap());
        }

        // Optionally generate the tangents
        let tangents = settings.use_tangents.then(|| MeshUtils::compute_tangents(&positions, &normals.unwrap(), &tex_coords.unwrap(), &triangles).unwrap());


        /*

        // Create the optional attribute vectors
        let normals = settings.use

        // Create a new mesh
        let mut mesh = Mesh::from_buffers(positions, normals, None, None, tex_coord, triangles).unwrap();

        //apply_mesh_settings(settings, &mut mesh, ctx);

        mesh
        */
        todo!()
    }
}
/*
fn apply_mesh_normals_attribute(mesh: &mut Mesh, normals: Vec<vek::Vec3<i8>>, matrix: vek::Mat4<f32>, invert: bool) {

}

fn generate_mesh_tangents_attribute(mesh: &mut Mesh, ctx: &mut Context, matrix: vek::Mat4<f32>, invert: bool) {

}

fn apply_mesh_positions_attribute(mesh: &mut Mesh, positions, matrix: vek::Mat4<f32>) {
    // Update the positions using the translation/rotation/scale matrices
    if !settings.translation.is_zero() || !settings.scale.is_one() || settings.rotation != vek::Quaternion::identity() {
        let mut verts = mesh.vertices_mut();
        let positions = verts.attribute_mut::<Position>().unwrap();
        let mut mapped = positions.map_mut().unwrap();
        

        for pos in mapped.as_slice_mut() {
            *pos = (matrix * vek::Vec4::new_point(pos.x, pos.y, pos.z)).xyz();
        }
    }
}

fn apply_mesh_tex_coord(mesh: &mut Mesh, flip_horizontal: bool, flip_vertical: bool) {
    // Invert UVs in both/one direction if needed
    if flip_horizontal || flip_vertical {
        let mut verts = mesh.vertices_mut();
        let uvs = verts.attribute_mut::<TexCoord>().unwrap();
        let mut mapped = uvs.map_mut().unwrap();
        let slice = mapped.as_slice_mut();            

        // Flip the uvs horizontally
        if flip_horizontal {
            slice.iter_mut().for_each(|uv| uv.x = 255 - uv.x);
        }

        // Flip the uvs vertically
        if settings.invert_vertical_tex_coord {
            slice.iter_mut().for_each(|uv| uv.y = 255 - uv.x);
        }
    }
}


fn apply_mesh_settings(settings: MeshImportSettings, mesh: &mut Mesh, ctx: &mut Context) {
    // Generate procedural tangents if needed
    if settings.use_tangents {
        mesh.compute_tangents(ctx, settings.mode).unwrap();
    }

    
    // Invert normals if needed
    if settings.use_normals && settings.invert_normals {
        let mut verts = mesh.vertices_mut();
        let normals = verts.attribute_mut::<Normal>().unwrap();
        let mut mapped = normals.map_mut().unwrap();
        mapped.as_slice_mut().iter_mut().for_each(|n| *n *= -1);
    }
    // Invert tangents if needed
    if settings.use_tangents && settings.invert_tangents {
        let mut verts = mesh.vertices_mut();
        let tangents = verts.attribute_mut::<Tangent>().unwrap();
        let mut mapped = tangents.map_mut().unwrap();
        mapped.as_slice_mut().iter_mut().for_each(|t| *t *= -1);
    }
}
*/