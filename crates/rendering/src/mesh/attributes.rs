use graphics::{
    Normalized, PerVertex, Vertex, VertexBuffer, VertexConfig,
    VertexInput, VertexInputInfo, XYZ, XYZW,
};
use paste::paste;
use std::marker::PhantomData;

use crate::{VerticesMut, VerticesRef};

bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledMeshAttributes: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;

        // TODO: Reimplement the color attribute
        //const COLORS = 1 << 3;
        const TEX_COORDS = 1 << 3;
    }
}

// This is the maximum number of active attributes that we can have inside a mesh
pub const MAX_MESH_VERTEX_ATTRIBUTES: usize =
    EnabledMeshAttributes::all().bits.count_ones() as usize;

// Contains the underlying array buffer for a specific attribute
pub type AttributeBuffer<A> = VertexBuffer<<A as MeshAttribute>::V>;

// A named attribute that has a specific name, like "Position", or "Normal"
pub trait MeshAttribute {
    type V: Vertex;
    type Input: VertexInput<Self::V>;
    const ATTRIBUTE: EnabledMeshAttributes;

    // Try to get the references to the underlying vertex buffers
    // Forgive me my children, for I have failed to bring you salvation, from this cold, dark, world...
    // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
    fn from_ref_as_ref<'a>(
        vertices: &VerticesRef<'a>,
    ) -> Option<&'a AttributeBuffer<Self>>;
    fn from_mut_as_mut<'a>(
        vertices: &'a mut VerticesMut,
    ) -> Option<&'a mut AttributeBuffer<Self>>;
    fn from_mut_as_ref<'a>(
        vertices: &'a VerticesMut,
    ) -> Option<&'a AttributeBuffer<Self>>;

    // Insert a mesh attribute vertex buffer into the vertices
    fn insert(
        vertices: &mut VerticesMut,
        buffer: AttributeBuffer<Self>,
    );

    // Try to remove the mesh attribute vertex buffer from the vertices
    fn remove(
        vertices: &mut VerticesMut,
    ) -> Option<AttributeBuffer<Self>>;

    // Get the attribute's index
    fn index() -> u32 {
        debug_assert_eq!(Self::ATTRIBUTE.bits().count_ones(), 1);
        Self::ATTRIBUTE.bits().trailing_zeros()
    }
}

// Get a list of the untyped attributes from the enabled mesh attributes
pub(crate) fn enabled_to_vertex_config(
    attributes: EnabledMeshAttributes,
) -> VertexConfig {
    // This will push the mesh attribute's input to the vector if the bitflags contain the vertex input
    fn push<M: MeshAttribute>(
        attributes: EnabledMeshAttributes,
        inputs: &mut Vec<VertexInputInfo>,
    ) {
        if attributes.contains(M::ATTRIBUTE) {
            let input = <M::Input as VertexInput<M::V>>::info();
            inputs.push(input);
        }
    }

    // Add the different types of mesh attributes
    let mut inputs = Vec::<VertexInputInfo>::new();
    push::<Position>(attributes, &mut inputs);
    push::<Normal>(attributes, &mut inputs);
    push::<Tangent>(attributes, &mut inputs);
    push::<TexCoord>(attributes, &mut inputs);

    VertexConfig { inputs }
}

macro_rules! impl_vertex_attribute {
    ($attribute:ident, $name:ident, $vertex:ty, $enabled:ident, $input:ident) => {
        paste! {
            pub struct $attribute(PhantomData<$vertex>);
            pub type [<Raw $attribute>] = <<$attribute as MeshAttribute>::V as Vertex>::Storage;

            impl MeshAttribute for $attribute {
                type V = $vertex;
                type Input = $input<Self::V>;
                const ATTRIBUTE: EnabledMeshAttributes = EnabledMeshAttributes::[<$enabled>];

                fn from_ref_as_ref<'a>(vertices: &VerticesRef<'a>) -> Option<&'a AttributeBuffer<Self>> {
                    vertices.is_enabled::<Self>().then(|| unsafe {
                        vertices.$name.assume_init_ref()
                    })
                }

                fn from_mut_as_mut<'a>(vertices: &'a mut VerticesMut) -> Option<&'a mut AttributeBuffer<Self>> {
                    vertices.is_enabled::<Self>().then(|| unsafe {
                        vertices.$name.assume_init_mut()
                    })
                }

                fn from_mut_as_ref<'a>(vertices: &'a VerticesMut) -> Option<&'a AttributeBuffer<Self>> {
                    vertices.is_enabled::<Self>().then(|| unsafe {
                        vertices.$name.assume_init_ref()
                    })
                }

                fn insert(vertices: &mut VerticesMut, buffer: AttributeBuffer<Self>) {
                    if vertices.is_enabled::<Self>() {
                        let mut old = std::mem::replace(vertices.$name, std::mem::MaybeUninit::new(buffer));
                        unsafe { old.assume_init_drop() }
                    } else {
                        *vertices.$name = std::mem::MaybeUninit::new(buffer);
                    }

                    vertices.enabled.insert(Self::ATTRIBUTE);
                }

                fn remove<'a>(vertices: &mut VerticesMut<'a>) -> Option<AttributeBuffer<Self>> {
                    vertices.enabled.remove(Self::ATTRIBUTE);
                    vertices.is_enabled::<Self>().then(|| {
                        std::mem::replace(vertices.$name, std::mem::MaybeUninit::uninit())
                    }).map(|x| unsafe { x.assume_init() })
                }
            }
        }
    };
}

impl_vertex_attribute!(
    Position,
    positions,
    XYZ<f32>,
    POSITIONS,
    PerVertex
);
impl_vertex_attribute!(
    Normal,
    normals,
    XYZW<Normalized<i8>>,
    NORMALS,
    PerVertex
);
impl_vertex_attribute!(
    Tangent,
    tangents,
    XYZW<Normalized<i8>>,
    TANGENTS,
    PerVertex
);
//impl_vertex_attribute!(Color, colors, XYZ<Normalized<u8>>, COLORS);
impl_vertex_attribute!(
    TexCoord,
    tex_coords,
    XYZW<Normalized<u8>>,
    TEX_COORDS,
    PerVertex
);
