use assets::Asset;
use math::AABB;

use super::{attributes::{AttributeSet, named::Position}, GeometryBuilder, VertexAssembly};
use crate::{
    buffer::{Buffer, BufferMode, ElementBuffer},
    canvas::ToRasterBuffers,
    context::Context,
};

// Specified what attributes are enabled in a vertex set
bitflags::bitflags! {
    pub struct VertexLayout: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 2;
        const TANGENTS = 1 << 3;
        const COLORS = 1 << 4;
        const TEX_COORD_0 = 1 << 5;
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::empty()
    }
}

// A submesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct SubMesh {
    // The vertex attribute buffers
    attributes: AttributeSet,

    // The index buffer (PS: Supports only triangles rn)
    indices: ElementBuffer<u32>,
}

impl SubMesh {
    // Construct a unsafe submesh using a vertex assembly and a index set
    // This will not check if the vertex assembly and index set are valid
    // This will initialize a valid VAO, EBO, and the proper vertex attribute buffers
    pub unsafe fn new_unchecked(
        ctx: &mut Context,
        vertices: VertexAssembly,
        indices: Vec<u32>,
        mode: BufferMode,
    ) -> Self {
        Self {
            attributes: AttributeSet::new(ctx, mode, vertices),
            indices: Buffer::new(ctx, mode, &indices).unwrap(),
        }
    }

    // Get the current submesh's layout
    pub fn layout(&self) -> VertexLayout {
        self.attributes.layout()
    }

    // Get the underlying attribute set immutably
    pub fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    // Get the underlying attribute set mutably
    pub fn attributes_mut(&mut self) -> &mut AttributeSet {
        &mut self.attributes
    }

    // Get the underlying index buffer immutably
    pub fn indices(&self) -> &ElementBuffer<u32> {
        &self.indices
    }

    // Get the underlying index buffer mutably
    pub fn indices_mut(&mut self) -> &mut ElementBuffer<u32> {
        &mut self.indices
    }

    // Calculate the AABB of this submesh
    pub fn compute_aabb(&self) -> AABB {
        let positions = self.attributes().get_attribute_buffer::<Position>().unwrap();
        positions.

        AABB { min: todo!(), max: todo!() }
    }
}

impl<'a> Asset<'a> for SubMesh {
    type Args = (&'a mut Context, BufferMode);

    fn extensions() -> &'static [&'static str] {
        GeometryBuilder::extensions()
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        let builder = GeometryBuilder::deserialize(data, ());
        builder.build(args.0, args.1).unwrap()
    }
}

impl ToRasterBuffers for SubMesh {
    fn vao(&self) -> &AttributeSet {
        self.attributes()
    }

    fn ebo(&self) -> &ElementBuffer<u32> {
        self.indices()
    }
}
