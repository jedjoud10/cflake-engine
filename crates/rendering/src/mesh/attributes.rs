use graphics::{
    Normalized, PerVertex, Vertex, VertexBuffer, VertexConfig,
    VertexInput, VertexInputInfo, XYZ, XYZW, TriangleBuffer, GpuPod, DrawIndexedIndirectBuffer, ColorLayout, DepthStencilLayout, ActiveGraphicsPipeline,
};
use paste::paste;
use utils::{Handle, Storage};
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;

use crate::{AttributeError, VerticesMut, VerticesRef, Mesh, DefaultMaterialResources, Material, Surface};

bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct MeshAttributes: u8 {
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
    MeshAttributes::all().bits.count_ones() as usize;

// Contains the underlying array buffer for a specific attribute
pub type DirectAttributeBuffer<A> = VertexBuffer<<A as MeshAttribute>::V>;
pub type IndirectAttributeBuffer<A> = Handle<VertexBuffer<<A as MeshAttribute>::V>>;

// TODO: Rename this since it's not really a rendering thing, more like mesh thing
// TODO: Move this out of here. It does not fit in this file
pub trait RenderPath: 'static + Send + Sync + Sized { 
    type AttributeBuffer<A: MeshAttribute>: 'static + Send + Sync + Sized;
    type TriangleBuffer<T: GpuPod>: 'static + Send + Sync + Sized;
    type Count: 'static + Send + Sync + Sized;

    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>
    ) -> &'a Mesh<Self>;

    fn draw<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
    >(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    );
}

// Direct and indirect mesh variants
pub struct Direct;
pub struct Indirect;


impl RenderPath for Direct { 
    type AttributeBuffer<A: MeshAttribute> = DirectAttributeBuffer<A>;
    type TriangleBuffer<T: GpuPod> = TriangleBuffer<T>;
    type Count = Option<usize>;

    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>
    ) -> &'a Mesh<Self> {
        defaults.meshes.get(&handle)
    }

    fn draw<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
    >(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) {
        let indices =
            0..(mesh.triangles().buffer().len() as u32 * 3);
        active.draw_indexed(indices, 0..1);
    }
}

impl RenderPath for Indirect {
    type AttributeBuffer<A: MeshAttribute> = IndirectAttributeBuffer<A>;
    type TriangleBuffer<T: GpuPod> = Handle<TriangleBuffer<T>>;
    type Count = Handle<DrawIndexedIndirectBuffer>;

    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>
    ) -> &'a Mesh<Self> {
        defaults.indirect_meshes.get(&handle)
    }

    fn draw<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
    >(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) {
        let handle = mesh.vertices().indirect().clone();
        let buffer = defaults.draw_indexed_indirect_buffers.get(&handle);
        active.draw_indexed_indirect(buffer, 0);
    }
}


// A named attribute that has a specific name, like "Position", or "Normal"
pub trait MeshAttribute: Sized {
    type V: Vertex;
    type Input: VertexInput<Self::V>;
    const ATTRIBUTE: MeshAttributes;

    // Try to get the references to the underlying vertex buffers
    // Forgive me my children, for I have failed to bring you salvation, from this cold, dark, world...
    // WWAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
    fn from_ref_as_ref<'a, R: RenderPath>(
        vertices: &VerticesRef<'a, R>,
    ) -> Result<&'a R::AttributeBuffer<Self>, AttributeError>;
    fn from_mut_as_mut<'a, R: RenderPath>(
        vertices: &'a VerticesMut<'_, R>,
    ) -> Result<RefMut<'a, R::AttributeBuffer<Self>>, AttributeError>;
    fn from_mut_as_ref<'a, R: RenderPath>(
        vertices: &'a VerticesMut<'_, R>,
    ) -> Result<Ref<'a, R::AttributeBuffer<Self>>, AttributeError>;

    // Insert a mesh attribute vertex buffer into the vertices
    // Replaces already existing attribute buffers
    fn insert<R: RenderPath>(
        vertices: &mut VerticesMut<'_, R>,
        buffer: R::AttributeBuffer<Self>,
    );

    // Try to remove the mesh attribute vertex buffer from the vertices
    // This will return the removed attribute buffer if successful
    fn remove<R: RenderPath>(
        vertices: &mut VerticesMut<'_, R>,
    ) -> Result<R::AttributeBuffer<Self>, AttributeError>;

    // Get the attribute's index
    fn index() -> u32 {
        debug_assert_eq!(Self::ATTRIBUTE.bits().count_ones(), 1);
        Self::ATTRIBUTE.bits().trailing_zeros()
    }
}

// Get a list of the untyped attributes from the enabled mesh attributes
pub(crate) fn enabled_to_vertex_config(
    attributes: MeshAttributes,
) -> VertexConfig {
    // This will push the mesh attribute's input to the vector if the bitflags contain the vertex input
    fn push<M: MeshAttribute>(
        attributes: MeshAttributes,
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
                const ATTRIBUTE: MeshAttributes = MeshAttributes::[<$enabled>];

                fn from_ref_as_ref<'a, R: RenderPath>(
                    vertices: &VerticesRef<'a, R>,
                ) -> Result<&'a R::AttributeBuffer<Self>, AttributeError> {
                    vertices.is_enabled::<Self>().then(|| {
                        vertices.$name.as_ref().unwrap()
                    }).ok_or(AttributeError::MissingAttribute)
                }

                fn from_mut_as_mut<'a, R: RenderPath>(
                    vertices: &'a VerticesMut<'_, R>,
                ) -> Result<RefMut<'a, R::AttributeBuffer<Self>>, AttributeError> {
                    if vertices.is_enabled::<Self>() {
                        let borrowed = vertices.$name.try_borrow_mut();
                        borrowed.map(|borrowed| {
                            RefMut::map(borrowed, |x| x.as_mut().unwrap())
                        }).map_err(AttributeError::BorrowMutError)
                    } else {
                        Err(AttributeError::MissingAttribute)
                    }
                }

                fn from_mut_as_ref<'a, R: RenderPath>(
                    vertices: &'a VerticesMut<'_, R>,
                ) -> Result<Ref<'a, R::AttributeBuffer<Self>>, AttributeError> {
                    if vertices.is_enabled::<Self>() {
                        let borrowed = vertices.$name.try_borrow();
                        borrowed.map(|borrowed| {
                            Ref::map(borrowed, |x| x.as_ref().unwrap())
                        }).map_err(AttributeError::BorrowError)
                    } else {
                        Err(AttributeError::MissingAttribute)
                    }
                }

                fn insert<R: RenderPath>(
                    vertices: &mut VerticesMut<'_, R>,
                    buffer: R::AttributeBuffer<Self>,
                ) {
                    **vertices.$name.get_mut() = Some(buffer);
                    vertices.enabled.insert(Self::ATTRIBUTE);
                }

                fn remove<R: RenderPath>(
                    vertices: &mut VerticesMut<'_, R>,
                ) -> Result<R::AttributeBuffer<Self>, AttributeError> {
                    vertices.enabled.remove(Self::ATTRIBUTE);
                    vertices.$name.get_mut().take().ok_or(AttributeError::MissingAttribute)
                }
            }
        }
    };
}

impl_vertex_attribute!(
    Position,
    positions,
    XYZW<f32>,
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
