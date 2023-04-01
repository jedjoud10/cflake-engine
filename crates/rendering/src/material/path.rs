use crate::{
    AttributeBuffer, DefaultMaterialResources, Mesh, MeshAttribute,
};
use graphics::{
    ActiveGraphicsPipeline, ColorLayout, DepthStencilLayout,
    DrawIndexedIndirectBuffer, GpuPod, SetIndexBufferError,
    SetVertexBufferError, TriangleBuffer,
};
use std::ops::RangeBounds;
use utils::Handle;

// This is a render path that a material can use to render it's meshes and surfaces
// There are two render paths currently available: Direct and Indirect
// The direct rendering path is for normal meshes that use normal buffers
// The indirect rendering path is for meshes that share buffers and use a DrawIndexedIndirect buffer to draw their surfaces
pub trait RenderPath: 'static + Send + Sync + Sized {
    // Attribute buffer types used by meshes that use this render path
    type AttributeBuffer<A: MeshAttribute>: 'static
        + Send
        + Sync
        + Sized;

    // Triangle buffer type used by meshes that use this render path
    type TriangleBuffer<T: GpuPod>: 'static + Send + Sync + Sized;

    // Either Option<usize> or a handle to DrawIndexedIndirectBuffer
    type Count: 'static + Send + Sync + Sized;

    // Get a mesh using it's handles from the shared material resources
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self>;

    // Checks if a mesh is valid for rendering
    fn is_valid(mesh: &Mesh<Self>) -> bool;

    // Sets the vertex buffer of a specific mesh into the given active graphics pipeline
    fn set_vertex_buffer<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
        A: MeshAttribute,
    >(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::AttributeBuffer<A>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError>;

    // Sets the triangle buffer of a specific mesh into the given active graphics pipeline
    fn set_index_buffer<'a, C: ColorLayout, DS: DepthStencilLayout>(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError>;

    // Draws a mesh surface into the given active graphics pipeline
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    );
}

// Direct and indirect render path variants
pub struct Direct;
pub struct Indirect;

impl RenderPath for Direct {
    type AttributeBuffer<A: MeshAttribute> = AttributeBuffer<A>;
    type TriangleBuffer<T: GpuPod> = TriangleBuffer<T>;
    type Count = Option<usize>;

    #[inline(always)]
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self> {
        defaults.meshes.get(&handle)
    }

    #[inline(always)]
    fn is_valid(mesh: &Mesh<Self>) -> bool {
        mesh.vertices().len().is_some()
    }

    #[inline(always)]
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) {
        let indices = 0..(mesh.triangles().buffer().len() as u32 * 3);
        active.draw_indexed(indices, 0..1);
    }

    #[inline(always)]
    fn set_vertex_buffer<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
        A: MeshAttribute,
    >(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::AttributeBuffer<A>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError> {
        active.set_vertex_buffer::<A::V>(slot, buffer, bounds)
    }

    #[inline(always)]
    fn set_index_buffer<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
    >(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError> {
        active.set_index_buffer(buffer, bounds)
    }
}

impl RenderPath for Indirect {
    type AttributeBuffer<A: MeshAttribute> =
        Handle<AttributeBuffer<A>>;
    type TriangleBuffer<T: GpuPod> = Handle<TriangleBuffer<T>>;
    type Count = (Handle<DrawIndexedIndirectBuffer>, usize);

    #[inline(always)]
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self> {
        defaults.indirect_meshes.get(&handle)
    }

    #[inline(always)]
    fn is_valid(_: &Mesh<Self>) -> bool {
        true
    }

    #[inline(always)]
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) {
        let handle = mesh.indirect().clone();
        let buffer =
            defaults.draw_indexed_indirect_buffers.get(&handle);
        active.draw_indexed_indirect(buffer, mesh.offset());
    }

    #[inline(always)]
    fn set_vertex_buffer<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
        A: MeshAttribute,
    >(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &Self::AttributeBuffer<A>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError> {
        let buffer =
            A::indirect_buffer_from_defaults(defaults, &buffer);
        active.set_vertex_buffer::<A::V>(slot, &buffer, bounds)
    }

    #[inline(always)]
    fn set_index_buffer<
        'a,
        C: ColorLayout,
        DS: DepthStencilLayout,
    >(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveGraphicsPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError> {
        let buffer = defaults.indirect_triangles.get(&buffer);
        active.set_index_buffer(&buffer, bounds)
    }
}
