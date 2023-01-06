use assets::Asset;
use graphics::{IndexBuffer, Graphics};
use obj::TexturedVertex;
use crate::attributes::{RawPosition, RawNormal, RawTexCoord};
use crate::{AttributeBuffer, EnabledMeshAttributes, MeshImportSettings, MeshImportError};
use crate::mesh::attributes::{TexCoord, Color, Tangent, Normal, Position};

// A mesh is a collection of 3D vertices connected by triangles
#[cfg(not(feature = "two-dim"))]
pub struct Mesh {
    // Enabled mesh attributes
    enabled: EnabledMeshAttributes,

    // Vertex attribute buffers
    positions: AttributeBuffer<Position>,
    normals: AttributeBuffer<Normal>,
    tangents: AttributeBuffer<Tangent>,
    colors: AttributeBuffer<Color>,
    uvs: AttributeBuffer<TexCoord>,

    // The number of vertices stored in this mesh
    len: usize,

    // The triangle buffer
    triangles: IndexBuffer<u32>,
}

// A mesh is a collection of 2D vertices connected by triangles
#[cfg(feature = "two-dim")]
pub struct Mesh {
    // Enabled mesh attributes
    enabled: EnabledMeshAttributes,

    // Vertex attribute buffers
    positions: AttributeBuffer<Position>,
    colors: AttributeBuffer<Color>,

    // The number of vertices stored in this mesh
    len: usize,

    // The triangle buffer
    triangles: IndexBuffer<u32>,
}

impl Asset for Mesh {
    type Context<'ctx> = &'ctx Graphics;
    type Settings<'stg> = MeshImportSettings;
    type Err = MeshImportError;

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize<'c, 's>(
        data: assets::Data,
        context: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        let graphics = context;

        // Load the .Obj mesh
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes()).unwrap();

        // Create temporary vectors containing the vertex attributes
        let capacity = parsed.vertices.len();
        let mut positions = Vec::<RawPosition>::with_capacity(capacity);
        let mut normals = Vec::<RawNormal>::with_capacity(capacity);
        let mut tex_coords = Vec::<RawTexCoord>::with_capacity(capacity);
        let mut triangles = Vec::<[u32; 3]>::with_capacity(parsed.indices.len() / 3);
        let indices = parsed.indices;
        use vek::{Vec2, Vec3, Mat4};

        // Convert the translation/rotation/scale settings to a unified matrix
        let translation = Mat4::<f32>::translation_3d(settings.translation);
        let rotation = Mat4::<f32>::from(settings.rotation);
        let scale = Mat4::<f32>::scaling_3d(settings.scale);
        let matrix = translation * rotation * scale;

        // Convert the vertices into the separate buffer
        for vertex in parsed.vertices {
            // Read and add the position
            positions.push(Vec3::from_slice(&vertex.position));

            // Read and add the normal
            let read = Vec3::from_slice(&vertex.normal);
            let viewed = read.map(|f| (f * 127.0) as i8);
            normals.push(viewed);

            // Read and add the texture coordinate
            let read = Vec2::from_slice(&vertex.texture);
            let viewed = read.map(|f| (f * 255.0) as u8);
            tex_coords.push(viewed);
        }

        // Convert the indices to triangles
        for triangle in indices.chunks_exact(3) {
            triangles.push(triangle.try_into().unwrap());
        }

        todo!()

        /*
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
        */

        /*
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
            Some(positions),
            normals,
            tangents,
            None,
            tex_coords,
            triangles,
        )
        .unwrap()
        */
    }
}