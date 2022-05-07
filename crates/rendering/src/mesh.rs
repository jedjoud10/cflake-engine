use crate::{Buffer, Context};

// A mesh is a collection of 3D vertices connected by triangles
// Each mesh must contain a single material associated with it
pub struct Mesh {
    // Vertex array object that combines all of the attributes and the EBO together
    vao: u32,

    // Vertex attributes
    positions: Buffer<vek::Vec3<f32>>,
    normals: Buffer<vek::Vec3<i8>>,
    tangents: Buffer<vek::Vec4<i8>>,
    colors: Buffer<vek::Vec3<u8>>,
    tex_coord: Buffer<vek::Vec2<u8>>,

    // Indices
    indices: Buffer<u32>,
}

impl Mesh {
    // Create a new empty mesh that can be modified later
    fn new(_ctx: &Context) -> Self {
        Self {
            vao: 0,
            positions: Buffer::new(_ctx, false),
            normals: Buffer::new(_ctx, false),
            tangents: Buffer::new(_ctx, false),
            colors: Buffer::new(_ctx, false),
            tex_coord: Buffer::new(_ctx, false),
            indices: Buffer::new(_ctx, false),
        }
    }
}
