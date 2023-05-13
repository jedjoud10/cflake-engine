use crate::{AttributeBuffer, DefaultMaterialResources, Mesh, MeshAttribute};
use graphics::{
    ActiveRenderPipeline, ColorLayout, DepthStencilLayout, DrawIndexedIndirectBuffer, GpuPod,
    SetIndexBufferError, SetVertexBufferError, TriangleBuffer, DrawIndexedError, DrawCountIndirectBuffer,
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
        + Sized
        + PartialEq<Self::AttributeBuffer<A>>;

    // Triangle buffer type used by meshes that use this render path
    type TriangleBuffer<T: GpuPod>: 'static
        + Send
        + Sync
        + Sized
        + PartialEq<Self::TriangleBuffer<T>>;

    // Optional argument that is stored within each mesh
    // In the Direct path, this represents the number of vertices the mesh contains
    // In the Indirect path, this contains a handle to a IndexedIndirectBuffer and an offset
    // In the IndirectMultiDraw path, this contains a handle to a IndexedIndirectBuffer, offset, and count (to use multi draw)
    type Args: 'static + Send + Sync + Sized + PartialEq<Self::Args>;

    // Get a mesh using it's handles from the shared material resources
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self>;

    // Checks if a mesh is valid for rendering
    fn is_valid(
        defaults: &DefaultMaterialResources,
        mesh: &Mesh<Self>,
    ) -> bool;

    // Get the number of vertices of a mesh
    // Return none if Indirect 
    fn vertex_count(mesh: &Mesh<Self>) -> Option<usize>;

    // Get the number of triangles of a mesh
    // Return none if Indirect 
    fn triangle_count(mesh: &Mesh<Self>) -> Option<usize>;

    // Sets the vertex buffer of a specific mesh into the given active graphics pipeline
    fn set_vertex_buffer<'a, C: ColorLayout, DS: DepthStencilLayout, A: MeshAttribute>(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::AttributeBuffer<A>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError>;

    // Sets the triangle buffer of a specific mesh into the given active graphics pipeline
    fn set_index_buffer<'a, C: ColorLayout, DS: DepthStencilLayout>(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError>;

    // Draws a mesh surface into the given active graphics pipeline
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), DrawIndexedError>;
}

// Render path variants
pub struct Direct;
pub struct Indirect;
pub struct MultiDrawIndirect;
pub struct MultiDrawIndirectCount;

impl RenderPath for Direct {
    type AttributeBuffer<A: MeshAttribute> = AttributeBuffer<A>;
    type TriangleBuffer<T: GpuPod> = TriangleBuffer<T>;
    type Args = Option<usize>;

    #[inline(always)]
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self> {
        defaults.meshes.get(handle)
    }

    #[inline(always)]
    fn is_valid(
        _defaults: &DefaultMaterialResources,
        mesh: &Mesh<Self>,
    ) -> bool {
        mesh.vertices().len().is_some()
    }

    #[inline(always)]
    fn vertex_count(mesh: &Mesh<Self>) -> Option<usize> {
        Some(mesh.vertices().len().unwrap_or_default())
    }

    #[inline(always)]
    fn triangle_count(mesh: &Mesh<Self>) -> Option<usize> {
        Some(mesh.triangles().buffer().len())
    }

    #[inline(always)]
    fn set_vertex_buffer<'a, C: ColorLayout, DS: DepthStencilLayout, A: MeshAttribute>(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::AttributeBuffer<A>,
        _defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError> {
        active.set_vertex_buffer::<A::V>(slot, buffer, bounds)
    }

    #[inline(always)]
    fn set_index_buffer<'a, C: ColorLayout, DS: DepthStencilLayout>(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        _defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError> {
        active.set_index_buffer(buffer, bounds)
    }

    #[inline(always)]
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        _defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), DrawIndexedError> {
        let indices = 0..(mesh.triangles().buffer().len() as u32 * 3);
        active.draw_indexed(indices, 0..1)
    }
}

// Used for Indirect meshes that can fetch their draw count and vertex count offset directly from the GPU
#[derive(PartialEq, Eq)]
pub struct IndirectMeshArgs {
    pub indirect: Handle<DrawIndexedIndirectBuffer>,
    pub offset: usize,
}

// Used for MultiDrawIndirect meshes that can dispatch multiple GPU draw calls on the GPU using a single multi draw call
#[derive(PartialEq, Eq)]
pub struct MultiDrawIndirectArgs {
    pub indirect: Handle<DrawIndexedIndirectBuffer>,
    pub offset: usize,
    pub count: usize,
}

// Used for MultiDrawIndirect meshes that can dispatch multiple GPU draw calls on the GPU using a single multi draw call
// based on the data stored within a secondary "count" buffer
#[derive(PartialEq, Eq)]
pub struct MultiDrawIndirectCountArgs {
    pub indirect: Handle<DrawIndexedIndirectBuffer>,
    pub count: Handle<DrawCountIndirectBuffer>,
    pub indirect_offset: usize,
    pub count_offset: usize,
}


impl RenderPath for Indirect {
    type AttributeBuffer<A: MeshAttribute> = Handle<AttributeBuffer<A>>;
    type TriangleBuffer<T: GpuPod> = Handle<TriangleBuffer<T>>;
    type Args = IndirectMeshArgs;

    #[inline(always)]
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self> {
        defaults.indirect_meshes.get(handle)
    }

    #[inline(always)]
    fn is_valid(
        _defaults: &DefaultMaterialResources,
        _mesh: &Mesh<Self>,
    ) -> bool {
        true
    }

    #[inline(always)]
    fn vertex_count(_mesh: &Mesh<Self>) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn triangle_count(_mesh: &Mesh<Self>) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), DrawIndexedError> {
        let handle = mesh.indirect().clone();
        let buffer = defaults.draw_indexed_indirect_buffers.get(&handle);
        active.draw_indexed_indirect(buffer, mesh.offset())
    }

    #[inline(always)]
    fn set_vertex_buffer<'a, C: ColorLayout, DS: DepthStencilLayout, A: MeshAttribute>(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &Self::AttributeBuffer<A>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError> {
        let buffer = A::indirect_buffer_from_defaults(defaults, buffer);
        active.set_vertex_buffer::<A::V>(slot, buffer, bounds)
    }

    #[inline(always)]
    fn set_index_buffer<'a, C: ColorLayout, DS: DepthStencilLayout>(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError> {
        let buffer = defaults.indirect_triangles.get(buffer);
        active.set_index_buffer(buffer, bounds)
    }
}

impl RenderPath for MultiDrawIndirect {
    type AttributeBuffer<A: MeshAttribute> = Handle<AttributeBuffer<A>>;
    type TriangleBuffer<T: GpuPod> = Handle<TriangleBuffer<T>>;
    type Args = MultiDrawIndirectArgs;

    #[inline(always)]
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self> {
        defaults.multi_draw_indirect_meshes.get(handle)
    }

    #[inline(always)]
    fn is_valid(
        defaults: &DefaultMaterialResources,
        mesh: &Mesh<Self>,
    ) -> bool {
        let buffer = defaults.draw_indexed_indirect_buffers.get(mesh.indirect());
        buffer.len() >= (mesh.offset() + mesh.count())
    }

    #[inline(always)]
    fn vertex_count(_mesh: &Mesh<Self>) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn triangle_count(_mesh: &Mesh<Self>) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn set_vertex_buffer<'a, C: ColorLayout, DS: DepthStencilLayout, A: MeshAttribute>(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &Self::AttributeBuffer<A>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError> {
        let buffer = A::indirect_buffer_from_defaults(defaults, buffer);
        active.set_vertex_buffer::<A::V>(slot, buffer, bounds)
    }

    #[inline(always)]
    fn set_index_buffer<'a, C: ColorLayout, DS: DepthStencilLayout>(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError> {
        let buffer = defaults.indirect_triangles.get(buffer);
        active.set_index_buffer(buffer, bounds)
    }

    #[inline(always)]
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), DrawIndexedError> {
        let handle = mesh.indirect().clone();
        let buffer = defaults.draw_indexed_indirect_buffers.get(&handle);
        active.multi_draw_indexed_indirect(buffer, mesh.offset(), mesh.count())
    }
}

impl RenderPath for MultiDrawIndirectCount {
    type AttributeBuffer<A: MeshAttribute> = Handle<AttributeBuffer<A>>;
    type TriangleBuffer<T: GpuPod> = Handle<TriangleBuffer<T>>;
    type Args = MultiDrawIndirectCountArgs;

    #[inline(always)]
    fn get<'a>(
        defaults: &DefaultMaterialResources<'a>,
        handle: &Handle<Mesh<Self>>,
    ) -> &'a Mesh<Self> {
        defaults.multi_draw_indirect_count_meshes.get(handle)
    }

    #[inline(always)]
    fn is_valid(
        defaults: &DefaultMaterialResources,
        mesh: &Mesh<Self>,
    ) -> bool {
        true
    }

    #[inline(always)]
    fn vertex_count(_mesh: &Mesh<Self>) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn triangle_count(_mesh: &Mesh<Self>) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn set_vertex_buffer<'a, C: ColorLayout, DS: DepthStencilLayout, A: MeshAttribute>(
        slot: u32,
        bounds: impl RangeBounds<usize>,
        buffer: &Self::AttributeBuffer<A>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetVertexBufferError> {
        let buffer = A::indirect_buffer_from_defaults(defaults, buffer);
        active.set_vertex_buffer::<A::V>(slot, buffer, bounds)
    }

    #[inline(always)]
    fn set_index_buffer<'a, C: ColorLayout, DS: DepthStencilLayout>(
        bounds: impl RangeBounds<usize>,
        buffer: &'a Self::TriangleBuffer<u32>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), SetIndexBufferError> {
        let buffer = defaults.indirect_triangles.get(buffer);
        active.set_index_buffer(buffer, bounds)
    }

    #[inline(always)]
    fn draw<'a, C: ColorLayout, DS: DepthStencilLayout>(
        mesh: &'a Mesh<Self>,
        defaults: &DefaultMaterialResources<'a>,
        active: &mut ActiveRenderPipeline<'_, 'a, '_, C, DS>,
    ) -> Result<(), DrawIndexedError> {
        let indirect = mesh.indirect().clone();
        let count = mesh.count().clone();
        let indirect = defaults.draw_indexed_indirect_buffers.get(&indirect);
        let count = defaults.draw_count_indirect_buffer.get(&count);
        todo!()
    }
}